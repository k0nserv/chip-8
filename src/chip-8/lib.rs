use std::ops::{Index, IndexMut};

mod memory;
pub use memory::Memory;

struct Timer {
    value: u8,
}

impl Timer {
    fn new() -> Self {
        Self { value: 0 }
    }

    fn current_value(&self) -> u8 {
        self.value
    }

    fn set_value(&mut self, new_value: u8) {
        self.value = new_value;
    }

    fn tick(&mut self) {
        if self.is_active() {
            self.value -= 1;
        }
    }

    fn is_active(&self) -> bool {
        self.value > 0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Input {
    fn is_key_down(&self, key: u8) -> bool;
    fn last_key_down(&self) -> Option<u8>;
}

pub trait Display {
    fn is_dirty(&self) -> bool;
    fn clear_dirty(&mut self);
    fn rgba_framebuffer(&self) -> Vec<u32>;
    fn draw_sprite(
        &mut self,
        x: u8,
        y: u8,
        base_address: u16,
        bytes_to_read: u8,
        memory: &Memory,
    ) -> bool;
    fn cls(&mut self);
}

const FRAME_BUFFER_PIXEL_WIDTH: usize = 64;
const FRAME_BUFFER_PIXEL_HEIGHT: usize = 32;
pub struct FramebufferDisplay {
    framebuffer: [u8; FRAME_BUFFER_PIXEL_WIDTH * FRAME_BUFFER_PIXEL_HEIGHT],
    dirty: bool,
}

impl Default for FramebufferDisplay {
    fn default() -> Self {
        Self {
            framebuffer: [0; FRAME_BUFFER_PIXEL_WIDTH * FRAME_BUFFER_PIXEL_HEIGHT],
            dirty: true,
        }
    }
}

impl Display for FramebufferDisplay {
    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn rgba_framebuffer(&self) -> Vec<u32> {
        self.framebuffer
            .iter()
            .map(|&byte| {
                assert!(
                    byte == 1 || byte == 0,
                    "Invalid byte {} in framebuffer",
                    byte
                );
                if byte == 1 {
                    0x00_68_BB_ED
                } else {
                    0x002C_50_66
                }
            })
            .collect()
    }

    fn cls(&mut self) {
        self.framebuffer = [0; FRAME_BUFFER_PIXEL_WIDTH * FRAME_BUFFER_PIXEL_HEIGHT];
        self.dirty = true;
    }

    fn draw_sprite(
        &mut self,
        x: u8,
        y: u8,
        base_address: u16,
        bytes_to_read: u8,
        memory: &Memory,
    ) -> bool {
        self.dirty = true;
        let height = bytes_to_read;
        let sprites = memory.as_slice(base_address, height as u16);

        sprites
            .iter()
            .enumerate()
            .fold(false, |did_collide, (y_offset, sprite)| {
                let y_norm = (y + y_offset as u8) % FRAME_BUFFER_PIXEL_HEIGHT as u8;
                let inner_collide =
                    (0..8_u8)
                        .into_iter()
                        .fold(false, |did_collide_inner, x_bit| {
                            let x_norm = (x + x_bit as u8) % FRAME_BUFFER_PIXEL_WIDTH as u8;
                            let sprite_pixel = ((sprite << x_bit) & 0x80) >> 7;

                            let buffer_index = (y_norm as usize * FRAME_BUFFER_PIXEL_WIDTH
                                + x_norm as usize)
                                as usize;
                            let previous_display_value = self.framebuffer[buffer_index];

                            assert!(sprite_pixel == 0x1 || sprite_pixel == 0);
                            self.framebuffer[buffer_index] = previous_display_value ^ sprite_pixel;
                            if sprite_pixel > 0 {
                                did_collide_inner || previous_display_value == 1
                            } else {
                                did_collide_inner
                            }
                        });

                did_collide || inner_collide
            })
    }
}

#[derive(Debug)]
struct Registers([u8; 16]);

impl Registers {
    fn as_slice_through(&self, idx: u16) -> &[u8] {
        assert!(
            idx < 16,
            "Cannot slice register through idx: {}. 15 is the max",
            idx
        );

        &self.0[0..=(idx as usize)]
    }

    fn clone_from_slice(&mut self, slice: &[u8]) {
        assert!(
            slice.len() <= 16,
            "Cannot clone into registers from slice {:?}. It has too many entries",
            slice
        );
        self.0[0..slice.len()].copy_from_slice(slice)
    }
}

impl Index<u16> for Registers {
    type Output = u8;

    fn index(&self, address: u16) -> &Self::Output {
        assert!(address < 16, "Invalid register {:#02x}", address);

        &self.0[address as usize]
    }
}

impl IndexMut<u16> for Registers {
    fn index_mut(&mut self, address: u16) -> &mut Self::Output {
        assert!(address < 16, "Invalid register {:#02x}", address);

        &mut self.0[address as usize]
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self([0; 16])
    }
}

const STACK_SIZE: usize = 128;
pub struct CPU {
    // Registers
    v: Registers,
    i: u16,

    // Program Counter
    pc: u16,
    // Current opcode
    opcode: u16,

    // Stack
    stack: [u16; STACK_SIZE],
    sp: u16,

    memory: Memory,
    pub display: Box<dyn Display>,

    delay_timer: Timer,
    sound_timer: Timer,
}

impl CPU {
    pub fn new(memory: Memory, display: Box<dyn Display>) -> Self {
        Self {
            v: Registers::default(),
            i: 0,
            // Program Counter starts at 0x200
            pc: 0x200,
            opcode: 0,

            sp: 0,
            stack: [0; STACK_SIZE],

            memory,
            display,

            delay_timer: Timer::default(),
            sound_timer: Timer::default(),
        }
    }

    pub fn cycle(&mut self, tick_timers: bool, input: &dyn Input) {
        self.opcode =
            (self.memory[self.pc] as u16) << 8 | self.memory[self.pc.wrapping_add(1)] as u16;
        self.pc = self.execute_opcode(self.opcode, self.pc, tick_timers, input);
    }

    fn execute_opcode(
        &mut self,
        opcode: u16,
        current_pc: u16,
        tick_timers: bool,
        input: &dyn Input,
    ) -> u16 {
        self.display.clear_dirty();
        // println!("{:04x}: {:04x}", current_pc, opcode);
        let next_pc = match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x000F {
                    // 00E0: Clear screen
                    0x0000 => {
                        self.display.cls();

                        current_pc + 2
                    }
                    // 00EE: Return from subroutine
                    0x000E => self.stack_pop(),
                    _ => panic!("Unknown opcode {:#02x}", opcode),
                }
            }
            // 1NNN: Jump to address NNN
            0x1000 => opcode & 0x0FFF,
            // 2NNN: Call NNN
            0x2000 => {
                let mut address = opcode & 0x0FFF;
                if address < 0x200 {
                    address += 0x200;
                }
                self.stack_push(current_pc + 2);

                // Jump to address
                address
            }

            // 3XKK: Skip next instruction if VX is equal to KK.
            0x3000 => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;

                if self.v[register] == value {
                    current_pc + 4
                } else {
                    current_pc + 2
                }
            }

            // 4XKK: Skip next instruction if VX is not equal to KK.
            0x4000 => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;

                if self.v[register] != value {
                    current_pc + 4
                } else {
                    current_pc + 2
                }
            }

