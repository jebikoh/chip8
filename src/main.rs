extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;

use std::fs::File;
use std::io::Read;

const RAM_SIZE: usize       = 4096;
const STACK_SIZE: usize     = 16;
const DISPLAY_WIDTH: usize  = 64;
const DISPLAY_HEIGHT: usize = 32;

const START_ADDR: usize     = 0x200;

const FONT_ADDR: usize      = 0x050;
const FONT_SIZE: usize      = 80;
const FONT_SET: [u8; 80]    = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

// Controls behavior of 8XY6 and 8XYE opcodes
// If set to true, the value of register Y is copied to register X before the operation
const SHIFT_QUIRK: bool = false;
// Controls the behavior of BNNN opcode
// If set to true, it will function as BXNN 
const JUMP_QUIRK: bool = false;

struct Stack {
    stack: [u16; STACK_SIZE],
    stack_ptr: u16,
}

impl Stack {
    fn new() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            stack_ptr: 0
        }
    }

    fn push(&mut self, addr: u16) {
        self.stack[self.stack_ptr as usize] = addr;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_ptr -= 1;
        return self.stack[self.stack_ptr as usize];
    }
}

struct Chip8 {
    display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    keypad: [bool; 16],
    ram: [u8; RAM_SIZE],
    stack: Stack,
    pc: u16,
    i_reg: u16, 
    v_reg: [u8; 16],
    d_timer: u8,
    s_timer: u8,
}

impl Chip8 {
    fn new() -> Self {
        let mut new_chip8: Chip8 = Self {
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            keypad: [false; 16],
            ram: [0; RAM_SIZE],
            stack: Stack::new(),
            pc: START_ADDR as u16,
            i_reg: 0,
            v_reg: [0; 16],
            d_timer: 0,
            s_timer: 0
        };

        // Load font set into memory
        new_chip8.ram[FONT_ADDR..FONT_ADDR + FONT_SIZE].copy_from_slice(&FONT_SET);
        return new_chip8;
    }

    fn fetch(&mut self) -> u16 {
        // Fetch instruction from memory
        let opcode = (self.ram[self.pc as usize] as u16) << 8 | self.ram[self.pc as usize + 1] as u16;
        self.pc += 2;
        return opcode;
    }

