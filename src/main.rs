type Byte = u8;

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
            ram: [0; RAM_SIZE],
            stack: Stack::new(),
            pc: 0,
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
                if (self.v_reg[d2 as usize] == (opcode & 0xFF) as u8) {
                    self.pc += 2;
                }
            },
            // 4XNN
            (4, _, _, _) => {
                if (self.v_reg[d2 as usize] != (opcode & 0xFF) as u8) {
                    self.pc += 2;
                }
            },
            // 5XY0
            (5, _, _, 0) => {
                if (self.v_reg[d2 as usize] == self.v_reg[d3 as usize]) {
                    self.pc += 2;
                }
            },
            // 9XY0
            (9, _, _, 0) => {
                if (self.v_reg[d2 as usize] != self.v_reg[d3 as usize]) {
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
                self.v_reg[reg] = self.v_reg[reg] + (opcode & 0xFF) as u8;
            }
            _ => println!("Unknown opcode: {:#X}", opcode)
        }
    }
}

fn main() {
    println!("Hello, world!");
}
