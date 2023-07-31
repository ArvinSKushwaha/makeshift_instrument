use flume::Sender;
use thiserror::Error;

use crate::playback::{Playback, PlaybackError, PlaybackCreationError};

pub type SoundByte = [f32; 256];

pub struct Engine {
    playback: Playback,
    channel: Sender<SoundByte>,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("PlaybackError: {0}")]
    PlaybackError(#[from] PlaybackError),
    #[error("PlaybackCreationError: {0}")]
    PlaybackCreationError(#[from] PlaybackCreationError),
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        let (mut playback, channel) = Playback::new()?;
        playback.initialize_stream()?;

        Ok(Self {
            playback,
            channel,
        })
    }

    pub fn channel(&self) -> &Sender<SoundByte> {
        &self.channel
    }

    pub fn play(&self) -> Result<(), PlaybackError> {
        self.playback.play()
    }

    pub fn pause(&self) -> Result<(), PlaybackError> {
        self.playback.pause()
    }
}
