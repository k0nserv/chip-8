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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// Main memory holding 4KiB of data.
/// The first 0x200 locations are reserved for private
/// use, namely the built in font.
///
pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Memory {
    /// Construct a new instance of `Memory`.
    ///
    /// The reserved memory regions will be intiailized appropriately
    /// and a ROM can be loaded at 0x200 to start execution.
    ///
    fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];
        memory[(FONTSET_BASE_ADDRESS as usize)..(FONTSET_BASE_ADDRESS as usize + FONTSET.len())]
            .copy_from_slice(&FONTSET);

        Self { memory: memory }
    }

    pub fn font_address_for_character(&self, character: u8) -> u16 {
        FONTSET_BASE_ADDRESS + (character as u16 * 5)
    }

    pub fn copy_from_slice(&mut self, base_address: u16, slice: &[u8]) {
        self.memory[(base_address as usize)..(base_address as usize + slice.len())]
            .copy_from_slice(slice);
    }

    pub fn as_slice(&self, base_address: u16, length: u16) -> &[u8] {
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

#[cfg(test)]
mod tests {
    use super::{Memory, FONTSET_BASE_ADDRESS};

    #[test]
    fn test_default() {
        let memory = Memory::default();

        assert_eq!(memory[FONTSET_BASE_ADDRESS], 0xF0);
        assert_eq!(memory[FONTSET_BASE_ADDRESS + 79], 0x80);
        assert_eq!(memory[0x200], 0x00);
    }

    #[test]
    fn test_font_address_for_character() {
        let memory = Memory::default();

        assert_eq!(
            memory.font_address_for_character(5),
            FONTSET_BASE_ADDRESS + 25
        );
    }

    #[test]
    fn test_copy_from_slice() {
        let mut memory = Memory::default();

        let rom = [0x00, 0xE0, 0x12, 0x00];
        memory.copy_from_slice(0x200, &rom);

        assert_eq!(&memory.memory[0x200..0x204], &rom);
    }

    #[test]
    fn test_as_slice() {
        let memory = Memory::default();

        let expected = [0x90, 0x90, 0xF0, 0x10, 0x10];

        assert_eq!(memory.as_slice(FONTSET_BASE_ADDRESS + 20, 5), &expected);
    }
}
