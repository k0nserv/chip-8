use std::ops::{Index, IndexMut};

const MEMORY_SIZE: usize = 4096;
const FONTSET_BASE_ADDRESS: u16 = 0x50;
const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // Fu
];

struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Memory {
    fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];
        memory[(FONTSET_BASE_ADDRESS as usize)..0xA0].clone_from_slice(&FONTSET);

        Self {
            memory: [0; MEMORY_SIZE],
        }
    }

    fn font_address_for_character(&self, character: u8) -> u16 {
        FONTSET_BASE_ADDRESS + (character as u16 * 5)
    }

    fn clone_from_slice(&mut self, base_address: u16, slice: &[u8]) {
        self.memory[(base_address as usize)..(base_address as usize + slice.len())]
            .clone_from_slice(slice);
    }

    fn as_slice(&self, base_address: u16, length: u16) -> &[u8] {
        &self.memory[base_address as usize..(base_address as usize + length as usize)]
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, address: u16) -> &Self::Output {
        assert!(
            address < MEMORY_SIZE as u16,
            "Invalid memory address {:#02x}",
            address
        );

        &self.memory[address as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, address: u16) -> &mut Self::Output {
        assert!(
            address < MEMORY_SIZE as u16,
            "Invalid memory address {:#02x}",
            address
        );

        &mut self.memory[address as usize]
    }
}

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

trait Input {
    fn is_key_down(&self, key: u8) -> bool;
    fn await_key(&self) -> u8;
}

trait Display {
    fn draw_sprite(&mut self, x: u8, y: u8, base_address: u16, bytes_to_read: u8) -> bool;
    fn cls(&mut self);
}

struct NOPDisplay {}
impl Default for NOPDisplay {
    fn default() -> Self {
        Self {}
    }
}

impl Display for NOPDisplay {
    fn draw_sprite(&mut self, x: u8, y: u8, base_address: u16, bytes_to_read: u8) -> bool {
        // Do nothing
        true
    }

    fn cls(&mut self) {
        // NOP
    }
}

struct NOPInput {}

impl Default for NOPInput {
    fn default() -> Self {
        Self {}
    }
}

impl Input for NOPInput {
    fn is_key_down(&self, key: u8) -> bool {
        false
    }

    fn await_key(&self) -> u8 {
        loop {}
    }
}

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
        self.0.clone_from_slice(slice)
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

const STACK_SIZE: usize = 16;
struct CPU {
    // Registers
    v: Registers,
    i: u16,

    // Program Counter
    pc: u16,
    // Current opcode
    opcode: u16,

    // Stack
    stack: [u16; 16],
    sp: u16,

    memory: Memory,
    display: Box<dyn Display>,
    input: Box<dyn Input>,

    delay_timer: Timer,
    sound_timer: Timer,
}

