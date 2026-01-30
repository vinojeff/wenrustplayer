use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, OutputCallbackInfo, Stream, StreamConfig,
};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender, TryRecvError};
use std::thread::JoinHandle;

/// Audio output using CPAL - runs in a dedicated thread
pub struct AudioOutput {
    command_sender: Sender<AudioCommand>,
    _thread_handle: JoinHandle<()>,
    sample_rate: u32,
    channels: u16,
}

enum AudioCommand {
    Play,
    Pause,
    Stop,
}

impl AudioOutput {
    /// Create a new audio output with a sample receiver
    pub fn new(
        sample_rate: u32,
        channels: u16,
        sample_receiver: Receiver<Vec<f32>>,
    ) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("No output device available")?;

        let config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let (command_sender, command_receiver): (Sender<AudioCommand>, Receiver<AudioCommand>) =
            bounded(16);

        let thread_handle = std::thread::spawn(move || {
            let stream = match device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    // Try to receive samples from the channel
                    match sample_receiver.try_recv() {
                        Ok(samples) => {
                            // Copy samples to the output buffer
                            let len = samples.len().min(data.len());
                            data[..len].copy_from_slice(&samples[..len]);
                            // Zero out the rest of the buffer
                            for i in len..data.len() {
                                data[i] = 0.0;
                            }
                        }
                        Err(TryRecvError::Empty) => {
                            // No samples available, output silence
                            for sample in data.iter_mut() {
                                *sample = 0.0;
                            }
                        }
                        Err(TryRecvError::Disconnected) => {
                            // Channel closed, output silence
                            for sample in data.iter_mut() {
                                *sample = 0.0;
                            }
                        }
                    }
                },
                move |err| {
                    eprintln!("Audio output error: {}", err);
                },
                None,
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to build audio stream: {}", e);
                    return;
                }
            };

            // Start playing
            if let Err(e) = stream.play() {
                eprintln!("Failed to start audio stream: {}", e);
                return;
            }

            // Listen for commands
            loop {
                match command_receiver.recv() {
                    Ok(AudioCommand::Play) => {
                        let _ = stream.play();
                    }
                    Ok(AudioCommand::Pause) => {
                        let _ = stream.pause();
                    }
                    Ok(AudioCommand::Stop) => {
                        let _ = stream.pause();
                        break;
                    }
                    Err(_) => {
                        // Channel closed, exit
                        break;
                    }
                }
            }
        });

        Ok(Self {
            command_sender,
            _thread_handle: thread_handle,
            sample_rate,
            channels,
        })
    }

    /// Stop the audio stream
    pub fn stop(&self) {
        let _ = self.command_sender.send(AudioCommand::Stop);
    }

    /// Pause playback
    pub fn pause(&self) {
        let _ = self.command_sender.send(AudioCommand::Pause);
    }

    /// Resume playback
    pub fn resume(&self) {
        let _ = self.command_sender.send(AudioCommand::Play);
    }
}

/// Audio buffer for queuing samples
pub struct AudioBuffer {
    sender: Sender<Vec<f32>>,
}

impl AudioBuffer {
    pub fn new(sender: Sender<Vec<f32>>) -> Self {
        Self { sender }
    }

    /// Add samples to the buffer
    pub fn push_samples(&self, samples: &[f32]) -> anyhow::Result<()> {
        self.sender
            .send(samples.to_vec())
            .context("Failed to send samples to audio output")?;
        Ok(())
    }
}

/// Create a sample channel pair
pub fn create_sample_channel() -> (Sender<Vec<f32>>, Receiver<Vec<f32>>) {
    unbounded()
}
