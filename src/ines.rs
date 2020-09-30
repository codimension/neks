use std::io::prelude::*;
use std::io::Error;
use std::fs::File;
use std::path::Path;
use std::ops::Index;

use crate::cpu::Register16;

pub struct Cartridge {
    pub data: Vec<u8>,
}

impl Cartridge {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        File::open(path).and_then(|mut f| {
            let mut data = Vec::new();
            let result = f.read_to_end(&mut data);
            result.and_then(|_| {
                Ok(Self{data: data})
            })
        })
    }
}

impl Index<Register16> for Cartridge {
    type Output = u8;
    fn index(&self, i: Register16) -> &u8 {
        &self.data[u16::from(i) as usize]
    }
}

