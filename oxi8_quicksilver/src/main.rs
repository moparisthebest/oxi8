// Draw some multi-colored geometry to the screen
use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Background::Col, Color, ImageScaleStrategy, ResizeStrategy},
    input::{ButtonState, Key as QKey},
    lifecycle::{run, Event, Settings, State, Window},
    Result,
};

use std::collections::HashMap;

use oxi8_cpu::{BoolDisplay, Cpu, Key, Rand, DISPLAY_HEIGHT, DISPLAY_WIDTH};

use rand::prelude::{Rng, ThreadRng};

#[cfg(target_arch = "wasm32")]
use base64::decode;
#[cfg(target_arch = "wasm32")]
use stdweb::web::window;

#[cfg(not(target_arch = "wasm32"))]
use die::Die;
#[cfg(not(target_arch = "wasm32"))]
use std::{env, fs};

const SCALE_FACTOR: u32 = 8;

struct ThreadRand {
    rng: ThreadRng,
}

impl ThreadRand {
    pub fn new() -> ThreadRand {
        ThreadRand {
            rng: rand::thread_rng(),
        }
    }
}

impl Rand for ThreadRand {
    fn next(&mut self) -> u8 {
        // 71 is good for debugging PONG
        self.rng.gen::<u8>()
    }
}

struct DrawGeometry {
    cpu: Cpu<BoolDisplay, ThreadRand>,
    keymap: HashMap<QKey, Key>,
    size: (u32, u32),
}

impl DrawGeometry {
    fn new_rom(rom: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let mut keymap = HashMap::new();
        /*
        1234  --->  123C
        QWER  --->  456D
        ASDF  --->  789E
        ZXCV  --->  A0BF
        */
        keymap.insert(QKey::Key1, Key::K1);
        keymap.insert(QKey::Key2, Key::K2);
        keymap.insert(QKey::Key3, Key::K3);
        keymap.insert(QKey::Q, Key::K4);
        keymap.insert(QKey::W, Key::K5);
        keymap.insert(QKey::E, Key::K6);
        keymap.insert(QKey::A, Key::K7);
        keymap.insert(QKey::S, Key::K8);
        keymap.insert(QKey::D, Key::K9);
        keymap.insert(QKey::X, Key::K0);
        keymap.insert(QKey::Z, Key::KA);
        keymap.insert(QKey::C, Key::KB);
        keymap.insert(QKey::Key4, Key::KC);
        keymap.insert(QKey::R, Key::KD);
        keymap.insert(QKey::F, Key::KE);
        keymap.insert(QKey::V, Key::KF);

        Ok(DrawGeometry {
            cpu: Cpu::new(rom, BoolDisplay::new(), ThreadRand::new()),
            keymap,
            size: (SCALE_FACTOR, SCALE_FACTOR),
        })
    }
}

