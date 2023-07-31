use cpal::{Sample as SampleTrait, U24};
use std::{collections::HashMap, fs::File, ops::Range, path::Path};

pub use cpal::SampleRate;
use riff::Chunk;
pub use soundfont::{data::Info, Instrument, Preset};
use soundfont::{
    data::{SampleData, SampleHeader},
    error::ParseError,
    SoundFont2,
};
use thiserror::Error;

use crate::SampleType;

#[derive(Debug)]
pub struct SoundFont {
    pub info: Info,
    pub presets: Vec<Preset>,
    pub instruments: Vec<Instrument>,
    pub samples: HashMap<String, Sample>,
}

#[derive(Debug)]
pub struct Sample {
    pub name: String,
    pub loop_range: Range<usize>,
    pub sample_rate: SampleRate,
    pub orig_pitch: u8,
    pub pitch_adj: i8,
    pub sample_link: u16,
    pub sample_type: SampleType,
    pub data: Vec<f32>,
}

#[derive(Debug, Error)]
pub enum SoundFontError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("ParseError: {0:?}")]
    ParseError(ParseError),

    #[error("Could not find samples")]
    CouldNotFindSamples,

    #[error("Encountered an odd number of samples")]
    OddNumberOfSamples,

    #[error("The number of samples between the 16 and 24 bit samples do not match")]
    SampleCountsDoNotMatch,
}

impl From<ParseError> for SoundFontError {
    fn from(value: ParseError) -> Self {
        SoundFontError::ParseError(value)
    }
}

impl Sample {
    fn try_load(
        header: soundfont::data::SampleHeader,
        smpl: &[f32],
    ) -> Result<(String, Sample), SoundFontError> {
        let SampleHeader {
            name,
            start,
            end,
            loop_start,
            loop_end,
            sample_rate,
            origpitch,
            pitchadj,
            sample_link,
            sample_type,
        } = header;
        Ok((
            name.clone(),
            Sample {
                name,
                loop_range: (loop_start.saturating_sub(start) as usize)
                    ..(loop_end.saturating_sub(start) as usize),
                sample_rate: SampleRate(sample_rate),
                orig_pitch: origpitch,
                pitch_adj: pitchadj,
                sample_link,
                sample_type,
                data: smpl[(start as usize)..(end as usize)].to_vec(),
            },
        ))
    }
}

impl SoundFont {
    pub fn open(p: impl AsRef<Path>) -> Result<SoundFont, SoundFontError> {
        let mut file = File::open(p)?;
        let SoundFont2 {
            info,
            presets,
            instruments,
            sample_headers,
            sample_data,
        } = SoundFont2::load(&mut file)?;

        let SampleData { smpl: Some(smpl), sm24 } = sample_data else {
            return Err(SoundFontError::CouldNotFindSamples)
        };

        let smpl = load_samples_from_chunk(smpl, &mut file)?;
        let sm24 = sm24.map(|sm24| load_samples_from_chunk(sm24, &mut file));

        if smpl.len() % 2 != 0 {
            return Err(SoundFontError::OddNumberOfSamples);
        }

        let smpl = {
            let mut data = Vec::with_capacity(smpl.len() >> 1);
            data.extend_from_slice(bytemuck::cast_slice(smpl.as_slice()));

            data
        };

        let samples = meld_data(smpl, sm24)?;

        let samples = sample_headers
            .into_iter()
            .map(|header| Sample::try_load(header, &samples))
            .collect::<Result<_, _>>()?;

        Ok(SoundFont {
            info,
            presets,
            instruments,
            samples,
        })
    }
}

fn load_samples_from_chunk(chunk: Chunk, file: &mut File) -> Result<Vec<u8>, SoundFontError> {
    Ok(chunk.read_contents(file)?)
}

fn meld_data(
    smpl: Vec<u16>,
    sm24: Option<Result<Vec<u8>, SoundFontError>>,
) -> Result<Vec<f32>, SoundFontError> {
    Ok(match sm24 {
        Some(sm24) => {
            let sm24 = sm24?;

            if sm24.len() != smpl.len() << 1 {
                return Err(SoundFontError::SampleCountsDoNotMatch);
            }

            smpl.into_iter()
                .zip(sm24)
                .map(|(high, low)| U24::new_unchecked((high as i32) << 8 | (low as i32)))
                .map(SampleTrait::to_float_sample)
                .collect()
        }
        None => smpl.into_iter().map(SampleTrait::to_float_sample).collect(),
    })
}
