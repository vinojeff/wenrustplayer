use crate::audio_output::{create_sample_channel, AudioOutput};
use crate::decoder::{DecoderInfo, FrameData, MediaDecoder, VideoFrame};
use anyhow::Result;
use crossbeam_channel::Sender;
use serde::Serialize;
use std::thread;
use std::time::Duration;

/// Playback state
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Ended,
}

/// Player status for frontend
#[derive(Clone, Serialize)]
pub struct PlayerStatus {
    pub is_playing: bool,
    pub current_time: f64,
    pub duration: f64,
    pub volume: f32,
    pub file_path: Option<String>,
    pub has_video: bool,
    pub has_audio: bool,
    pub video_width: u32,
    pub video_height: u32,
}

/// Main media player supporting both audio and video
pub struct MediaPlayer {
    decoder: MediaDecoder,
    audio_output: Option<AudioOutput>,
    sample_sender: Option<Sender<Vec<f32>>>,
    state: PlaybackState,
    current_time: f64,
    duration: f64,
    volume: f32,
    file_path: Option<String>,
    has_video: bool,
    has_audio: bool,
    video_width: u32,
    video_height: u32,
}

impl MediaPlayer {
    pub fn new() -> Self {
        Self {
            decoder: MediaDecoder::new(),
            audio_output: None,
            sample_sender: None,
            state: PlaybackState::Stopped,
            current_time: 0.0,
            duration: 0.0,
            volume: 0.8,
            file_path: None,
            has_video: false,
            has_audio: false,
            video_width: 0,
            video_height: 0,
        }
    }

    /// Load a media file with optional video frame sender
    pub fn load(
        &mut self,
        path: &str,
        video_sender: Option<Sender<VideoFrame>>,
    ) -> Result<PlayerStatus> {
        // Stop current playback
        self.stop();

        // Load file in decoder with video sender
        let info = self.decoder.load(path, video_sender)?;

        self.has_video = info.has_video;
        self.has_audio = info.has_audio;
        self.video_width = info.video_width;
        self.video_height = info.video_height;
        self.duration = info.duration;
        self.file_path = info.file_path.clone();
        self.current_time = 0.0;
        self.state = PlaybackState::Stopped;

        // Setup audio if available
        if self.has_audio {
            let (sample_sender, sample_receiver) = create_sample_channel();
            self.sample_sender = Some(sample_sender);

            self.audio_output = Some(AudioOutput::new(44100, 2, sample_receiver)?);
        }

        Ok(self.get_status())
    }

    /// Play media
    pub fn play(&mut self) -> Result<()> {
        match self.state {
            PlaybackState::Stopped | PlaybackState::Ended => {
                // Start from beginning
                self.decoder.play()?;
            }
            PlaybackState::Paused => {
                // Resume
                self.decoder.play()?;
            }
            PlaybackState::Playing => {}
        }

        if let Some(ref output) = self.audio_output {
            output.resume();
        }

        self.state = PlaybackState::Playing;
        Ok(())
    }

    /// Pause media
    pub fn pause(&mut self) -> Result<()> {
        if self.state == PlaybackState::Playing {
            self.decoder.pause()?;

            if let Some(ref output) = self.audio_output {
                output.pause();
            }

            self.state = PlaybackState::Paused;
        }
        Ok(())
    }

    /// Stop media
    pub fn stop(&mut self) {
        let _ = self.decoder.stop();

        if let Some(ref output) = self.audio_output {
            output.stop();
        }

        self.state = PlaybackState::Stopped;
        self.current_time = 0.0;
        self.audio_output = None;
        self.sample_sender = None;
    }

    /// Seek to a specific time in seconds
    pub fn seek(&mut self, time: f64) -> Result<()> {
        let time = time.clamp(0.0, self.duration);
        self.decoder.seek(time)?;
        self.current_time = time;
        Ok(())
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        self.volume = volume;
        let _ = self.decoder.set_volume(volume);
    }

    /// Get current status
    pub fn get_status(&self) -> PlayerStatus {
        PlayerStatus {
            is_playing: self.state == PlaybackState::Playing,
            current_time: self.current_time,
            duration: self.duration,
            volume: self.volume,
            file_path: self.file_path.clone(),
            has_video: self.has_video,
            has_audio: self.has_audio,
            video_width: self.video_width,
            video_height: self.video_height,
        }
    }

    /// Get current playback state
    pub fn get_state(&self) -> PlaybackState {
        self.state
    }

    /// Get volume
    pub fn get_volume(&self) -> f32 {
        self.volume
    }
}

impl Default for MediaPlayer {
    fn default() -> Self {
        Self::new()
    }
}

// Type alias for backward compatibility
pub type AudioPlayer = MediaPlayer;