    #[allow(unused_parens)]
    fn execute(&mut self, opcode: u16) {
        let d1 = (opcode & 0xF000) >> 12;
        let d2 = (opcode & 0x0F00) >> 8;
        let d3 = (opcode & 0x00F0) >> 4;
        let d4 = (opcode & 0x000F);

        match (d1, d2, d3, d4) {
            // 00E0
            (0, 0, 0xE, 0) => {
                self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
            },
            // 1NNN
            (1, _, _, _) => {
                self.pc = opcode & 0x0FFF;
            },
            // 2NNN
            (2, _, _, _) => {
                self.stack.push(self.pc);
                self.pc = opcode & 0x0FFF;
            },
            // 00EE
            (0, 0, 0xE, 0xE) => {
                self.pc = self.stack.pop();
            },
            // 3XNN
            (3, _, _, _) => {
                if self.v_reg[d2 as usize] == ((opcode & 0xFF) as u8) {
                    self.pc += 2;
                }
            },
            // 4XNN
            (4, _, _, _) => {
                if self.v_reg[d2 as usize] != ((opcode & 0xFF) as u8) {
                    self.pc += 2;
                }
            },
            // 5XY0
            (5, _, _, 0) => {
                if self.v_reg[d2 as usize] == self.v_reg[d3 as usize] {
                    self.pc += 2;
                }
            },
            // 9XY0
            (9, _, _, 0) => {
                if self.v_reg[d2 as usize] != self.v_reg[d3 as usize] {
                    self.pc += 2;
                }
            },
            // 6XNN
            (6, _, _, _) => {
                self.v_reg[d2 as usize] = (opcode & 0xFF) as u8;
            },
            // 7XNN
            (7, _, _, _) => {
                let reg = d2 as usize;
                self.v_reg[reg] = self.v_reg[reg].wrapping_add((opcode & 0xFF) as u8);
            },
            (8, _, _, 0) => {
                self.v_reg[d2 as usize] = self.v_reg[d3 as usize];
            },
            (8, _, _, 1) => {
                self.v_reg[d2 as usize] |= self.v_reg[d3 as usize];
            },
            (8, _, _, 2) => {
                self.v_reg[d2 as usize] &= self.v_reg[d3 as usize];
            },
            (8, _, _, 3) => {
                self.v_reg[d2 as usize] ^= self.v_reg[d3 as usize];
            },
            (8, _, _, 4) => {
                let (vx, flag) = self.v_reg[d2 as usize].overflowing_add(self.v_reg[d3 as usize]);
                self.v_reg[d2 as usize] = vx;
                self.v_reg[0xF as usize] = if flag { 1 } else { 0 };
            }
            (8, _, _, 5) => {
                let (vx, flag) = self.v_reg[d2 as usize].overflowing_sub(self.v_reg[d3 as usize]);
                self.v_reg[d2 as usize] = vx;
                self.v_reg[0xF as usize] = if flag { 0 } else { 1 };
            },
            (8, _, _, 7) => {
                let (vx, flag) = self.v_reg[d3 as usize].overflowing_sub(self.v_reg[d2 as usize]);
                self.v_reg[d2 as usize] = vx;
                self.v_reg[0xF as usize] = if flag { 0 } else { 1 };
            },
            (8, _, _, 6) => {
                let x = d2 as usize;
                if SHIFT_QUIRK {
                    self.v_reg[x] = self.v_reg[d3 as usize];
                }
                self.v_reg[0xF as usize] = self.v_reg[x] & 0x1;
                self.v_reg[x] >>= 1;
            },
            (8, _, _, 0xE) => {
                let x = d2 as usize;
                if SHIFT_QUIRK {
                    self.v_reg[x] = self.v_reg[d3 as usize];
                }
                self.v_reg[0xF as usize] = (self.v_reg[x] >> 7) & 0x1;
                self.v_reg[x] <<= 1;
            },
            (0xA, _, _, _) => {
                self.i_reg = opcode & 0x0FFF;
            },
            (0xB, _, _, _) => {
                let offset = if JUMP_QUIRK { self.v_reg[d2 as usize] as u16 } else { self.v_reg[0] as u16 };
                self.pc = offset + (opcode & 0x0FFF);
            },
            (0xC, _, _, _) => {
                let r: u8 = rand::random();
                self.v_reg[d2 as usize] = r & ((opcode & 0x00FF) as u8);
            },
            (0xD, _, _, _) => {
                // Display!
                let x = self.v_reg[d2 as usize] % (DISPLAY_WIDTH as u8);
                let y = self.v_reg[d3 as usize] % (DISPLAY_HEIGHT as u8);
                self.v_reg[0xF] = 0;
            
                for row in 0..d4 {
                    let sprite = self.ram[(self.i_reg + (row as u16)) as usize];
                    // Check if row is out of bounds
                    if y + row as u8 >= DISPLAY_HEIGHT as u8 {
                        break;
                    }
                    for col in 0..8 {
                        // Check if column is out of bounds
                        if x + col as u8 >= DISPLAY_WIDTH as u8 {
                            break;
                        }
                        // Check if the pixel is set on the sprite
                        if (sprite & (0x80 >> col)) != 0 {
                            let idx = (y + row as u8) as usize * DISPLAY_WIDTH + (x + col) as usize;
                            // Flip the bit if it's off; otherwise set the flag
                            if self.display[idx] {
                                self.v_reg[0xF] = 1;
                            }
                            self.display[idx] ^= true;
                        }
                    }

                }
            },
            (0xE, _, 9, 0xE) => {
                if self.keypad[self.v_reg[d2 as usize] as usize] { self.pc += 2; }
            },
            (0xE, _, 0xA, 1) => {
                if !self.keypad[self.v_reg[d2 as usize] as usize] { self.pc += 2; }
            },
            (0xF, _, 0, 7) => {
                self.v_reg[d2 as usize] = self.d_timer;
            },
            (0xF, _, 1, 5) => {
                self.d_timer = self.v_reg[d2 as usize];
            },
            (0xF, _, 1, 8) => {
                self.s_timer = self.v_reg[d2 as usize];
            },
            (0xF, _, 1, 0xE) => {
                let (i, flag) = self.i_reg.overflowing_add(self.v_reg[d2 as usize] as u16);
                self.i_reg = i;
                self.v_reg[0xF] = if flag && i > 0xFFF { 1 } else { 0 };
            },
            (0xF, _, 0, 0xA) => {
                let mut key_pressed = false;
                for i in 0..16 {
                    if self.keypad[i] {
                        self.v_reg[d2 as usize] = i as u8;
                        key_pressed = true;
                        break;
                    }
                }

                if !key_pressed {
                    self.pc -= 2;
                }
            },
            (0xF, _, 2, 9) => {
                self.i_reg = FONT_ADDR as u16 + (self.v_reg[d2 as usize] as u16) * 5;
            },
            (0xF, _, 3, 3) => {
                let num = self.v_reg[d2 as usize];
                self.ram[self.i_reg as usize] = num % 10;
                self.ram[self.i_reg as usize + 1] = (num / 10) % 10;
                self.ram[self.i_reg as usize + 2] = num / 100;
            },
            (0xF, _, 5, 5) => {
                for i in 0..=d2 {
                    self.ram[(self.i_reg + i as u16) as usize] = self.v_reg[i as usize];
                }
            },
            (0xF, _, 6, 5) => {
                for i in 0..=d2 {
                    self.v_reg[i as usize] = self.ram[(self.i_reg + i as u16) as usize];
                }
            }
            _ => println!("Unknown opcode: {:#X}", opcode)
        }
    }

