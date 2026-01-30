use anyhow::{Context, Result};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use ffmpeg_next as ffmpeg;

/// Video frame data
#[derive(Clone, Debug, serde::Serialize)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA data
    pub timestamp: f64,
}

/// Audio frame data
#[derive(Clone, Debug)]
pub struct AudioFrame {
    pub samples: Vec<f32>,
    pub timestamp: f64,
}

/// Frame data sent from decoder thread
#[derive(Clone, Debug)]
pub enum FrameData {
    Video(VideoFrame),
    Audio(AudioFrame),
    EndOfFile,
}

/// Commands sent to decoder thread
pub enum DecoderCommand {
    Load(String, Option<Sender<VideoFrame>>), // path + optional video frame sender
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f32),
}

/// Decoder thread handle
pub struct MediaDecoder {
    command_sender: Sender<DecoderCommand>,
    frame_receiver: Receiver<FrameData>,
    info_receiver: Receiver<DecoderInfo>,
}

/// Decoder information
#[derive(Clone, Debug)]
pub struct DecoderInfo {
    pub has_video: bool,
    pub has_audio: bool,
    pub video_width: u32,
    pub video_height: u32,
    pub duration: f64,
    pub file_path: Option<String>,
}

impl MediaDecoder {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = bounded(32);
        let (frame_tx, frame_rx) = unbounded();
        let (info_tx, info_rx) = bounded(1);

        // Spawn decoder thread
        std::thread::spawn(move || {
            decoder_thread(cmd_rx, frame_tx, info_tx);
        });

        Self {
            command_sender: cmd_tx,
            frame_receiver: frame_rx,
            info_receiver: info_rx,
        }
    }

    pub fn load(
        &self,
        path: &str,
        video_sender: Option<Sender<VideoFrame>>,
    ) -> Result<DecoderInfo> {
        self.command_sender
            .send(DecoderCommand::Load(path.to_string(), video_sender))
            .map_err(|_| anyhow::anyhow!("Decoder thread closed"))?;

        // Wait for decoder info
        match self.info_receiver.recv() {
            Ok(info) => Ok(info),
            Err(_) => Err(anyhow::anyhow!("Decoder info channel closed")),
        }
    }

    pub fn play(&self) -> Result<()> {
        self.command_sender
            .send(DecoderCommand::Play)
            .map_err(|_| anyhow::anyhow!("Decoder thread closed"))?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.command_sender
            .send(DecoderCommand::Pause)
            .map_err(|_| anyhow::anyhow!("Decoder thread closed"))?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.command_sender
            .send(DecoderCommand::Stop)
            .map_err(|_| anyhow::anyhow!("Decoder thread closed"))?;
        Ok(())
    }

    pub fn seek(&self, time: f64) -> Result<()> {
        self.command_sender
            .send(DecoderCommand::Seek(time))
            .map_err(|_| anyhow::anyhow!("Decoder thread closed"))?;
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> Result<()> {
        self.command_sender
            .send(DecoderCommand::SetVolume(volume))
            .map_err(|_| anyhow::anyhow!("Decoder thread closed"))?;
        Ok(())
    }

    pub fn try_recv_frame(&self) -> Option<FrameData> {
        self.frame_receiver.try_recv().ok()
    }

    pub fn recv_frame(&self) -> Result<FrameData> {
        self.frame_receiver
            .recv()
            .map_err(|_| anyhow::anyhow!("Frame channel closed"))
    }
}

