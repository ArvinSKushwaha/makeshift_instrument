use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};
use bytes::Buf;
use thiserror::Error;

#[derive(Copy, Clone, Pod, Zeroable, PartialEq, Eq)]
#[repr(transparent)]
pub struct FourCC([u8; 4]);

impl Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FourCC")
            .field(&self.0.escape_ascii().to_string())
            .finish()
    }
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(transparent)]
pub struct ChunkSize(u32);

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ChunkHeader {
    chunk_id: FourCC,
    chunk_size: ChunkSize,
}

#[derive(Debug, Error)]
pub enum SoundFontReadError {
    #[error("Not enough remaining data to read chunk")]
    NotEnoughRemainingData,

    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Could not find a matching chunk")]
    CouldNotFindMatchingChunk,

    #[error("Missing Expected Chunk")]
    MissingExpectedChunk(&'static str),
}

pub trait Chunk {
    const CHUNK_ID: FourCC;

    fn read(buf: &mut impl Buf) -> Result<Self, SoundFontReadError>
    where
        Self: Sized;
}

fn read_chunk(buf: &mut impl Buf) -> Result<ChunkHeader, SoundFontReadError> {
    if buf.remaining() < 8 {
        return Err(SoundFontReadError::NotEnoughRemainingData);
    }

    let chunk_id = FourCC(buf.get_u32().to_be_bytes());
    let chunk_size = ChunkSize(buf.get_u32_le());
    println!("{} {}", buf.remaining(), chunk_size.0);

    // buf.advance(((chunk_size.0 + 1) & (u32::MAX - 1)) as usize);

    if chunk_size.0 % 2 == 1 {
        buf.get_u8();
    }

    Ok(ChunkHeader {
        chunk_id,
        chunk_size,
    })
}

macro_rules! impl_riff_sequence_chunk_read {
    ($v:vis struct $i:ident($i_str:literal) [$b:literal] {
        $($r:ident : $r_ty:ty,)+
        $(_: $o:ident : $o_ty:ty),*
        $(,)?
    }) => {
        #[derive(Debug, Clone)]
        $v struct $i {
            $(pub $r: $r_ty,)+
            $(pub $o: Option<$o_ty>),*
        }

        impl Chunk for $i {
            const CHUNK_ID: FourCC = FourCC(*$i_str);

            fn read(buf: &mut impl Buf) -> Result<Self, SoundFontReadError> {
                let sequence_header = read_chunk(buf)?;
                println!("{} {}", buf.remaining(), std::any::type_name::<Self>());
                let mut buf = buf.copy_to_bytes(sequence_header.chunk_size.0 as usize);

                if sequence_header.chunk_id != FourCC(*$b) {
                    return Err(SoundFontReadError::MissingExpectedChunk(stringify!($b)));
                }

                let header = read_chunk(&mut buf)?;
                println!("{:?}", header);
                let mut buf = buf.copy_to_bytes(header.chunk_size.0 as usize);

                $(let mut $r = None;)+
                $(let mut $o = None;)*

                while buf.has_remaining() {
                    match header.chunk_id {
                        $(<$r_ty>::CHUNK_ID => $r = Some(<$r_ty>::read(&mut buf)?),)+
                        $(<$o_ty>::CHUNK_ID => $o = Some(<$o_ty>::read(&mut buf)?),)*
                        _ => return Err(SoundFontReadError::CouldNotFindMatchingChunk),
                    }
                }

                $(let $r = $r.ok_or(SoundFontReadError::MissingExpectedChunk(stringify!($r_ty)))?;)+

                Ok($i {
                    $($r,)+
                    $($o,)*
                })
            }
        }
    };
}

impl_riff_sequence_chunk_read! {
    pub struct Sfbk(b"sfbk") [b"RIFF"] {
        info: Info,
        sdta: Sdta,
        pdta: Pdta,
    }
}

impl_riff_sequence_chunk_read! {
    pub struct Info(b"INFO") [b"LIST"] {
        ifil: Ifil,
        isng: Isng,
        inam: Inam,
        _: irom: Irom,
        _: iver: Iver,
        _: icrd: Icrd,
        _: ieng: Ieng,
        _: iprd: Iprd,
        _: icop: Icop,
        _: icmt: Icmt,
        _: isft: Isft,
    }
}

impl_riff_sequence_chunk_read! {
    pub struct Sdta(b"sdta") [b"LIST"] {
        smpl: Smpl,
        sm24: Sm24,
    }
}

impl_riff_sequence_chunk_read! {
    pub struct Pdta(b"pdta") [b"LIST"] {
        phdr: Phdr,
        pbag: Pbag,
        pmod: Pmod,
        pgen: Pgen,
        inst: Inst,
        ibag: Ibag,
        imod: Imod,
        igen: Igen,
        shdr: Shdr,
    }
}

macro_rules! impl_riff_terminal_chunk {
    ($v:vis struct $i:ident($i_str:literal)) => {
        #[derive(Debug, Clone)]
        $v struct $i(pub Vec<u8>);

        impl Chunk for $i {
            const CHUNK_ID: FourCC = FourCC(*$i_str);

            fn read(buf: &mut impl Buf) -> Result<Self, SoundFontReadError> {
                let header = read_chunk(buf)?;
                let buf = buf.copy_to_bytes(header.chunk_size.0 as usize);
                let mut data = Vec::with_capacity(header.chunk_size.0 as usize);

                while buf.has_remaining() {
                    data.extend(buf.chunk().iter());
                }


                Ok($i(data))
            }
        }
    };
}

impl_riff_terminal_chunk! {
    pub struct Ifil(b"ifil")
}

impl_riff_terminal_chunk! {
    pub struct Isng(b"isng")
}

impl_riff_terminal_chunk! {
    pub struct Inam(b"inam")
}

impl_riff_terminal_chunk! {
    pub struct Irom(b"irom")
}

impl_riff_terminal_chunk! {
    pub struct Iver(b"iver")
}

impl_riff_terminal_chunk! {
    pub struct Icrd(b"icrd")
}

impl_riff_terminal_chunk! {
    pub struct Ieng(b"ieng")
}

impl_riff_terminal_chunk! {
    pub struct Iprd(b"iprd")
}

impl_riff_terminal_chunk! {
    pub struct Icop(b"icop")
}

impl_riff_terminal_chunk! {
    pub struct Icmt(b"icmt")
}

impl_riff_terminal_chunk! {
    pub struct Isft(b"isft")
}

impl_riff_terminal_chunk! {
    pub struct Smpl(b"smpl")
}

impl_riff_terminal_chunk! {
    pub struct Sm24(b"sm24")
}

impl_riff_terminal_chunk! {
    pub struct Phdr(b"phdr")
}

impl_riff_terminal_chunk! {
    pub struct Pbag(b"pbag")
}

impl_riff_terminal_chunk! {
    pub struct Pmod(b"pmod")
}

impl_riff_terminal_chunk! {
    pub struct Pgen(b"pgen")
}

impl_riff_terminal_chunk! {
    pub struct Inst(b"inst")
}

impl_riff_terminal_chunk! {
    pub struct Ibag(b"ibag")
}

impl_riff_terminal_chunk! {
    pub struct Imod(b"imod")
}

impl_riff_terminal_chunk! {
    pub struct Igen(b"igen")
}

impl_riff_terminal_chunk! {
    pub struct Shdr(b"shdr")
}