impl State for DrawGeometry {
    fn new() -> Result<DrawGeometry> {
        DrawGeometry::new_rom(&get_rom())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        // quicksilver is *supposed* to call this at exactly 60hz
        // if it doesn't, we should call .cycle() instead
        //self.cpu.cycle();
        self.cpu.cycle_60hz();
        Ok(())
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        if let Event::Key(key, button_state) = event {
            let pressed = *button_state == ButtonState::Pressed;
            if pressed || *button_state == ButtonState::Released {
                match self.keymap.get(&key) {
                    Some(key) => self.cpu.keyboard.toggle_key(*key, pressed),
                    None => {
                        if pressed && *key == QKey::Return {
                            self.cpu.reset();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        //println!("draw");

        //window.set_size((DISPLAY_WIDTH * SCALE_FACTOR, DISPLAY_HEIGHT * SCALE_FACTOR));
        window.clear(Color::BLACK)?;

        //let mut display = String::new();
        //println!("starting draw");
        for (y, row) in self.cpu.display.get_buffer().iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                if *val {
                    //println!("drawing point ({}, {})", j, i);
                    //display.push_str("*");
                    let x = x as u32;
                    let y = y as u32;
                    window.draw(
                        &Rectangle::new((x * SCALE_FACTOR, y * SCALE_FACTOR), self.size),
                        Col(Color::WHITE),
                    );
                //rect.s
                } else {
                    //display.push_str(" ");
                }
            }
            //display.push_str("\n");
        }
        //fs::write("/tmp/display", display.as_bytes()).expect("Unable to write display file");
        //println!("finished draw");
        Ok(())
    }
}

const PONG: [u8; 246] = [
    0x6A, 0x2, 0x6B, 0xC, 0x6C, 0x3F, 0x6D, 0xC, 0xA2, 0xEA, 0xDA, 0xB6, 0xDC, 0xD6, 0x6E, 0x0,
    0x22, 0xD4, 0x66, 0x3, 0x68, 0x2, 0x60, 0x60, 0xF0, 0x15, 0xF0, 0x7, 0x30, 0x0, 0x12, 0x1A,
    0xC7, 0x17, 0x77, 0x8, 0x69, 0xFF, 0xA2, 0xF0, 0xD6, 0x71, 0xA2, 0xEA, 0xDA, 0xB6, 0xDC, 0xD6,
    0x60, 0x1, 0xE0, 0xA1, 0x7B, 0xFE, 0x60, 0x4, 0xE0, 0xA1, 0x7B, 0x2, 0x60, 0x1F, 0x8B, 0x2,
    0xDA, 0xB6, 0x60, 0xC, 0xE0, 0xA1, 0x7D, 0xFE, 0x60, 0xD, 0xE0, 0xA1, 0x7D, 0x2, 0x60, 0x1F,
    0x8D, 0x2, 0xDC, 0xD6, 0xA2, 0xF0, 0xD6, 0x71, 0x86, 0x84, 0x87, 0x94, 0x60, 0x3F, 0x86, 0x2,
    0x61, 0x1F, 0x87, 0x12, 0x46, 0x2, 0x12, 0x78, 0x46, 0x3F, 0x12, 0x82, 0x47, 0x1F, 0x69, 0xFF,
    0x47, 0x0, 0x69, 0x1, 0xD6, 0x71, 0x12, 0x2A, 0x68, 0x2, 0x63, 0x1, 0x80, 0x70, 0x80, 0xB5,
    0x12, 0x8A, 0x68, 0xFE, 0x63, 0xA, 0x80, 0x70, 0x80, 0xD5, 0x3F, 0x1, 0x12, 0xA2, 0x61, 0x2,
    0x80, 0x15, 0x3F, 0x1, 0x12, 0xBA, 0x80, 0x15, 0x3F, 0x1, 0x12, 0xC8, 0x80, 0x15, 0x3F, 0x1,
    0x12, 0xC2, 0x60, 0x20, 0xF0, 0x18, 0x22, 0xD4, 0x8E, 0x34, 0x22, 0xD4, 0x66, 0x3E, 0x33, 0x1,
    0x66, 0x3, 0x68, 0xFE, 0x33, 0x1, 0x68, 0x2, 0x12, 0x16, 0x79, 0xFF, 0x49, 0xFE, 0x69, 0xFF,
    0x12, 0xC8, 0x79, 0x1, 0x49, 0x2, 0x69, 0x1, 0x60, 0x4, 0xF0, 0x18, 0x76, 0x1, 0x46, 0x40,
    0x76, 0xFE, 0x12, 0x6C, 0xA2, 0xF2, 0xFE, 0x33, 0xF2, 0x65, 0xF1, 0x29, 0x64, 0x14, 0x65, 0x0,
    0xD4, 0x55, 0x74, 0x15, 0xF2, 0x29, 0xD4, 0x55, 0x0, 0xEE, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x80, 0x0, 0x0, 0x0, 0x0, 0x0,
];

#[cfg(target_arch = "wasm32")]
fn get_rom() -> Vec<u8> {
    match window().location() {
        Some(location) => match location.hash() {
            Ok(hash) => {
                if hash.len() > 1 {
                    match decode(&hash[1..hash.len()]) {
                        Ok(rom) => rom,
                        Err(_) => {
                            window().alert("Supplied rom invalid base64, loading default...");
                            PONG.to_vec()
                        }
                    }
                } else {
                    PONG.to_vec()
                }
            }
            Err(_) => PONG.to_vec(),
        },
        None => PONG.to_vec(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn get_rom() -> Vec<u8> {
    match env::args().nth(1) {
        Some(file_name) => fs::read(file_name).die("Unable to read first arg as rom"),
        None => PONG.to_vec(),
    }
}

fn main() {
    let settings = Settings {
        show_cursor: false,
        //min_size: Some(Vector { x: DISPLAY_WIDTH as f32 * SCALE_FACTOR, y: DISPLAY_HEIGHT as f32 * SCALE_FACTOR }),
        min_size: Some(Vector {
            x: DISPLAY_WIDTH as f32,
            y: DISPLAY_HEIGHT as f32,
        }),
        max_size: None,
        resize: ResizeStrategy::default(),
        scale: ImageScaleStrategy::default(),
        fullscreen: false,
        update_rate: 1000. / 60.,
        max_updates: 0,
        draw_rate: 0.,
        icon_path: None,
        vsync: true,
        multisampling: None,
    };

    run::<DrawGeometry>(
        "oxi8",
        Vector::new(DISPLAY_WIDTH * SCALE_FACTOR, DISPLAY_HEIGHT * SCALE_FACTOR),
        settings,
    );
    //run_with::<DrawGeometry, FnOnce()->Result<DrawGeometry>>("oxi8", Vector::new(DISPLAY_WIDTH * SCALE_FACTOR, DISPLAY_HEIGHT * SCALE_FACTOR), settings,
    //                         || DrawGeometry::new_rom(&rom)
    //);
}
