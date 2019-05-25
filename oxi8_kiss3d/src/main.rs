use std::{env, fs};

use std::collections::HashMap;

use oxi8_cpu::{BoolDisplay, Cpu, Display, Key, Rand, DISPLAY_HEIGHT, DISPLAY_WIDTH};

use kiss3d::camera::{ArcBall, Camera};
use kiss3d::event::{Action, Key as KissKey, WindowEvent};
use kiss3d::light::Light;
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::renderer::Renderer;
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};

use nalgebra::{Point3, Translation3, Vector2};

use rand::prelude::{Rng, ThreadRng};

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
        self.rng.gen::<u8>()
    }
}

pub struct BlockDisplay {
    buffer: Vec<Vec<SceneNode>>,
}

impl BlockDisplay {
    pub fn new(window: &mut Window) -> BlockDisplay {
        let mut buffer = Vec::new();
        for y in 0..DISPLAY_HEIGHT {
            let mut row = Vec::new();
            for x in 0..DISPLAY_WIDTH {
                row.push(add_square(window, x as f32, -(y as f32)));
            }
            buffer.push(row);
        }

        BlockDisplay { buffer }
    }
}

impl Display for BlockDisplay {
    fn current_pixel(&self, x: usize, y: usize) -> u8 {
        let current_pixel = *(self.buffer[y][x]
            .data()
            .get_object()
            .data()
            .color()
            .coords
            .get(0)
            .unwrap_or_else(|| &0.0)) as u8;
        let current_pixel = if current_pixel == 1 { 0 } else { 1 }; // todo need to swap this if we flip colors
        current_pixel
    }

    fn set_pixel(&mut self, x: usize, y: usize, new_pixel: u8) {
        if new_pixel != 0 {
            self.buffer[y][x].set_color(0.0, 0.0, 0.0);
        } else {
            self.buffer[y][x].set_color(1.0, 1.0, 1.0);
        }
    }

    fn clear(&mut self) {
        panic!("todo BlockDisplay clear");
    }
}

fn add_square(window: &mut Window, x: f32, y: f32) -> SceneNode {
    let size = 5.0;
    //let mut rect = window.add_rectangle(size, size);
    let mut rect = window.add_cube(size, size, size);
    //rect.set_color(1.0, 1.0, 1.0);
    rect.set_color(0.0, 0.0, 0.0);
    //rect.append_translation(&Translation2::new(x * size, y * size));
    rect.append_translation(&Translation3::new(x * size, y * size, 0.0));
    rect
}

struct GLState<T: Display, R: Rand> {
    cpu: Cpu<T, R>,
    color: Point3<f32>,
    camera: ArcBall,
    keymap: HashMap<KissKey, Key>,
}

impl<T: Display, R: Rand> GLState<T, R> {
    fn common_step(&mut self, window: &mut Window) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::FramebufferSize(x, y) => {
                    let point_size = y as f32 / 32.0; // todo: this is ridiculous but seems about right...
                    println!(
                        "frame buffer size event {}, {}, point_size: {}",
                        x, y, point_size
                    );
                    window.set_point_size(point_size);
                }
                WindowEvent::MouseButton(button, Action::Press, modif) => {
                    println!("mouse press event on {:?} with {:?}", button, modif);
                    let window_size =
                        Vector2::new(window.size()[0] as f32, window.size()[1] as f32);
                    //sel_pos = camera.unproject(&last_pos, &window_size);
                    //println!("conv {:?} to {:?} win siz {:?} ", last_pos, sel_pos, window_size);
                    println!("win siz {:?} ", window_size);
                }
                WindowEvent::Key(key, action, modif) => {
                    println!("key event {:?} on {:?} with {:?}", key, action, modif);
                    match self.keymap.get(&key) {
                        Some(key) => self.cpu.keyboard.toggle_key(*key, action == Action::Press),
                        None => {}
                    }
                }
                WindowEvent::CursorPos(_x, _y, _modif) => {
                    //last_pos = na::Point2::new(x as f32, y as f32);
                }
                WindowEvent::Close => {
                    println!("close event");
                }
                _ => {}
            }
        }

        self.cpu.cycle();
    }
}

