use super::memory::Memory;
use super::Display;

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
                    0x00_FF_FF_FF
                } else {
                    0x00_00_00_00
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
                let inner_collide = (0..8_u8).fold(false, |did_collide_inner, x_bit| {
                    let x_norm = (x + x_bit as u8) % FRAME_BUFFER_PIXEL_WIDTH as u8;
                    let sprite_pixel = ((sprite << x_bit) & 0x80) >> 7;

                    let buffer_index =
                        (y_norm as usize * FRAME_BUFFER_PIXEL_WIDTH + x_norm as usize) as usize;
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
