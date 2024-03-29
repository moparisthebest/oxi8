use core::{fmt, slice::Iter};

#[cfg(target_arch = "wasm32")]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.0

const RAM_SIZE: usize = 4096; // 0-511 reserved for interpreter, useless today
const PROGRAM_OFFSET: usize = 512; // can be 1536 for ETI 660 programs
const NUM_REGISTERS: usize = 16;
const NUM_FLAG_REGISTERS: usize = 8; // for SCHIP only
const STACK_SIZE: usize = 16; // maximum size of the stack, 12 for CHIP-8, 16 for SCHIP

// these should be config options, standard is 64x32, 128x64 is also common
// ETI 660 supported 64x48 and 64x64...
pub const DISPLAY_WIDTH: u32 = 64;
pub const DISPLAY_HEIGHT: u32 = 32;

const SMALL_SPRITE_LEN: usize = 80;
const SPRITE_LEN: usize = SMALL_SPRITE_LEN + SMALL_SPRITE_LEN * 2; // regular font size + schip font size
const SPRITES: [u8; SPRITE_LEN] = [
    // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#font
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    // double size SCHIP sprites, starting at index 80 here
    0xFF, 0xFF, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, // 0
    0x18, 0x78, 0x78, 0x18, 0x18, 0x18, 0x18, 0x18, 0xFF, 0xFF, // 1
    0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // 2
    0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 3
    0xC3, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0x03, 0x03, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 5
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 6
    0xFF, 0xFF, 0x03, 0x03, 0x06, 0x0C, 0x18, 0x18, 0x18, 0x18, // 7
    0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, // 8
    0xFF, 0xFF, 0xC3, 0xC3, 0xFF, 0xFF, 0x03, 0x03, 0xFF, 0xFF, // 9
    // *technically* SCHIP doesn't include A-F
    // https://github.com/Chromatophore/HP48-Superchip/blob/master/investigations/quirk_font.md
    // but it doesn't *hurt* anything in case a game wants to use them...
    0x7E, 0xFF, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xC3, // A
    0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, // B
    0x3C, 0xFF, 0xC3, 0xC0, 0xC0, 0xC0, 0xC0, 0xC3, 0xFF, 0x3C, // C
    0xFC, 0xFE, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFE, 0xFC, // D
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // E
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xC0, 0xC0, // F
];

const NUM_KEYS: usize = 16;

#[derive(Clone, Copy)]
pub enum Key {
    K0 = 0x0,
    K1 = 0x1,
    K2 = 0x2,
    K3 = 0x3,
    K4 = 0x4,
    K5 = 0x5,
    K6 = 0x6,
    K7 = 0x7,
    K8 = 0x8,
    K9 = 0x9,
    KA = 0xA,
    KB = 0xB,
    KC = 0xC,
    KD = 0xD,
    KE = 0xE,
    KF = 0xF,
}

pub struct Keyboard {
    keys: [bool; NUM_KEYS],
    keywait: KeyWait,
}

#[derive(PartialEq)]
enum KeyWait {
    None,        // nothing to wait for
    Wait,        // wait for a keypress
    Pressed(u8), // a key was pressed
}

impl Keyboard {
    fn key_pressed(&self, keycode: u8) -> bool {
        self.keys[keycode as usize]
    }

    pub fn toggle_key(&mut self, key: Key, pressed: bool) {
        self.keys[key as usize] = pressed;
        if pressed && self.keywait == KeyWait::Wait {
            self.keywait = KeyWait::Pressed(key as u8);
        }
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            keys: [false; NUM_KEYS],
            keywait: KeyWait::None,
        }
    }
}

// chip-8 500hz schip 1000hz https://github.com/AfBu/haxe-chip-8-emulator/wiki/(Super)CHIP-8-Secrets
const CLOCK_RATE_HZ: u32 = 500;

const DELAY_DECREMENT_HZ: u32 = 60; // also used for sound register

const NANOS_PER_SEC: u32 = 1_000_000_000;

struct Timer {
    cycle_every_nanos: u128,
    last_cycle_timestamp: u128,
}

