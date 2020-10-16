pub mod ines; // ines is the predominant ROM file format for NES, this implements reading the format
pub mod cpu;  // CPU functionality
pub mod memory; // Memory access functionality
pub mod ppu; // The picture processing unit