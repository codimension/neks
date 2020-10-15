use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
use std::path::Path;
use std::ops::Index;

use bitflags::*;

use nom::IResult;
use nom::error::VerboseError;
use nom::multi::count;
use nom::sequence::tuple;
use nom::number::complete::le_u8;
use nom::bytes::complete::{tag, take};

type CartridgeError<'a> = VerboseError<&'a [u8]>;

#[derive(Debug)]
pub struct Header {
    pub prg_rom_size: u8,
    pub chr_rom_size: u8,
    pub flags_6: Flags6,
    pub flags_7: u8,
    pub flags_8: u8,
    pub flags_9: u8,
    pub flags_10: u8,
}

bitflags! {
    pub struct Flags6: u8 {
        const mirroring = 0b00000001;
        const persistent_ram = 0b00000010;
        const trainer_present = 0b00000100;
    }
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header, CartridgeError> {
    match tuple((
        tag(b"NES\x1A"),
        le_u8, le_u8,
        le_u8, le_u8,
        le_u8, le_u8,
        le_u8,
        take(5usize), // padding bytes, unused in iNES
    ))(input)
    {
        Ok((remaining_input, (
            _, //TAG
            prg_size,
            chr_size,
            flags_6,
            flags_7,
            flags_8,
            flags_9,
            flags_10,
            _,
        ))) => Ok((remaining_input, Header {
            prg_rom_size: prg_size,
            chr_rom_size: chr_size,
            flags_6: Flags6::from_bits_truncate(flags_6),
            flags_7: flags_7,
            flags_8: flags_8,
            flags_9: flags_9,
            flags_10: flags_10,
    })),
        Err(e) => Err(e),
    } 
}

pub struct Cartridge {
    pub header: Header,
    pub trainer: [u8; 512],
    pub prg_rom_data: Vec<u8>,
    pub chr_rom_data: Vec<u8>,
}

fn parse_file(input: &[u8]) -> IResult<&[u8], Cartridge, CartridgeError> {
    let (i, header) = parse_header(input)?;
    let (i, trainer) = match header.flags_6.contains(Flags6::trainer_present) {
        true => {
            let (i, o) = take(512usize)(i)?;
            (i, Some(o))
        },
        false => (i, None),
    };
    let prg_size = header.prg_rom_size as usize * 16384usize; // 16KB chunks
    let chr_size = header.chr_rom_size as usize * 8192usize; // 8KB chunks
    let (i, prg_rom_data) = count(le_u8, prg_size)(i)?;
    let (i, chr_rom_data) = count(le_u8, chr_size)(i)?;
    let mut file = (i, Cartridge {
        header: header,
        trainer: [0; 512],
        prg_rom_data: prg_rom_data,
        chr_rom_data: chr_rom_data,
    });

    match trainer {
        Some(bytes) => file.1.trainer.copy_from_slice(bytes),
        None => (),
    }

    Ok(file)
}

/// This is just a data structure which owns the ROM data to be parsed
/// Allowing it to own the data makes it easier to deal with parsing errors
pub struct RomFileParser {
    data: Vec<u8>,
}

impl RomFileParser {
    /// The only way to create a RomFileParser - to load data from a file somewhere
    pub fn load<P: AsRef<Path>>(path: P) -> Result<RomFileParser, Error> {
        let mut parser = RomFileParser {
            // ROM files are likely to be at least 32KB,
            // At least for basic carts with no mappers
            data: Vec::with_capacity(32768),
        };
        File::open(path).and_then(|mut f| {
            let result = f.read_to_end(&mut parser.data);
            result.and_then(|_| {
                Ok(parser)
            })
        })
    }

    /// Parses an open file, returning the result
    pub fn parse(&self) -> IResult<&[u8], Cartridge, CartridgeError> {
        parse_file(&self.data)
    }
}

impl Index<u16> for Cartridge {
    type Output = u8;
    fn index(&self, i: u16) -> &u8 {
        &self.prg_rom_data[i as usize]
    }
}