impl Timer {
    fn new(rate_hz: u32) -> Timer {
        Timer {
            cycle_every_nanos: (NANOS_PER_SEC / rate_hz) as u128,
            last_cycle_timestamp: 0,
        }
    }

    fn set_rate_hz(&mut self, rate_hz: u32) {
        self.cycle_every_nanos = (NANOS_PER_SEC / rate_hz) as u128;
    }

    fn num_cycles(&mut self, total_elapsed_nanos: u128) -> std::ops::Range<u128> {
        let nanos_since_last_cycle = total_elapsed_nanos - self.last_cycle_timestamp;

        let num_instructions = nanos_since_last_cycle / self.cycle_every_nanos;

        //println!("num_instructions: {}, nanos_since_last_cycle: {}", num_instructions, nanos_since_last_cycle);

        if num_instructions > 0 {
            self.last_cycle_timestamp = total_elapsed_nanos;
            0..num_instructions
        } else {
            0..0
        }
    }

    /*
    // todo: it'd be nice if ownership could make this work...
    fn execute_cycles(&mut self, total_elapsed_nanos: u128, mut f: impl FnMut()) {
        for x in self.num_cycles(total_elapsed_nanos) {
            //println!("running x: {}", x);
            f();
        }
    }
    */
}

// chrono crate doesn't support wasm32 arch yet, workaround
#[cfg(target_arch = "wasm32")]
struct Instant {
    start_time: u64,
}

#[cfg(target_arch = "wasm32")]
impl Instant {
    pub fn now() -> Instant {
        Instant {
            start_time: Instant::millis_since_epoch(),
        }
    }

    pub fn millis_since_epoch() -> u64 {
        stdweb::web::Date::now() as u64
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_millis(Instant::millis_since_epoch() - self.start_time)
    }
}

pub trait Rand {
    fn next(&mut self) -> u8 {
        4 // chosen by fair dice roll. guaranteed to be random.
    }
}

pub struct ConstantRand {}
impl Rand for ConstantRand {}

struct Stack {
    stack: [u16; STACK_SIZE],
    sp: usize, // stack pointer
}

impl Stack {
    fn new() -> Stack {
        Stack {
            stack: [0; STACK_SIZE],
            sp: 0,
        }
    }

    fn pop(&mut self) -> Option<u16> {
        if self.sp == 0 {
            None
        } else {
            self.sp -= 1;
            Some(self.stack[self.sp])
        }
    }

    fn push(&mut self, value: u16) -> Option<()> {
        if self.sp == STACK_SIZE {
            return None;
        }
        self.stack[self.sp] = value;
        self.sp += 1;
        Some(())
    }

    fn clear(&mut self) {
        self.sp = 0;
        // we *could* clear stack here, but don't really *need* to...
        //self.stack = [0; STACK_SIZE];
    }
}

pub struct Cpu<T: Display, R: Rand> {
    // these are accessible by programs
    i: u16,                 // generally used to store memory addresses so only 12 bits used...
    v: [u8; NUM_REGISTERS], // general purpose
    flag: [u8; NUM_FLAG_REGISTERS], // SCHIP 64 one-bit flag registers
    delay: u8,              // when non-zero decremented at 60hz
    pub sound: u8,          // when non-zero decremented at 60hz and sound buzzer
    ram: [u8; RAM_SIZE],

    // these are used by the emulator
    pc: u16, // program counter
    stack: Stack,
    pub display: T,
    pub keyboard: Keyboard,
    start_time: Instant,
    clock_rate_hz: u32,
    cpu_timer: Timer,
    delay_timer: Timer,
    num_instructions_per_decrement: u32,
    rand: R,
}

impl<T: Display, R: Rand> fmt::Debug for Cpu<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cpu {{ I: {:04X?}, V: {:02X?}", self.i, self.v)?;
        //write!(f, ", delay: {}, sound: {}", self.delay, self.sound)?;
        write!(f, " }}")
    }
}

