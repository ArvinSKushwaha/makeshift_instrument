use flume::Sender;
use thiserror::Error;

use crate::playback::{Playback, PlaybackError, PlaybackCreationError};

pub struct Engine {
    playback: Playback,
    channel: Sender<f32>,
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
        let (playback, channel) = Playback::new()?;

        Ok(Self {
            playback,
            channel,
        })
    }
}