    fn cycle(&mut self) {
        let opcode = self.fetch();
        self.execute(opcode);
    }

    fn timer_tick(&mut self) {
        if self.d_timer > 0 {
            self.d_timer -= 1;
        }
        if self.s_timer > 0 {
            self.s_timer -= 1;
        }
    }

    fn load_rom(&mut self, rom: Vec<u8>) {
        self.ram[START_ADDR..START_ADDR + rom.len()].copy_from_slice(&rom);
    }
}

fn read_rom(file_path: &str) -> Vec<u8> {
    let mut f = File::open(file_path).expect("File not found");
    let mut rom = Vec::new();
    f.read_to_end(&mut rom).expect("Failed to read file");
    return rom;
}


const CYCLES_PER_FRAME: usize = 12;
const DISPLAY_SCALE:u32     = 10;

fn key_to_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q    => Some(0x4),
        Keycode::W    => Some(0x5),
        Keycode::E    => Some(0x6),
        Keycode::R    => Some(0xD),
        Keycode::A    => Some(0x7),
        Keycode::S    => Some(0x8),
        Keycode::D    => Some(0x9),
        Keycode::F    => Some(0xE),
        Keycode::Z    => Some(0xA),
        Keycode::X    => Some(0x0),
        Keycode::C    => Some(0xB),
        Keycode::V    => Some(0xF),
        _ => None
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <rom>", args[0]);
        return;
    }

    let rom = read_rom(&args[1]);

    let mut chip8 = Chip8::new();
    chip8.load_rom(rom);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Chip8", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                },
                Event::KeyDown {keycode: Some(key), ..} => {
                    if let Some(button) = key_to_button(key) {
                        chip8.keypad[button] = true;
                    }
                },
                Event::KeyUp {keycode: Some(key), ..} => {
                    if let Some(button) = key_to_button(key) {
                        chip8.keypad[button] = false;
                    }
                }
                _ => {}
            }
        }

        for _ in 0..CYCLES_PER_FRAME {
            chip8.cycle();
        }
        chip8.timer_tick();
        
        // Draw screen
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.set_draw_color(Color::WHITE);
        let screen_buf = chip8.display;
        for (i, pixel) in screen_buf.iter().enumerate() {
            if *pixel {
                let x = (i % DISPLAY_WIDTH) as u32;
                let y = (i / DISPLAY_WIDTH) as u32;
                canvas.fill_rect(Rect::new((x * DISPLAY_SCALE) as i32, (y * DISPLAY_SCALE) as i32, DISPLAY_SCALE, DISPLAY_SCALE)).unwrap();
            }
        }
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