impl Default for MediaDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decoder thread function
fn decoder_thread(
    cmd_rx: Receiver<DecoderCommand>,
    frame_tx: Sender<FrameData>,
    info_tx: Sender<DecoderInfo>,
) {
    // Decoder state
    let mut input_context: Option<ffmpeg::format::context::Input> = None;
    let mut audio_decoder: Option<ffmpeg::decoder::Audio> = None;
    let mut video_decoder: Option<ffmpeg::decoder::Video> = None;
    let mut audio_resampler: Option<ffmpeg::software::resampling::context::Context> = None;
    let mut video_scaler: Option<ffmpeg::software::scaling::context::Context> = None;
    let mut audio_stream_index: Option<usize> = None;
    let mut video_stream_index: Option<usize> = None;
    let mut audio_time_base: Option<ffmpeg::Rational> = None;
    let mut video_time_base: Option<ffmpeg::Rational> = None;
    let mut volume: f32 = 0.8;
    let mut is_playing = false;
    let mut _file_path: Option<String> = None;
    let mut duration: f64 = 0.0;
    let mut has_video = false;
    let mut has_audio = false;
    let mut video_sender: Option<Sender<VideoFrame>> = None;

    loop {
        // Check for commands (non-blocking)
        match cmd_rx.try_recv() {
            Ok(DecoderCommand::Load(path, vsender)) => {
                video_sender = vsender;
                // Initialize FFmpeg
                let _ = ffmpeg::init();

                // Open file
                match ffmpeg::format::input(&path) {
                    Ok(mut ictx) => {
                        // Find streams
                        let mut audio_idx = None;
                        let mut video_idx = None;

                        for (i, stream) in ictx.streams().enumerate() {
                            match stream.parameters().medium() {
                                ffmpeg::media::Type::Audio if audio_idx.is_none() => {
                                    audio_idx = Some(i);
                                }
                                ffmpeg::media::Type::Video if video_idx.is_none() => {
                                    video_idx = Some(i);
                                }
                                _ => {}
                            }
                        }

                        // Setup audio decoder
                        if let Some(idx) = audio_idx {
                            let stream = ictx.stream(idx).unwrap();
                            let codec_params = stream.parameters();
                            audio_time_base = Some(stream.time_base());

                            let mut decoder_context = ffmpeg::codec::Context::new();
                            if decoder_context.set_parameters(codec_params).is_ok() {
                                if let Ok(decoder) = decoder_context.decoder().audio() {
                                    // Create resampler
                                    if let Ok(resampler) =
                                        ffmpeg::software::resampling::context::Context::get(
                                            decoder.format(),
                                            decoder.channel_layout(),
                                            decoder.rate(),
                                            ffmpeg::format::Sample::F32(
                                                ffmpeg::format::sample::Type::Planar,
                                            ),
                                            ffmpeg::channel_layout::ChannelLayout::STEREO,
                                            44100,
                                        )
                                    {
                                        audio_decoder = Some(decoder);
                                        audio_resampler = Some(resampler);
                                        audio_stream_index = Some(idx);
                                        has_audio = true;
                                    }
                                }
                            }
                        }

                        // Setup video decoder
                        let mut video_width = 0;
                        let mut video_height = 0;
                        if let Some(idx) = video_idx {
                            let stream = ictx.stream(idx).unwrap();
                            let codec_params = stream.parameters();
                            video_time_base = Some(stream.time_base());

                            let mut decoder_context = ffmpeg::codec::Context::new();
                            if decoder_context.set_parameters(codec_params).is_ok() {
                                if let Ok(decoder) = decoder_context.decoder().video() {
                                    video_width = decoder.width();
                                    video_height = decoder.height();

                                    // Create scaler
                                    if let Ok(scaler) =
                                        ffmpeg::software::scaling::context::Context::get(
                                            decoder.format(),
                                            decoder.width(),
                                            decoder.height(),
                                            ffmpeg::format::Pixel::RGBA,
                                            decoder.width(),
                                            decoder.height(),
                                            ffmpeg::software::scaling::flag::Flags::BILINEAR,
                                        )
                                    {
                                        video_decoder = Some(decoder);
                                        video_scaler = Some(scaler);
                                        video_stream_index = Some(idx);
                                        has_video = true;
                                    }
                                }
                            }
                        }

                        duration = ictx.duration() as f64 / 1_000_000.0;
                        _file_path = Some(path.clone());
                        input_context = Some(ictx);

                        // Send decoder info
                        let info = DecoderInfo {
                            has_video,
                            has_audio,
                            video_width,
                            video_height,
                            duration,
                            file_path: Some(path),
                        };
                        let _ = info_tx.send(info);
                    }
                    Err(e) => {
                        eprintln!("Failed to open file: {}", e);
                    }
                }
            }
            Ok(DecoderCommand::Play) => {
                is_playing = true;
            }
            Ok(DecoderCommand::Pause) => {
                is_playing = false;
            }
            Ok(DecoderCommand::Stop) => {
                is_playing = false;
                // Reset decoders
                input_context = None;
                audio_decoder = None;
                video_decoder = None;
                audio_resampler = None;
                video_scaler = None;
            }
            Ok(DecoderCommand::Seek(time)) => {
                if let Some(ref mut ictx) = input_context {
                    let timestamp = (time * 1_000_000.0) as i64;
                    let _ = ictx.seek(timestamp, ..);

                    // Flush decoders
                    if let Some(ref mut dec) = audio_decoder {
                        dec.flush();
                    }
                    if let Some(ref mut dec) = video_decoder {
                        dec.flush();
                    }
                }
            }
            Ok(DecoderCommand::SetVolume(v)) => {
                volume = v.clamp(0.0, 1.0);
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                break;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {}
        }

        // Decode frames if playing
        if is_playing {
            if let Some(ref mut ictx) = input_context {
                // Get next packet
                match ictx.packets().next() {
                    Some((stream, packet)) => {
                        let stream_idx = stream.index();

                        // Process audio packet
                        if Some(stream_idx) == audio_stream_index {
                            if let Some(ref mut decoder) = audio_decoder {
                                if decoder.send_packet(&packet).is_ok() {
                                    let mut frame = ffmpeg::frame::Audio::empty();
                                    while decoder.receive_frame(&mut frame).is_ok() {
                                        // Resample
                                        if let Some(ref mut resampler) = audio_resampler {
                                            let mut resampled = ffmpeg::frame::Audio::empty();
                                            if resampler.run(&frame, &mut resampled).is_ok() {
                                                // Extract samples
                                                let sample_count = resampled.samples();
                                                let channels =
                                                    resampled.channel_layout().channels() as usize;
                                                let mut samples =
                                                    Vec::with_capacity(sample_count * channels);

                                                for i in 0..sample_count {
                                                    for ch in 0..channels.min(2) {
                                                        let plane_data = resampled.data(ch);
                                                        let offset = i * 4;
                                                        if offset + 4 <= plane_data.len() {
                                                            let bytes =
                                                                &plane_data[offset..offset + 4];
                                                            let value = f32::from_ne_bytes([
                                                                bytes[0], bytes[1], bytes[2],
                                                                bytes[3],
                                                            ]);
                                                            samples.push(value * volume);
                                                        }
                                                    }
                                                }

                                                let timestamp = frame
                                                    .timestamp()
                                                    .map(|ts| {
                                                        ts as f64
                                                            * f64::from(audio_time_base.unwrap())
                                                    })
                                                    .unwrap_or(0.0);

                                                let _ =
                                                    frame_tx.send(FrameData::Audio(AudioFrame {
                                                        samples,
                                                        timestamp,
                                                    }));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Process video packet
                        if Some(stream_idx) == video_stream_index {
                            if let Some(ref mut decoder) = video_decoder {
                                if decoder.send_packet(&packet).is_ok() {
                                    let mut frame = ffmpeg::frame::Video::empty();
                                    while decoder.receive_frame(&mut frame).is_ok() {
                                        // Scale to RGBA
                                        if let Some(ref mut scaler) = video_scaler {
                                            let mut scaled = ffmpeg::frame::Video::empty();
                                            if scaler.run(&frame, &mut scaled).is_ok() {
                                                let width = scaled.width();
                                                let height = scaled.height();
                                                let data = scaled.data(0).to_vec();

                                                let timestamp = frame
                                                    .timestamp()
                                                    .map(|ts| {
                                                        ts as f64
                                                            * f64::from(video_time_base.unwrap())
                                                    })
                                                    .unwrap_or(0.0);

                                                // Send video frame to frontend if sender is available
                                                if let Some(ref sender) = video_sender {
                                                    let _ = sender.send(VideoFrame {
                                                        width,
                                                        height,
                                                        data,
                                                        timestamp,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        // End of file
                        let _ = frame_tx.send(FrameData::EndOfFile);
                        is_playing = false;
                    }
                }
            } else {
                // No file loaded, yield
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        } else {
            // Not playing, yield
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