impl<T: Display, R: Rand> Cpu<T, R> {
    pub fn new(rom: &[u8], display: T, rand: R) -> Cpu<T, R> {
        let mut ram = [0; RAM_SIZE];
        // sprites go up front
        ram[0..SPRITE_LEN].copy_from_slice(&SPRITES);
        // rom goes to PROGRAM_OFFSET
        ram[PROGRAM_OFFSET..(PROGRAM_OFFSET + rom.len())].copy_from_slice(rom);

        // for handy inspection with xxd
        //fs::write("/tmp/ram.debug", &ram[0..RAM_SIZE]).expect("Unable to write file");

        Cpu {
            i: 0,
            v: [0; NUM_REGISTERS],
            flag: [0; NUM_FLAG_REGISTERS], // todo: provide a way to save/load these?
            delay: 0,
            sound: 0,
            ram,
            pc: PROGRAM_OFFSET as u16,
            stack: Stack::new(),
            display,
            keyboard: Keyboard::default(),
            start_time: Instant::now(),
            clock_rate_hz: CLOCK_RATE_HZ,
            cpu_timer: Timer::new(CLOCK_RATE_HZ),
            delay_timer: Timer::new(DELAY_DECREMENT_HZ),
            num_instructions_per_decrement: CLOCK_RATE_HZ / DELAY_DECREMENT_HZ,
            rand,
        }
    }

    pub fn get_clock_rate_hz(&self) -> u32 {
        self.clock_rate_hz
    }

    pub fn set_clock_rate_hz(&mut self, rate_hz: u32) {
        self.clock_rate_hz = rate_hz;
        self.num_instructions_per_decrement = rate_hz / DELAY_DECREMENT_HZ;
        self.cpu_timer.set_rate_hz(rate_hz);
    }

    pub fn inc_clock_rate_hz(&mut self, amount: i32) {
        let new_rate = (self.clock_rate_hz as i32).wrapping_add(amount);
        // this does cap max at i32 instead of u32 but I don't care...
        if new_rate >= DELAY_DECREMENT_HZ as i32 {
            self.set_clock_rate_hz(new_rate as u32);
        }
    }

    // this can be called at any rate, and runs the *correct* number of cycles and timer decrements
    // that should have been ran since the last time this was called
    pub fn cycle(&mut self) {
        let total_elapsed_nanos = self.start_time.elapsed().as_nanos();

        for _ in self.delay_timer.num_cycles(total_elapsed_nanos) {
            //println!("running x: {}", x);
            self.decrement_timers();
        }

        // bummer: self.delay_timer.execute_cycles(total_elapsed_nanos, ||self.decrement_timers());

        for _ in self.cpu_timer.num_cycles(total_elapsed_nanos) {
            //println!("running x: {}", x);
            self.execute_next_instruction();
        }
    }

    // this MUST be called at exactly 60hz, 60 times per second
    pub fn cycle_60hz(&mut self) {
        self.decrement_timers();

        for _ in 0..self.num_instructions_per_decrement {
            self.execute_next_instruction();
        }
    }

    pub fn decrement_timers(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound > 0 {
            //println!("**BEEP**");
            self.sound -= 1;
        }
    }

    #[inline(always)]
    pub fn next_instruction(&self) -> Instruction {
        Instruction {
            wx: self.ram.g(self.pc),
            yz: self.ram.g(self.pc + 1),
        }
    }

    pub fn execute_next_instruction(&mut self) {
        let instruction = self.next_instruction();
        //println!("ins: {}", instruction);
        //print!("ins: {}, before: {:?}", instruction, self);
        self.pc = self.execute_instruction(instruction);
        //println!(", after : {:?}", self);
    }

    pub fn reset(&mut self) {
        self.i = 0;
        //self.v.iter_mut().for_each(|x| *x = 0);
        self.v = [0; NUM_REGISTERS];
        self.delay = 0;
        self.sound = 0;
        // *technically* the rom can modify itself in ram, but we are going to ignore that for now
        self.pc = PROGRAM_OFFSET as u16;
        self.stack.clear();
        self.display.set_hires(false);
        self.keyboard.keywait = KeyWait::None;
        // probably don't *need* to reset these timers...
        self.start_time = Instant::now();
        self.cpu_timer.last_cycle_timestamp = 0;
        self.delay_timer.last_cycle_timestamp = 0;
    }