            // 5XY0: Skip next instruction if VX is equal to VY.
            0x5000 => {
                let lhs_register = (opcode & 0x0F00) >> 8;
                let rhs_register = (opcode & 0x00F0) >> 4;

                if self.v[lhs_register] == self.v[rhs_register] {
                    current_pc + 4
                } else {
                    current_pc + 2
                }
            }

            // 6XNN: Set VX to NN.
            0x6000 => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;

                self.v[register] = value;

                current_pc + 2
            }

            // 7XNN: Add NN to VX, carry flag is not changed.
            0x7000 => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;

                self.v[register] = self.v[register].wrapping_add(value);

                current_pc + 2
            }

            0x8000 => {
                let lhs_register = (opcode & 0x0F00) >> 8;
                let rhs_register = (opcode & 0x00F0) >> 4;

                match opcode & 0x000F {
                    // 8XY0: Set VX to the value of VY.
                    0x0000 => {
                        self.v[lhs_register] = self.v[rhs_register];
                    }

                    // 8XY1: Set VX to the result of VX | VY
                    0x0001 => {
                        self.v[lhs_register] = self.v[lhs_register] | self.v[rhs_register];
                    }

                    // 8XY2: Set VX to the result of VX & VY
                    0x0002 => {
                        self.v[lhs_register] = self.v[lhs_register] & self.v[rhs_register];
                    }

                    // 8XY3: Set VX to the result of VX ^ VY
                    0x0003 => {
                        self.v[lhs_register] = self.v[lhs_register] ^ self.v[rhs_register];
                    }

                    // 8XY4: Add VY to VX. VF is set to 1 if there is a carry, 0 if not.
                    0x0004 => {
                        let will_overflow = self.v[lhs_register]
                            .checked_add(self.v[rhs_register])
                            .is_none();
                        self.v[0xF] = if will_overflow { 1 } else { 0 };

                        self.v[lhs_register] =
                            self.v[lhs_register].wrapping_add(self.v[rhs_register]);
                    }

                    // 8XY5: Subtract VY from VX. VF is set to 0 if there is a borrow, 1 if not.
                    0x0005 => {
                        self.v[0xF] = if self.v[lhs_register] > self.v[rhs_register] {
                            1
                        } else {
                            0
                        };

                        self.v[lhs_register] =
                            self.v[lhs_register].wrapping_sub(self.v[rhs_register]);
                    }

                    // 8XY6: Store the least significant bit of VX in VF and then shift VX to the
                    // right by 1.
                    0x0006 => {
                        self.v[0xF] = self.v[lhs_register] & 0x1;
                        self.v[lhs_register] = self.v[lhs_register] >> 1;
                    }

                    // 8XY7: Set VX to the result of VY - VX. VF is set 0 when there is a borrow, 1
                    // if not.
                    0x0007 => {
                        self.v[0xF] = if self.v[rhs_register] > self.v[lhs_register] {
                            1
                        } else {
                            0
                        };
                        self.v[lhs_register] =
                            self.v[rhs_register].wrapping_sub(self.v[lhs_register]);
                    }

                    // 8XYE: Store the most significant bit of VX in VF and then shift VX to the
                    // left by 1.
                    0x000E => {
                        self.v[0xF] = (self.v[lhs_register] & 0x80) >> 7;
                        self.v[lhs_register] = self.v[lhs_register] << 1;
                    }
                    _ => panic!("Unknown opcode {:#02x}", opcode),
                }

                current_pc + 2
            }

            // 9XY0: Skip the next instruction if VX is not equal VY
            0x9000 => {
                let lhs_register = (opcode & 0x0F00) >> 8;
                let rhs_register = (opcode & 0x00F0) >> 4;

                if self.v[lhs_register] != self.v[rhs_register] {
                    current_pc + 4
                } else {
                    current_pc + 2
                }
            }

            // ANNN: Set `I` to address NNN
            0xA000 => {
                self.i = opcode & 0x0FFF;

                current_pc + 2
            }

            // BNNN: Jump to the address NNN + V0
            0xB000 => {
                let address = opcode & 0x0FFF;

                address + self.v[0] as u16
            }

            // CXNN: Set the VX to the result of rand() & NN.
            0xC000 => {
                let random: u8 = rand::random();
                let mask = (opcode & 0x00FF) as u8;
                let target_register = (opcode & 0x0F00) >> 8;
                let value = mask & random;

                self.v[target_register] = value;

                current_pc + 2
            }

            // DXYN: Draw a sprite at VX, VY of widht 8 and height N.
            0xD000 => {
                // println!("{:04x}", opcode);
                let x = self.v[(opcode & 0x0F00) >> 8];
                let y = self.v[(opcode & 0x00F0) >> 4];
                let n = (opcode & 0x000F) as u8;

                self.v[0xF] = if self.display.draw_sprite(x, y, self.i, n, &self.memory) {
                    1
                } else {
                    0
                };

                current_pc + 2
            }

            0xE000 => {
                let register_value = self.v[(opcode & 0x0F00) >> 8];

                match opcode & 0x00FF {
                    // EX9E: Skip the next instruction if the key stored in VX is pressed
                    0x009E => {
                        if input.is_key_down(register_value) {
                            current_pc + 4
                        } else {
                            current_pc + 2
                        }
                    }

                    // EXA1: Skip the next instruction if the key stored in VX isn't pressed
                    0x00A1 => {
                        if input.is_key_down(register_value) {
                            current_pc + 2
                        } else {
                            current_pc + 4
                        }
                    }
                    _ => panic!("Unknown opcode {:#02x}", opcode),
                }
            }

            0xF000 => {
                let register = (opcode & 0x0F00) >> 8;
                let blocked = match opcode & 0x00FF {
                    // FX07: Set the VX value to the value of the delay timer
                    0x0007 => {
                        self.v[register] = self.delay_timer.current_value();

                        false
                    }

                    // FX0A: Block execution until a key is pressed. Pressed key is stored in VX.
                    0x000A => match input.last_key_down() {
                        Some(key) => {
                            self.v[register] = key;
                            false
                        }
                        None => true,
                    },

                    // FX15: Set the delay timer to the value of VX
                    0x0015 => {
                        self.delay_timer.set_value(self.v[register]);

                        false
                    }

                    // FX18: Set the sound timer to the value of VX
                    0x0018 => {
                        self.sound_timer.set_value(self.v[register]);

                        false
                    }

                    // FX1E: Add VX to I
                    0x001E => {
                        self.i = self.i.wrapping_add(self.v[register] as u16);

                        false
                    }

                    // FX29: Set I to the location of the sprite for the character in VX.
                    0x0029 => {
                        self.i = self.memory.font_address_for_character(self.v[register]);

                        false
                    }

                    // FX33:  Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    0x0033 => {
                        let value = self.v[register];

                        self.memory[self.i] = value / 100;
                        self.memory[self.i + 1] = (value / 10) % 10;
                        self.memory[self.i + 2] = (value % 100) % 10;

                        false
                    }

                    // FX55: Store registers V0 through VX in memory starting at I.
                    0x0055 => {
                        self.memory
                            .copy_from_slice(self.i, self.v.as_slice_through(register));

                        false
                    }

                    // FX65: Read into register v0 through VX starting at I.
                    0x0065 => {
                        self.v
                            .clone_from_slice(self.memory.as_slice(self.i, register + 1));

                        false
                    }

                    _ => panic!("Unknown opcode {:#02x}", opcode),
                };

                if !blocked {
                    current_pc + 2
                } else {
                    current_pc
                }
            }
            _ => panic!("Unknown opcode {:#02x}", opcode),
        };

        if tick_timers {
            self.delay_timer.tick();
            self.sound_timer.tick();
        }

        next_pc
    }

    fn stack_push(&mut self, value: u16) {
        assert!(
            (self.sp as usize) < STACK_SIZE,
            "Attempting to push when stack is full"
        );
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        assert!(self.sp != 0, "Attempting to pop empty stack");
        let value = self.stack[(self.sp - 1) as usize];
        self.sp -= 1;

        value
    }
}
