#![recursion_limit = "256"]

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

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_arch = "wasm32")]
use base64::decode;
#[cfg(target_arch = "wasm32")]
use stdweb::{
    web::window,
    {_js_impl, js},
};

#[cfg(not(target_arch = "wasm32"))]
use die::{die, Die};
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
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
    cycle: fn(&mut DrawGeometry),
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

        //#[cfg(not(target_arch = "wasm32"))]
        //Beep::start();

        Ok(DrawGeometry {
            cpu: Cpu::new(rom, BoolDisplay::new(SCALE_FACTOR), ThreadRand::new()),
            keymap,
            cycle: DrawGeometry::cycle,
        })
    }

    fn noop(&mut self) {
        // do nothing
    }

    fn cycle(&mut self) {
        self.cpu.cycle_60hz();
        SOUND_ON.store(self.cpu.sound > 0, Ordering::Relaxed);
    }

    fn toggle_debug(&mut self) {
        self.cycle = if self.cycle as usize == DrawGeometry::cycle as usize {
            DrawGeometry::noop
        } else {
            DrawGeometry::cycle
        };
    }
}

impl State for DrawGeometry {
    fn new() -> Result<DrawGeometry> {
        DrawGeometry::new_rom(&get_rom())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        // quicksilver is *supposed* to call this at exactly 60hz
        // if it doesn't, we should call .cycle() instead
        (self.cycle)(self);
        Ok(())
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        if let Event::Key(key, button_state) = event {
            let pressed = *button_state == ButtonState::Pressed;
            if pressed || *button_state == ButtonState::Released {
                match self.keymap.get(&key) {
                    Some(key) => self.cpu.keyboard.toggle_key(*key, pressed),
                    None => {
                        if pressed {
                            match *key {
                                QKey::Return => self.cpu.reset(),
                                QKey::Back => quit(),
                                QKey::Space => self.toggle_debug(),
                                QKey::I => self.toggle_debug(),
                                QKey::O => {
                                    if self.cycle as usize == DrawGeometry::noop as usize {
                                        let instruction = self.cpu.next_instruction();
                                        //println!("ins: {}", instruction);
                                        print!("ins: {}, before: {:?}", instruction, self.cpu);
                                        self.cpu.decrement_timers();
                                        self.cpu.execute_next_instruction();
                                        println!(", after : {:?}", self.cpu);
                                    }
                                }
                                QKey::Equals => self.cpu.inc_clock_rate_hz(10),
                                // todo: as a native app the _/- button is 'Subtract' but in WASM it's 'Minus'...
                                QKey::Subtract => self.cpu.inc_clock_rate_hz(-10),
                                QKey::Minus => self.cpu.inc_clock_rate_hz(-10),
                                QKey::Key0 => self.cpu.set_clock_rate_hz(500), // default chip-8 speed
                                QKey::Key9 => self.cpu.set_clock_rate_hz(1000), // default schip speed
                                _ => (), // ignore everything else
                            }
                            //println!("key: {:?}", *key);
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
        let scale_factor = self.cpu.display.get_scale();
        let size = (scale_factor, scale_factor);
        for (y, row) in self.cpu.display.get_buffer().enumerate() {
            for (x, val) in row.iter().enumerate() {
                if *val {
                    //println!("drawing point ({}, {})", j, i);
                    //display.push_str("*");
                    let x = x as u32;
                    let y = y as u32;
                    window.draw(
                        &Rectangle::new((x * scale_factor, y * scale_factor), size),
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

#[cfg(target_arch = "wasm32")]
fn quit() {
    //window().history().back().expect("can't go back?");
    // doesn't look like you can set location.href with just stdweb rust...
    //window().location().expect("can't get location?").set("./games.html").expect("can't redirect to games.html");
    js! {
    @(no_return)
    window.location.href = "./games.html";
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn quit() {
    die!("exiting..."; 0);
}

static SOUND_ON: AtomicBool = AtomicBool::new(false);

#[cfg(target_arch = "wasm32")]
fn start_audio() {
    let sound_on = || SOUND_ON.load(Ordering::Relaxed);
    js! {
        @(no_return)
        var ctxClass = window.AudioContext || window.audioContext || window.webkitAudioContext;
        var ctx = new ctxClass();

        var osc = undefined;

        var sound_on = @{sound_on};

        var on_now = false;

        setInterval(function () {
            var beep = sound_on();
            if(beep == on_now)
                return;
            on_now = beep;
            if(beep) {
                osc = ctx.createOscillator();
                // Only 0-4 are valid types.
                //osc.type = (type % 5) || 0;
                osc.type = "sine";
                osc.connect(ctx.destination);

                if (osc.noteOn) osc.noteOn(0);
                if (osc.start) osc.start();
            } else {
                if (osc.noteOff) osc.noteOff(0);
                if (osc.stop) osc.stop();
            }
        }, 16); // 1000 / 60 = 16.166667 = 60hz
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_audio() {
    thread::spawn(|| {
        let device =
            cpal::default_output_device().expect("Failed to get default audio output device");
        let format = device
            .default_output_format()
            .expect("Failed to get default audio output format");
        let event_loop = cpal::EventLoop::new();
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
        event_loop.play_stream(stream_id.clone());

        let sample_rate = format.sample_rate.0 as f32; // 44_100 on my computer
        let mut sample_clock = 0f32;

        // Produce a sinusoid of maximum amplitude.
        let mut next_value = || {
            if SOUND_ON.load(Ordering::Relaxed) {
                sample_clock = (sample_clock + 1.0) % sample_rate;
                (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
            } else {
                0.0
            }
        };

        event_loop.run(move |_, data| match data {
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = (next_value() * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = next_value();
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            _ => (),
        });
    });
}

fn main() {
    start_audio();

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
