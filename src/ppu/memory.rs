use std::rc::Rc;
use std::cell::RefCell;

use crate::ines::Cartridge;

pub(crate) struct GraphicsMemory {
    pub ram: [u8; 0x800],
    cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl GraphicsMemory {
    pub fn init() -> Self {
        Self {
            ram: [0; 0x800],
            cartridge: None
        }
    }
}