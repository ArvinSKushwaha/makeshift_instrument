use std::{fs::File, io::{Read, Cursor}, path::Path};

use self::parser::{SoundFontReadError, Sfbk, Chunk};

mod parser;

pub struct SoundFont {
    pub sfbk: Sfbk
}

impl SoundFont {
    pub fn open(mut reader: impl Read) -> Result<Self, SoundFontReadError> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        println!("{}", buf.len());
        let mut buf = Cursor::new(buf);

        Ok(SoundFont {
            sfbk: Sfbk::read(&mut buf)?
        })
    }

    pub fn open_path(path: impl AsRef<Path>) -> Result<Self, SoundFontReadError> {
        Self::open(File::open(path)?)
    }
}