impl<R: 'static + Rand> State for GLState<BoolDisplay, R> {
    fn step(&mut self, window: &mut Window) {
        self.common_step(window);

        //let mut display = String::new();
        //println!("starting draw");
        for (y, row) in self.cpu.display.get_buffer().iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                if *val {
                    //println!("drawing point ({}, {})", j, i);
                    //display.push_str("*");
                    let d = Point3::new(x as f32, -(y as f32), 0.0);
                    window.draw_point(&d, &self.color);
                //rect.s
                } else {
                    //display.push_str(" ");
                }
            }
            //display.push_str("\n");
        }
        //fs::write("/tmp/display", display.as_bytes()).expect("Unable to write display file");
        //println!("finished draw");
    }

    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut Camera>,
        Option<&mut PlanarCamera>,
        Option<&mut Renderer>,
        Option<&mut PostProcessingEffect>,
    ) {
        (Some(&mut self.camera), None, None, None)
    }
}

impl<R: 'static + Rand> State for GLState<BlockDisplay, R> {
    fn step(&mut self, window: &mut Window) {
        self.common_step(window);
    }

    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut Camera>,
        Option<&mut PlanarCamera>,
        Option<&mut Renderer>,
        Option<&mut PostProcessingEffect>,
    ) {
        (Some(&mut self.camera), None, None, None)
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

fn main() {
    //let file_name = env::args().nth(1).expect("Must give game name as first file");
    //let rom = fs::read(file_name).expect("Unable to read rom");
    let rom = PONG;
    //let rom = env::args().nth(1).and_then(|file_name| fs::read(file_name).ok());

    //println!("{:X?}", rom);

    let factor = 20;
    let mut window = Window::new_with_size("oxi8", DISPLAY_WIDTH * factor, DISPLAY_HEIGHT * factor);

    //window.size()

    //window.set_light(Light::StickToCamera);
    //window.set_point_size(32.0);
    //window.

    //let mut camera = kiss3d::planar_camera::FixedView::new();
    //while window.render_with(None, Some(&mut camera), None) {

    //camera.set_yaw(180.0);
    //camera.set_pitch(180.0);
    //camera.set_dist(20.0);
    //camera.set_rotate_modifiers()
    //camera.set_up_axis(Vector3::z());

    // start BlockDisplay Config

    window.set_light(Light::StickToCamera);

    let center_x = DISPLAY_WIDTH as f32 * 2.5;
    let center_y = -(DISPLAY_HEIGHT as f32 * 2.5);
    let zoom = DISPLAY_HEIGHT as f32 * 6.4;

    //let center_x = 64.0;

    let eye = Point3::new(center_x, center_y, zoom);
    let at = Point3::new(center_x, center_y, 0.0);
    let camera = ArcBall::new(eye, at);

    let display = BlockDisplay::new(&mut window);
    // stop BlockDisplay Config

    // start BoolDisplay Config
    /*
    let center_x = (DISPLAY_WIDTH / 2) as f32;
    let center_y = -((DISPLAY_HEIGHT / 2) as f32);
    let zoom = 40.0;
    let eye = Point3::new(center_x, center_y, zoom);
    let at = Point3::new(center_x, center_y, 0.0);
    let mut camera = ArcBall::new(eye, at);

    let display = BoolDisplay::new();
    */
    // stop BoolDisplay Config

    let mut keymap = HashMap::new();
    /*
    1234  --->  123C
    QWER  --->  456D
    ASDF  --->  789E
    ZXCV  --->  A0BF
    */
    keymap.insert(KissKey::Key1, Key::K1);
    keymap.insert(KissKey::Key2, Key::K2);
    keymap.insert(KissKey::Key3, Key::K3);
    keymap.insert(KissKey::Q, Key::K4);
    keymap.insert(KissKey::W, Key::K5);
    keymap.insert(KissKey::E, Key::K6);
    keymap.insert(KissKey::A, Key::K7);
    keymap.insert(KissKey::S, Key::K8);
    keymap.insert(KissKey::D, Key::K9);
    keymap.insert(KissKey::X, Key::K0);
    keymap.insert(KissKey::Z, Key::KA);
    keymap.insert(KissKey::C, Key::KB);
    keymap.insert(KissKey::Key4, Key::KC);
    keymap.insert(KissKey::R, Key::KD);
    keymap.insert(KissKey::F, Key::KE);
    keymap.insert(KissKey::V, Key::KF);

    let state = GLState {
        cpu: Cpu::new(&rom, display, ThreadRand::new()),
        //cpu: Cpu::new(&rom, display, oxi8_cpu::ConstantRand{}),
        color: Point3::new(1.0, 1.0, 1.0),
        camera,
        keymap,
    };

    window.render_loop(state)
}
