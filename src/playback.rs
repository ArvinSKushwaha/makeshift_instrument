use std::collections::VecDeque;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, PauseStreamError, PlayStreamError, Stream, StreamConfig,
    SupportedStreamConfigsError, BuildStreamError,
};
use flume::{Receiver, Sender};
use thiserror::Error;

use crate::engine::SoundByte;

pub const DEFAULT_SAMPLE_RATE: cpal::SampleRate = cpal::SampleRate(44100);

pub struct Playback {
    host: Host,
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
    buffer: VecDeque<f32>,
    recv: Receiver<SoundByte>,
}

#[derive(Debug, Error)]
pub enum PlaybackCreationError {
    #[error("No default output device")]
    NoDefaultOutputDevice,
    #[error("No supported configuration")]
    NoSupportedConfiguration,
    #[error("SupportedStreamConfigsError: {0}")]
    SupportedStreamConfigsError(#[from] SupportedStreamConfigsError),
}

#[derive(Debug, Error)]
pub enum PlaybackError {
    #[error("PlayStreamError: {0}")]
    PlayStreamError(#[from] PlayStreamError),
    #[error("PauseStreamError: {0}")]
    PauseStreamError(#[from] PauseStreamError),
    #[error("BuildStreamError: {0}")]
    BuildStreamError(#[from] BuildStreamError),
}

impl Playback {
    pub fn new() -> Result<(Self, Sender<SoundByte>), PlaybackCreationError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackCreationError::NoDefaultOutputDevice)?;

        let mut supported_configs_range = device.supported_output_configs()?;
        let supported_config = supported_configs_range
            .next()
            .ok_or(PlaybackCreationError::NoSupportedConfiguration)?
            .with_sample_rate(DEFAULT_SAMPLE_RATE);
        let config = supported_config.config();

        let (trns, recv) = flume::unbounded();

        Ok((
            Playback {
                host,
                device,
                config,
                recv,
                buffer: VecDeque::with_capacity(256),
                stream: None,
            },
            trns,
        ))
    }

    pub fn play(&self) -> Result<(), PlaybackError> {
        let stream = self.stream.as_ref().unwrap();

        stream.play()?;

        Ok(())
    }

    pub fn pause(&self) -> Result<(), PlaybackError> {
        match &self.stream {
            Some(stream) => {
                stream.pause()?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn initialize_stream(&mut self) -> Result<(), PlaybackError> {
        let recv = self.recv.clone();
        self.stream = Some(self.device.build_output_stream(
            &self.config,
            move |buffer: &mut [f32], _| {
                println!("{}", buffer.len());
                // buffer
                //     .iter_mut()
                //     .zip(recv.try_iter().chain(std::iter::repeat(0.0)))
                //     .for_each(|(b, c)| {
                //         *b = c;
                //     })
            },
            |err| eprintln!("Encountered error: {}", err),
            None,
        )?);

        Ok(())
    }
}
