pub mod instruction;
pub mod renderer;

use instruction::ChipInst;
use std::fs::File;
use std::io;
use std::io::Read;

/**
 * Retro-compatibility options
 */
#[derive(Debug)]
struct ChipCfg {
    font_start: u16,       // Starting address of the fonts bytes
    off_jump_legacy: bool, // If true, BNNN will jump to NNN + V0. Else, to NNN + Vx
    reg_save_legacy: bool, // If true, FX55 and FX65 will alter the value of I
    index_add_carry: bool, // If true, carry will be set when I overflows with FX1E
}

impl Default for ChipCfg {
    fn default() -> Self {
        ChipCfg {
            font_start: 0x050,
            off_jump_legacy: false,
            reg_save_legacy: false,
            index_add_carry: false,
        }
    }
}

const DEFAULT_FONT: [u8; 80] = [
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
];

#[derive(Debug)]
pub struct Chip8 {
    i: u16,  // 16-bit index register
    pc: u16, // 16-bit program counter
    dt: u8,  // 8-bit delay timer
    st: u8,  // 8-bit sound timer
    sp: u8,  // 8-bit stack pointer

    v: [u8; 16],          // 16 multi-purpose 8-bit registers
    stack: [u16; 32],     // 32 words deep call-stack
    mem: [u8; 4096usize], // 4 KiB RAM

    disp: renderer::AsciiDisplay, // The outpur display
    config: ChipCfg,              // Chip configuration
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            i: 0,
            pc: 0x200,
            dt: 0,
            st: 0,
            sp: 0,
            v: [0; 16],
            stack: [0; 32],
            mem: [0; 4096],
            disp: Default::default(),
            config: Default::default(),
        }
    }

    pub fn load_file(&mut self, path: &str) -> Result<(), io::Error> {
        // Load the file
        let mut f: File = File::open(path)?;

        // Start writing at address 0x200 (512)
        // Because 0x0 - 0x1FF is kept for internal use
        let addr = 0x200;
        f.read(&mut self.mem[addr..])?;
        Ok(())
    }

    pub fn load_font(&mut self, path: &str) -> Result<(), io::Error> {
        // Load the file
        let mut f: File = File::open(path)?;

        // Start writing at font_beg, as given by the config
        // Then write 80 (5 * 16) bytes
        let font_beg = self.config.font_start as usize;
        let font_end = font_beg + (5 * 16) as usize;
        f.read(&mut self.mem[font_beg..font_end])?;
        Ok(())
    }

    pub fn load_default_font(&mut self) {
        // Load the hardcoded font
        let offset = self.config.font_start as usize;
        for i in 0..DEFAULT_FONT.len() {
            self.mem[offset + i] = DEFAULT_FONT[i];
        }
    }

    pub fn fetch(&mut self) -> ChipInst {
        let b1 = self.mem[self.pc as usize];
        let b2 = self.mem[(self.pc + 1) as usize];
        self.pc += 2;
        let w: u16 = ((b1 as u16) << 8) | (b2 as u16);
        ChipInst::new(w)
    }
}