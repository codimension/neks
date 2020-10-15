const VERSION: i64 = 1;

use std::path::PathBuf;
use structopt::StructOpt;

use neks::ines::RomFileParser;
// TODO: Create type within ines for this. User of the emulation lib shouldn't have to know about nom
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

    let io_result = RomFileParser::load(opt.input);
    match io_result {
        Ok(parser) => match parser.parse() {
            Ok((_, cartridge)) => {
                println!("Initializing CPU");
                println!("PRG ROM: {:x?}", &cartridge.prg_rom_data[0..256]);
                let mut cpu = CPU::init(cartridge);
                cpu.run();
            },
            Err(err) => println!("Couldn't parse the cartridge file: {}", err),
        },
        Err(e) => println!("IO error: {}", e),
    }
}