const VERSION: i64 = 1;

use std::path::PathBuf;
use structopt::StructOpt;

use neks::ines::Cartridge;
use neks::cpu::CPU;

#[derive(Debug, StructOpt)]
#[structopt(name = "neks", about = "NES emulator")]
struct CommandLineOptions {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    println!("Neks version {}", VERSION);

    let opt = CommandLineOptions::from_args();
    println!("Found file: {:?}", opt.input);

    let c = Cartridge::read(opt.input).expect("Couldn't open the requested cartridge");
    let s = Vec::from(&c.data[0..256]);
    println!("Data dump: {:?}", s);

    println!("Initializing CPU");
    let mut cpu = CPU::new(c);
    cpu.run();
}