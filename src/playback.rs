use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, PauseStreamError, PlayStreamError, Stream, StreamConfig,
    SupportedStreamConfigsError, BuildStreamError,
};
use flume::{Receiver, Sender};
use thiserror::Error;

pub struct Playback {
    host: Host,
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
    recv: Receiver<f32>,
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
    pub fn new() -> Result<(Self, Sender<f32>), PlaybackCreationError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(PlaybackCreationError::NoDefaultOutputDevice)?;

        let mut supported_configs_range = device.supported_output_configs()?;
        let supported_config = supported_configs_range
            .next()
            .ok_or(PlaybackCreationError::NoSupportedConfiguration)?
            .with_sample_rate(cpal::SampleRate(44100));
        let config = supported_config.config();

        let (trns, recv) = flume::unbounded();

        Ok((
            Playback {
                host,
                device,
                config,
                recv,
                stream: None,
            },
            trns,
        ))
    }

    pub fn play(&mut self) -> Result<(), PlaybackError> {
        if self.stream.is_none() {
            self.initialize_stream()?;
        }

        let stream = self.stream.as_ref().unwrap();

        stream.play()?;

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), PlaybackError> {
        match &self.stream {
            Some(stream) => {
                stream.pause()?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn initialize_stream(&mut self) -> Result<(), PlaybackError> {
        let recv = self.recv.clone();
        self.device.build_output_stream(
            &self.config,
            move |buffer, _| {
                buffer
                    .iter_mut()
                    .zip(recv.try_iter().chain(std::iter::repeat(0.0)))
                    .for_each(|(b, c)| {
                        *b = c;
                    })
            },
            |err| eprintln!("Encountered error: {}", err),
            None,
        )?;

        Ok(())
    }
}