impl CPU {
    fn new(memory: Memory, display: Box<dyn Display>, input: Box<dyn Input>) -> Self {
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
            input,

            delay_timer: Timer::default(),
            sound_timer: Timer::default(),
        }
    }

    fn cycle(&mut self) {
        self.opcode =
            (self.memory[self.pc] as u16) << 8 | self.memory[self.pc.wrapping_add(1)] as u16;
        self.pc = self.execute_opcode(self.opcode, self.pc);
    }

    fn execute_opcode(&mut self, opcode: u16, current_pc: u16) -> u16 {
        let next_pc = match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x000F {
                    // 00E0: Clear screen
                    0x0000 => {
                        self.display.cls();

                        current_pc + 2
                    }
                    // 00EE: Return from subroutine
                    0x000E => {
                        let return_to = self.stack_pop() + 2;
                        return_to
                    }
                    _ => panic!("Unknown opcode {:#02x}", opcode),
                }
            }
            // 1NNN: Jump to address NNN
            0x1000 => opcode & 0x0FFF,
            // 2NNN: Call NNN
            0x2000 => {
                let address = opcode & 0x0FFF;
                self.stack_push(current_pc);

                // Jump to address
                address
            }

            // 3XKK: Skip next instruction if VX is equal to KK.
            0x3000 => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;

                if self.v[register] == value {
                    current_pc + 2
                } else {
                    current_pc + 4
                }
            }

            // 4XKK: Skip next instruction if VX is not equal to KK.
            0x4000 => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;

                if self.v[register] != value {
                    current_pc + 2
                } else {
                    current_pc + 4
                }
            }

            // 5XY0: Skip next instruction if VX is equal to VY.
            0x5000 => {
                let lhs_register = (opcode & 0x0F00) >> 8;
                let rhs_register = (opcode & 0x00F0) >> 4;

                if self.v[lhs_register] == self.v[rhs_register] {
                    current_pc + 2
                } else {
                    current_pc + 4
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
                        self.v[0xF] = self.v[lhs_register] & 0b1000_0000;
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
                    current_pc + 2
                } else {
                    current_pc + 4
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
                let target_register = (opcode & 0x0F00) >> 8;
                let value = ((opcode & 0x00FF) >> 8) as u8;

                self.v[target_register] = random & value;

                current_pc + 2
            }

            // DXYN: Draw a sprite at VX, VY of widht 8 and height N.
            0xD000 => {
                let x = self.v[(opcode & 0x0F00) >> 8];
                let y = self.v[(opcode & 0x00F0) >> 4];
                let n = self.v[opcode & 0x000F];

                self.v[0xF] = if self.display.draw_sprite(x, y, self.i, n) {
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
                        if self.input.is_key_down(register_value) {
                            current_pc + 4
                        } else {
                            current_pc + 2
                        }
                    }

                    // EXA1: Skip the next instruction if the key stored in VX isn't pressed
                    0x00A1 => {
                        if self.input.is_key_down(register_value) {
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
                match opcode & 0x00FF {
                    // FX07: Set the VX value to the value of the delay timer
                    0x0007 => {
                        self.v[register] = self.delay_timer.current_value();
                    }
                    // FX0A: Block execution until a key is pressed. Pressed key is stored in VX.
                    0x000A => {
                        self.v[register] = self.input.await_key();
                    }

                    // FX15: Set the delay timer to the value of VX
                    0x0005 => {
                        self.delay_timer.set_value(self.v[register]);
                    }

                    // FX18: Set the sound timer to the value of VX
                    0x0008 => {
                        self.sound_timer.set_value(self.v[register]);
                    }

                    // FX1E: Add VX to I
                    0x000E => {
                        self.i = self.i.wrapping_add(self.v[register] as u16);
                    }

                    // FX29: Set I to the location of the sprite for the character in VX.
                    0x0009 => {
                        self.i = self.memory.font_address_for_character(self.v[register]);
                    }

                    // FX33:  Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    0x0033 => {
                        let value = self.v[register];

                        self.memory[self.i] = value / 100;
                        self.memory[self.i + 1] = (value / 10) % 10;
                        self.memory[self.i + 2] = (value % 100) % 10;
                    }

                    // FX55: Store registers V0 through VX in memory starting at I.
                    0x0055 => {
                        self.memory
                            .clone_from_slice(self.i, self.v.as_slice_through(register));
                    }

                    // FX65: Read into register v0 through VX starting at I.
                    0x0065 => {
                        self.v
                            .clone_from_slice(self.memory.as_slice(self.i, register + 1));
                    }

                    _ => panic!("Unknown opcode {:#02x}", opcode),
                }

                current_pc + 2
            }
            _ => panic!("Unknown opcode {:#02x}", opcode),
        };

        self.delay_timer.tick();
        self.sound_timer.tick();

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

#[cfg(test)]
mod test {
    use super::{Display, Input, Memory, NOPInput, Timer, CPU};
    struct DisplayStub {
        clear_calls: usize,
    }

    impl Display for DisplayStub {
        fn cls(&mut self) {
            self.clear_calls += 1;
        }

        fn draw_sprite(&mut self, x: u8, y: u8, base_address: u16, bytes_to_read: u8) -> bool {
            // TODO

            true
        }
    }

    impl Default for DisplayStub {
        fn default() -> Self {
            Self { clear_calls: 0 }
        }
    }

    fn make_test_cpu() -> CPU {
        CPU::new(
            Memory::default(),
            Box::new(DisplayStub::default()),
            Box::new(NOPInput::default()),
        )
    }

    #[test]
    fn test_jump() {
        let mut cpu = make_test_cpu();

        let pc = cpu.execute_opcode(0x1FED, cpu.pc);

        assert_eq!(pc, 0x0FED);
    }

    #[test]
    fn test_cls() {
        let mut cpu = make_test_cpu();

        let pc = cpu.execute_opcode(0x00E0, cpu.pc);

        assert_eq!(pc, 0x202);
    }

    #[test]
    fn test_load_registers_into_memory() {
        let mut cpu = make_test_cpu();
        let data: Vec<u8> = (0..16).into_iter().map(|i| i * 2).collect();
        cpu.v.clone_from_slice(&data);

        let pc = cpu.execute_opcode(0xFF55, cpu.pc);

        assert_eq!(cpu.memory.as_slice(cpu.i, 16), data.as_slice());
    }

    #[test]
    fn test_load_memory_into_registers() {
        let mut cpu = make_test_cpu();
        let data: Vec<u8> = (0..16).into_iter().map(|i| i * 2).collect();
        cpu.memory.clone_from_slice(cpu.i, &data);

        let pc = cpu.execute_opcode(0xFF65, cpu.pc);

        assert_eq!(cpu.v.as_slice_through(15), data.as_slice());
    }
}
