mod cpu;
mod display;
mod memory;
mod timer;

pub use cpu::CPU;
pub use display::FramebufferDisplay;
pub use memory::Memory;

pub trait Input {
    fn is_key_down(&self, key: u8) -> bool;
    fn last_key_down(&self) -> Option<u8>;
}

/// The Display for the emulator, typically 64x32 pixels.
pub trait Display {
    /// Wether the Display is dirty i.e. needs to be rewdrawn in the next draw cycle.
    fn is_dirty(&self) -> bool;

    /// Clear the dirty flag, typically after drawing in a draw cycle.
    fn clear_dirty(&mut self);

    /// The current framebuffer as a packed vector of u32 values. Each
    /// value u32 values represents a single pixel on the format XRGB. The `X`
    /// nibble is ignored when rendering as alpha is not supported.
    ///
    /// Should be in row major layout.
    fn rgba_framebuffer(&self) -> Vec<u32>;

    /// Draw a sprite at `x`, `y` in the display starting from `base_address` in the RAM.
    /// `bytes_to_read` specifies the height of sprite to draw.
    fn draw_sprite(
        &mut self,
        x: u8,
        y: u8,
        base_address: u16,
        bytes_to_read: u8,
        memory: &Memory,
    ) -> bool;

    /// Clear the screen by setting all pixels back to 0.
    fn cls(&mut self);
}
