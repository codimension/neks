const VERSION: i64 = 1;

use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use structopt::StructOpt;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use neks::ines::RomFileParser;
use neks::cpu::CPU;

#[derive(Debug, StructOpt)]
#[structopt(name = "neks", about = "NES emulator")]
struct CommandLineOptions {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), String> {
    println!("Neks version {}", VERSION);

    let opt = CommandLineOptions::from_args();
    println!("Found file: {:?}", opt.input);

    let cartridge = RomFileParser::load(opt.input)
                    .unwrap()
                    .parse()
                    .map(|(_remaining, cartridge)| cartridge)
                    .unwrap();

    let mut cpu = CPU::init(cartridge);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Emulator", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                },
                _ => {}
            }
        }

        cpu.step();
        // Want to render here

        canvas.clear();
        canvas.present();
    }

    Ok(())
}