    // executes 1 instruction and returns updated program_counter
    pub fn execute_instruction(&mut self, i: Instruction) -> u16 {
        match i.w() {
            0x0 => match i.xyz() {
                // 0000 - Octo - FREEZE
                // https://github.com/JohnEarnest/Octo/blob/gh-pages/docs/SuperChip.md
                0x000 => panic!("0000 - Octo - FREEZE instruction!"), // todo: take callback function for this
                // 00E0 - CLS: clear display
                0x0E0 => {
                    self.display.clear();
                    self.next()
                }
                // 00FB - SCHIP - SCROLL_RIGHT: Scroll the display right by 4 pixels in hires, 2 in low
                0x0FB => {
                    self.display.scroll_right();
                    self.next()
                }
                // 00FC - SCHIP - SCROLL_LEFT: Scroll the display left by 4 pixels in hires, 2 in low
                0x0FC => {
                    self.display.scroll_left();
                    self.next()
                }
                // 00FD - SCHIP - EXIT: Exit the emulator
                0x0FD => panic!("00FD - SCHIP - EXIT instruction!"), // todo: take callback function for this
                // 00FE - SCHIP - LORES: Disable high resolution graphics mode and return to 64x32
                0x0FE => {
                    self.display.set_hires(false);
                    self.next()
                }
                // 00FF - SCHIP - HIRES: Enable 128x64 high resolution graphics mode
                0x0FF => {
                    self.display.set_hires(true);
                    self.next()
                }
                // 00EE - RET: return from subroutine
                0x0EE => self.stack.pop().expect("returning with no value on stack?") + 2,
                // 0xyz - SYS addr: Jump to a machine code routine at nnn. Ignored by interpreters
                _ => match i.xy() {
                    // 00Cz - SCHIP - SCROLL_DOWN: Scroll the display down by z pixels in hires, z/2 in low
                    0x0C => {
                        self.display.scroll_down(i.z());
                        self.next()
                    }
                    _ => self.bad(i),
                },
            },
            // 1xyz - JP addr: Jump to location xyz
            0x1 => i.xyz(),
            // 2xyz - CALL addr: Call subroutine at xyz
            0x2 => {
                self.stack
                    .push(self.pc)
                    .expect("exceeded maximum stack size");
                i.xyz()
            }
            // 3xyz - SE Vx, yz: Skip next instruction if Vx = yz
            0x3 => self.skip_if(self.v.g(i.x()) == i.yz()),
            // 4xyz - SNE Vx, yz: Skip next instruction if Vx != yz
            0x4 => self.skip_if(self.v.g(i.x()) != i.yz()),
            // 5xy0 - SE Vx, Vy: Skip next instruction if Vx = Vy
            0x5 => match i.z() {
                // do we REALLY need to check that last nibble (z) is 0 here?
                0 => self.skip_if(self.v.g(i.x()) == self.v.g(i.y())),
                _ => self.bad(i),
            },
            // 6xyz - LD Vx, yz: Set Vx = yz
            0x6 => {
                self.v.s(i.x(), i.yz());
                self.next()
            }
            // 7xyz - ADD Vx, yz: Set Vx = Vx + yz
            0x7 => {
                // OVERFLOW:
                // *self.v.i(i.x()) += i.yz();
                let vx = self.v.i(i.x());
                *vx = (*vx).wrapping_add(i.yz());
                self.next()
            }
            0x8 => {
                let vy = self.v.g(i.y()); // need this for every one of these except 8xy6, I'm ok with the perf hit for that one :)
                let x = i.x();
                let vx = self.v.g(x);
                let vx = match i.z() {
                    // 8xy0 - LD Vx, Vy: Set Vx = Vy
                    0x0 => vy,
                    // 8xy1 - OR Vx, Vy: Set Vx = Vx OR Vy
                    0x1 => vx | vy,
                    // 8xy2 - AND Vx, Vy: Set Vx = Vx AND Vy
                    0x2 => vx & vy,
                    // 8xy3 - XOR Vx, Vy: Set Vx = Vx XOR Vy
                    0x3 => vx ^ vy,
                    // 8xy4 - ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry.
                    0x4 => {
                        let (vx, overflowed) = vx.overflowing_add(vy);
                        self.v[0xF] = if overflowed { 1 } else { 0 };
                        vx
                    }
                    // 8xy5 - SUB Vx, Vy: If Vx > Vy, then VF is set to 1, otherwise 0, then Set Vx = Vx - Vy
                    0x5 => {
                        self.v[0xF] = if vx > vy { 1 } else { 0 };
                        // OVERFLOW:
                        // vx - vy
                        vx.wrapping_sub(vy)
                    }
                    // 8xy6 - SHR Vx {, Vy}: Set Vx = Vx SHR 1. If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0, then Set Vx = Vx / 2
                    0x6 => {
                        self.v[0xF] = vx & 0b1;
                        // OVERFLOW:
                        // vx / 2
                        vx.wrapping_div(2)
                    }
                    // 8xy7 - SUBN Vx, Vy: If Vy > Vx, then VF is set to 1, otherwise 0, then Set Vx = Vy - Vx.
                    0x7 => {
                        self.v[0xF] = if vy > vx { 1 } else { 0 };
                        // OVERFLOW:
                        // vy - vx
                        vy.wrapping_sub(vx)
                    }
                    // 8xyE - SHL Vx {, Vy}, Set Vx = Vx SHL 1. If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0, then Set Vx = Vx * 2
                    0xE => {
                        self.v[0xF] = vx & 0b10000000;
                        // OVERFLOW:
                        // vx * 2
                        vx.wrapping_mul(2)
                    }
                    _ => {
                        panic!("bad instruction: {:?}", i);
                        //vx
                    }
                };
                self.v.s(x, vx);
                self.next()
            }
            // 9xy0 - SNE Vx, Vy: Skip next instruction if Vx != Vy
            0x9 => match i.z() {
                // do we REALLY need to check that last nibble (z) is 0 here?
                0 => self.skip_if(self.v.g(i.x()) != self.v.g(i.y())),
                _ => self.bad(i),
            },
            // Axyz - LD I, xyz: Set I = xyz
            0xA => {
                self.i = i.xyz();
                self.next()
            }
            // Bxyz - JP V0, xyz: Jump to location V0 + xyz
            0xB => self.v[0] as u16 + i.xyz(),
            // Cxyz - RND Vx, yz: Set Vx = random byte (0-255) AND yz
            0xC => {
                let rand = self.rand.next();
                self.v.s(i.x(), rand & i.yz());
                self.next()
            }
            // Dxyz - DRW Vx, Vy, z: Display z-length-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            // Dxy0 - SCHIP - DRW Vx, Vy, 0: Display a special SCHIP 16x16 sprite at memory location I at (Vx, Vy), set VF = collision
            0xD => {
                let vx = self.v.g(i.x()) as usize;
                let vy = self.v.g(i.y()) as usize;
                let z = i.z() as usize;
                let from = self.i as usize;
                self.v[0xF] = if z == 0 {
                    // Draw SCHIP 16x16 sprite
                    let to = from + 32;
                    self.display.schip_draw(vx, vy, &self.ram[from..to])
                } else {
                    // Draw standard chip8 sprite
                    let to = from + z;
                    self.display.draw(vx, vy, &self.ram[from..to])
                } as u8;
                self.next()
            }
            0xE => match i.yz() {
                // Ex9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed.
                0x9E => self.skip_if(self.keyboard.key_pressed(self.v.g(i.x()))),
                // ExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed.
                0xA1 => self.skip_if(!self.keyboard.key_pressed(self.v.g(i.x()))),
                _ => self.bad(i),
            },
            0xF => {
                match i.yz() {
                    // Fx07 - LD Vx, DT: Set Vx = delay timer value
                    0x07 => self.v.s(i.x(), self.delay),
                    // Fx0A - LD Vx, K: Wait for a key press, store the value of the key in Vx
                    0x0A => {
                        // this is done a little funky but I'm not sure anything else would be better
                        // basically, until a key is pressed subtract 2 from pc, the below
                        // call to self.next() will advance *back* to this instruction next time
                        // and we will check again, that in practice continues to return to this code
                        // over and over polling if a key is down yet
                        let found = match &self.keyboard.keywait {
                            KeyWait::None => {
                                self.keyboard.keywait = KeyWait::Wait;
                                false
                            }
                            KeyWait::Wait => false,
                            KeyWait::Pressed(key_value) => {
                                // finally! a key pressed
                                self.v.s(i.x(), *key_value);
                                true
                            }
                        };
                        if found {
                            self.keyboard.keywait = KeyWait::None;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    // Fx15 - LD DT, Vx: Set delay timer = Vx
                    0x15 => self.delay = self.v.g(i.x()),
                    // Fx18 - LD ST, Vx: Set sound timer = Vx
                    0x18 => self.sound = self.v.g(i.x()),
                    // Fx1E - ADD I, Vx: Set I = I + Vx
                    0x1E => self.i += self.v.g(i.x()) as u16,
                    // Fx29 - LD F, Vx: Set I = location of sprite for digit Vx
                    0x29 => self.i = (self.v.g(i.x()) * 5) as u16,
                    // Fx30 - LD F, Vx: Set I = location of SCHIP sprite for digit Vx
                    0x30 => self.i = SMALL_SPRITE_LEN as u16 + self.v.g(i.x()) as u16 * 10,
                    // Fx33 - LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2
                    0x33 => {
                        // takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
                        let vx = self.v.g(i.x());
                        self.ram.s(self.i, (vx / 100) % 10);
                        self.ram.s(self.i + 1, (vx / 10) % 10);
                        self.ram.s(self.i + 2, vx % 10);
                    }
                    // Fx55 - LD [I], Vx: Copy the values of registers V0 through Vx into memory, starting at the address in I
                    0x55 => {
                        let x = i.x() as usize + 1;
                        let i = self.i as usize;
                        self.ram[i..(i + x)].copy_from_slice(&self.v[0..x])
                    }
                    // Fx65 - LD Vx, [I]: Copy the values from memory starting at location I into registers V0 through Vx
                    0x65 => {
                        //println!("before Fx65 v: {:02X?}", self.v);
                        let x = i.x() as usize + 1;
                        let i = self.i as usize;
                        self.v[0..x].copy_from_slice(&self.ram[i..(i + x)])
                    }
                    // Fx75 - SCHIP - SAVE_FLAGS: Save v0-vX to flag registers
                    0x75 => {
                        let x = i.x() as usize + 1;
                        if x > NUM_FLAG_REGISTERS {
                            panic!("Fx75 out of bounds x: {}", x - 1);
                        }
                        self.flag[0..x].copy_from_slice(&self.v[0..x])
                    }
                    // Fx85 - SCHIP - LOAD_FLAGS: Restore v0-vX from flag registers
                    0x85 => {
                        let x = i.x() as usize + 1;
                        if x > NUM_FLAG_REGISTERS {
                            panic!("Fx85 out of bounds x: {}", x - 1);
                        }
                        self.v[0..x].copy_from_slice(&self.flag[0..x])
                    }
                    _ => {
                        self.bad(i);
                    }
                }
                self.next()
            }
            _ => self.bad(i),
        }
    }

    fn bad(&self, i: Instruction) -> u16 {
        panic!("bad instruction: {:?}", i);
        //PROGRAM_OFFSET as u16
    }

    #[inline(always)]
    fn next(&self) -> u16 {
        self.pc + 2
    }

    #[inline(always)]
    fn skip_if(&self, skip: bool) -> u16 {
        self.pc + if skip { 4 } else { 2 }
    }
}

trait Indexable<I, T> {
    // should be able to use i() for everything but borrow checker too dumb:
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=bc29610bfd8079be5b1c85e639bfadf1
    // index to pointer
    fn i(&mut self, index: I) -> &mut T;
    // get for immutable borrow
    fn g(&self, index: I) -> T;
    // set
    fn s(&mut self, index: I, value: T);
}

impl Indexable<u8, u8> for [u8] {
    #[inline(always)]
    fn i(&mut self, index: u8) -> &mut u8 {
        &mut self[index as usize]
    }
    #[inline(always)]
    fn g(&self, index: u8) -> u8 {
        self[index as usize]
    }
    #[inline(always)]
    fn s(&mut self, index: u8, value: u8) {
        self[index as usize] = value;
    }
}

impl Indexable<u16, u8> for [u8] {
    #[inline(always)]
    fn i(&mut self, index: u16) -> &mut u8 {
        &mut self[index as usize]
    }
    #[inline(always)]
    fn g(&self, index: u16) -> u8 {
        self[index as usize]
    }
    #[inline(always)]
    fn s(&mut self, index: u16, value: u8) {
        self[index as usize] = value;
    }
}

impl Indexable<u8, u16> for [u16] {
    #[inline(always)]
    fn i(&mut self, index: u8) -> &mut u16 {
        &mut self[index as usize]
    }
    #[inline(always)]
    fn g(&self, index: u8) -> u16 {
        self[index as usize]
    }
    #[inline(always)]
    fn s(&mut self, index: u8, value: u16) {
        self[index as usize] = value;
    }
}

trait Nibble {
    fn high(&self) -> u8;
    fn low(&self) -> u8;
}

impl Nibble for u8 {
    #[inline(always)]
    fn high(&self) -> u8 {
        (self >> 4) & 0xF_u8
    }

    #[inline(always)]
    fn low(&self) -> u8 {
        self & 0xF
    }
}

// A chip-8 instruction can be thought of as 4 4-bit nibbles
// here I name them in order wxyz, where wx is high, and yz is low
pub struct Instruction {
    wx: u8,
    yz: u8,
}

impl Instruction {
    #[inline(always)]
    fn w(&self) -> u8 {
        self.wx.high()
    }

    #[inline(always)]
    fn x(&self) -> u8 {
        self.wx.low()
    }

    #[inline(always)]
    fn y(&self) -> u8 {
        self.yz.high()
    }

    #[inline(always)]
    fn z(&self) -> u8 {
        self.yz.low()
    }

    #[inline(always)]
    fn wx(&self) -> u8 {
        self.wx
    }

    #[inline(always)]
    fn yz(&self) -> u8 {
        self.yz
    }

    #[inline(always)]
    fn xy(&self) -> u8 {
        self.wx.low() + self.yz.high()
    }

    #[inline(always)]
    fn xyz(&self) -> u16 {
        // we need to extract the low 12 bits
        (((self.wx as u16) << 8) + (self.yz as u16)) & 0xFFF
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i = self;
        write!(f, "{:02X?}{:02X?}", i.wx, i.yz)
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let i = self;
        write!(
            f,
            "Instruction {{ {:X?}{:X?}{:X?}{:X?}, wx: {:02X?}, yz: {:02X?}, xy: {:02X?}, xyz: {:03X?} }}",
            i.w(),
            i.x(),
            i.y(),
            i.z(),
            i.wx(),
            i.yz(),
            i.xy(),
            i.xyz()
        )
    }
}

pub trait Display {
    fn schip_draw(&mut self, starting_x: usize, starting_y: usize, memory: &[u8]) -> bool {
        let mut pixel_turned_off = false;
        let hires = self.hires();
        //println!("hires: {}", hires);
        //let hires = true;
        let hires_starting_x = starting_x + 8;
        let mut y = starting_y as usize % self.height();
        let mut iter = memory.iter();
        // memory *must* be 32 bytes long here, we ensure that at the call site...
        for _ in 1..16 {
            let left = iter.next().unwrap();
            let right = iter.next().unwrap();
            pixel_turned_off |= self.draw_byte(starting_x, y, left);
            if hires {
                pixel_turned_off |= self.draw_byte(hires_starting_x, y, right);
            }
            y = (y + 1) % self.height();
        }
        pixel_turned_off
    }

    fn draw(&mut self, starting_x: usize, starting_y: usize, memory: &[u8]) -> bool {
        let mut pixel_turned_off = false;
        let mut y = starting_y as usize % self.height();
        for byte in memory.iter() {
            pixel_turned_off |= self.draw_byte(starting_x, y, byte);
            y = (y + 1) % self.height();
        }
        pixel_turned_off
    }

    fn draw_byte(&mut self, starting_x: usize, y: usize, byte: &u8) -> bool {
        let mut pixel_turned_off = false;
        for bit_number in 0..8 {
            let x = (starting_x + bit_number) % self.width();

            let current_bit = (byte >> (7 - bit_number)) & 1;

            let current_pixel = self.current_pixel(x, y);
            let new_pixel = current_bit ^ current_pixel;

            self.set_pixel(x, y, new_pixel);

            if current_pixel == 1 && new_pixel == 0 {
                pixel_turned_off = true;
            }
        }
        pixel_turned_off
    }

    fn height(&self) -> usize;
    fn width(&self) -> usize;
    fn current_pixel(&self, x: usize, y: usize) -> u8;
    fn set_pixel(&mut self, x: usize, y: usize, new_pixel: u8);
    fn clear(&mut self);
    fn set_hires(&mut self, on: bool);
    fn hires(&self) -> bool;
    // for all pixel scrolling, number is halved when in lowres
    fn scroll_left(&mut self); // scroll left 4 pixels
    fn scroll_right(&mut self); // scroll right 4 pixels
    fn scroll_down(&mut self, n: u8); // scroll down 0-15 pixels
}

const WIDTH: usize = DISPLAY_WIDTH as usize;
const HEIGHT: usize = DISPLAY_HEIGHT as usize;

pub struct BoolDisplay {
    buffer: Vec<Vec<bool>>,
    scale: u32,
    width: usize,
    height: usize,
    hires: bool,
}

impl BoolDisplay {
    pub fn new(scale: u32) -> BoolDisplay {
        BoolDisplay {
            width: WIDTH,
            height: HEIGHT,
            buffer: vec![vec![false; WIDTH]; HEIGHT],
            scale,
            hires: false,
        }
    }

    pub fn get_buffer(&self) -> Iter<Vec<bool>> {
        self.buffer.iter()
    }

    pub fn get_scale(&self) -> u32 {
        self.scale
    }
}

impl Display for BoolDisplay {
    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn current_pixel(&self, x: usize, y: usize) -> u8 {
        self.buffer[y][x] as u8
    }

    fn set_pixel(&mut self, x: usize, y: usize, new_pixel: u8) {
        self.buffer[y][x] = new_pixel != 0;
    }

    fn clear(&mut self) {
        for row in self.buffer.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }

    fn set_hires(&mut self, on: bool) {
        if self.hires == on {
            //panic!("attempt to set same resolution again..");
            // sweetcopter does this...
            // todo: should we self.clear() here???
            return;
        }
        self.hires = on;
        if on {
            let new_width = WIDTH * 2;
            self.height = HEIGHT * 2;
            self.width = new_width;
            self.buffer.resize(self.height, vec![false; self.width]);
            self.buffer
                .iter_mut()
                .for_each(|row| row.resize(new_width, false));
            self.scale /= 2
        } else {
            self.height = HEIGHT;
            self.width = WIDTH;
            self.buffer.truncate(HEIGHT);
            self.buffer.iter_mut().for_each(|row| row.truncate(WIDTH));
            self.scale *= 2
        }
        self.clear();
    }

    fn hires(&self) -> bool {
        self.hires
    }

    fn scroll_left(&mut self) {
        let pixels = if self.hires { 4 } else { 2 };
        let width = self.width;
        // for each row
        self.buffer.iter_mut().for_each(|row| {
            // delete a number of pixels at the beginning
            row.drain(0..pixels);
            // insert the same number of pixels at the end
            row.resize(width, false);
        });
    }

    fn scroll_right(&mut self) {
        let pixels = if self.hires { 4 } else { 2 };
        let truncate_to = self.width - pixels;
        let prepend: Vec<bool> = vec![false; pixels];
        // for each row
        self.buffer.iter_mut().for_each(|row| {
            // delete a number of pixels at the end
            row.truncate(truncate_to);
            // insert the same number of pixels at the beginning
            row.splice(0..0, prepend.iter().cloned());
        });
    }

    fn scroll_down(&mut self, pixels: u8) {
        let pixels = if self.hires { pixels } else { pixels / 2 } as usize;
        // delete entire rows of pixels at the bottom
        self.buffer.truncate(self.height - pixels);
        // create row we can clone
        let row: Vec<bool> = vec![false; self.width];
        // insert same number of rows of pixels at top
        self.buffer.splice(0..0, (0..pixels).map(|_| row.clone()));
    }
}
