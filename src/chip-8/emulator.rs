use crate::cpu::CPU;
use crate::memory::Memory;
use crate::{Display, Input, RandomNumberProvider};

pub struct Emulator {
    cpu: CPU,
    current_rom: Vec<u8>,
    is_initial_state: bool,
}

impl Emulator {
    pub fn new(
        display: Box<dyn Display>,
        rom: Vec<u8>,
        random_number_provider: Box<RandomNumberProvider>,
    ) -> Self {
        let mut memory = Memory::default();
        memory.copy_from_slice(0x200, &rom);
        let cpu = CPU::new(memory, display, random_number_provider);

        Self {
            cpu,
            current_rom: rom,
            is_initial_state: true,
        }
    }

    pub fn is_initial_state(&self) -> bool {
        self.is_initial_state
    }

    pub fn reset(self) -> Self {
        let mut memory = Memory::default();
        memory.copy_from_slice(0x200, &self.current_rom);
        let mut cpu = self.cpu.reset(memory);

        Self {
            cpu,
            current_rom: self.current_rom,
            is_initial_state: true,
        }
    }

    pub fn cycle(&mut self, should_tick_timer: bool, input: &dyn Input) {
        if self.is_initial_state {
            self.is_initial_state = false;
        }

        self.cpu.cycle(should_tick_timer, input);
    }

    pub fn display(&self) -> &dyn Display {
        self.cpu.display.as_ref()
    }
}
