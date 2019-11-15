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
