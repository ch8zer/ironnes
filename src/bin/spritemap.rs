fn main() {
    todo!("Sample code written as file comment")
}

//use simplelog::*;
//use std::fs::File;
//
//use sdl2::event::Event;
//use sdl2::keyboard::Keycode;
//use sdl2::pixels::Color;
//use sdl2::pixels::PixelFormatEnum;
//use sdl2::EventPump;
//
//mod debugger;
//
//use iron_nes::error::*;
//use iron_nes::nes::cartridge::Cartridge;
//use iron_nes::nes::ppu::{Frame, Ppu};
//
//fn main() {
//    let yaml = clap::load_yaml!("spritemap.yml");
//    let matches = clap::App::from_yaml(yaml)
//        .version(&*format!("v{}", clap::crate_version!()))
//        .get_matches();
//
//    let rom = matches.value_of("rom").unwrap();
//    let log_level = match matches.occurrences_of("v") {
//        0 => LevelFilter::Error,
//        1 => LevelFilter::Warn,
//        2 => LevelFilter::Info,
//        _ => LevelFilter::Trace,
//    };
//
//    let term_logger = TermLogger::new(log_level, Config::default(), TerminalMode::Mixed).unwrap();
//    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![term_logger];
//
//    if let Some(logfile) = matches.value_of("log") {
//        let file_logger = WriteLogger::new(
//            LevelFilter::Trace,
//            Config::default(),
//            File::create(logfile).unwrap(),
//        );
//        loggers.push(file_logger);
//    }
//
//    CombinedLogger::init(loggers).unwrap();
//
//    let (_, _, ppu_vram) = Cartridge::load(rom).unwrap();
//    let mut ppu = Ppu::new(ppu_vram);
//
//    // SDL STUFF
//
//    const SCALE: f32 = 2.0;
//
//    let sdl_context = sdl2::init().unwrap();
//    let video_subsystem = sdl_context.video().unwrap();
//    let window = video_subsystem
//        .window(
//            "IronNES",
//            (256.0 * 2.0 * SCALE) as u32,
//            (240.0 * SCALE) as u32,
//        )
//        .position_centered()
//        .build()
//        .unwrap();
//
//    let mut canvas = window.into_canvas().accelerated().build().unwrap();
//    let mut event_pump = sdl_context.event_pump().unwrap();
//    canvas.set_scale(SCALE, SCALE).unwrap();
//
//    let creator = canvas.texture_creator();
//    let mut texture = creator
//        .create_texture_target(PixelFormatEnum::RGB24, 256 * 2, 240)
//        .unwrap();
//
//    let mut timer = sdl_context.timer().unwrap();
//    let mut running = true;
//
//    while running {
//        for event in event_pump.poll_iter() {
//            match event {
//                Event::Quit { .. }
//                | Event::KeyDown {
//                    keycode: Some(Keycode::Escape),
//                    ..
//                } => {
//                    running = false;
//                }
//                _ => (),
//            }
//        }
//
//        let ticks = timer.ticks() as i32;
//
//        let frame = ppu.render();
//        texture.update(None, &frame.0, 256 * 2 * 3);
//        canvas.clear();
//        canvas.copy(&texture, None, None).unwrap();
//        canvas.present();
//    }
//}
