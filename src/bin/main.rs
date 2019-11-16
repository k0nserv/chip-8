use chip_8;

use chip_8::{Emulator, FramebufferDisplay, Input};
use clap::{crate_authors, crate_version, App, Arg};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

const MICROS_BETWEEN_CYCLES: u128 = 1000_000 / 1000;
const MICROS_BETWEEN_TIMER_TICKS: u128 = 1000_000 / 60;
const MICROS_BETWEEN_DISPLAY_REFRESH: u128 = 1000_000 / 60;

struct MiniFBInput {
    key_states: [bool; 16],
    last_down: Option<u8>,
}

impl MiniFBInput {
    fn new() -> Self {
        Self {
            key_states: [false; 16],
            last_down: None,
        }
    }

    fn update_key_state(&mut self, window: &Window) {
        for key in 0..0xF {
            if let Some(key_enum) = MiniFBInput::map_key(key) {
                self.key_states[key as usize] = window.is_key_down(key_enum);
            }
        }

        self.last_down = window
            .get_keys()
            .map(|keys| {
                keys.iter()
                    .filter_map(|key_enum| MiniFBInput::map_key_enum(key_enum))
                    .nth(0)
            })
            .unwrap_or(None);
    }

    fn map_key(key: u8) -> Option<Key> {
        match key {
            0x1 => Some(Key::Key1),
            0x2 => Some(Key::Key2),
            0x3 => Some(Key::Key3),
            0xc => Some(Key::Key4),

            0x4 => Some(Key::Q),
            0x5 => Some(Key::W),
            0x6 => Some(Key::E),
            0xd => Some(Key::R),

            0x7 => Some(Key::A),
            0x8 => Some(Key::S),
            0x9 => Some(Key::D),
            0xe => Some(Key::F),

            0xa => Some(Key::Z),
            0x0 => Some(Key::X),
            0xb => Some(Key::C),
            0xf => Some(Key::V),
            _ => None,
        }
    }

    fn map_key_enum(key: &Key) -> Option<u8> {
        match key {
            Key::Key1 => Some(0x1),
            Key::Key2 => Some(0x2),
            Key::Key3 => Some(0x3),
            Key::Key4 => Some(0xc),

            Key::Q => Some(0x4),
            Key::W => Some(0x5),
            Key::E => Some(0x6),
            Key::R => Some(0xd),

            Key::A => Some(0x7),
            Key::S => Some(0x8),
            Key::D => Some(0x9),
            Key::F => Some(0xe),

            Key::Z => Some(0xa),
            Key::X => Some(0x0),
            Key::C => Some(0xb),
            Key::V => Some(0xf),
            _ => None,
        }
    }
}

impl Input for MiniFBInput {
    fn is_key_down(&self, key: u8) -> bool {
        self.key_states[key as usize]
    }
    fn last_key_down(&self) -> Option<u8> {
        None
    }
}

fn load_rom(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn create_window() -> Result<Window, Box<dyn std::error::Error>> {
    let mut opts = WindowOptions::default();

    opts.scale = Scale::X16;
    let window = Window::new("CHIP-8", 64, 32, opts)?;

    Ok(window)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("CHIP-8")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A CHIP-8 emulator")
        .arg(
            Arg::with_name("ROM")
                .help("The CHIP-8 ROM to run")
                .required(true)
                .index(1),
        )
        .get_matches();

    let mut last_instant = Instant::now();
    let mut last_timer_tick = Instant::now();
    let mut last_redraw = Instant::now();
    let rom = load_rom(Path::new(matches.value_of("ROM").unwrap()))?;

    let mut window = create_window()?;
    let mut input = MiniFBInput::new();
    let display = FramebufferDisplay::default();
    let mut emulator = Emulator::new(Box::new(display), rom);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::F1, KeyRepeat::No) && !emulator.is_initial_state() {
            emulator = emulator.reset();
            last_instant = Instant::now();
            last_timer_tick = Instant::now();
            last_redraw = Instant::now();
            continue;
        }

        let delta = last_instant.elapsed();
        let timer_delta = last_timer_tick.elapsed();

        let should_tick_timer = if timer_delta.as_micros() >= MICROS_BETWEEN_TIMER_TICKS {
            last_timer_tick = Instant::now();

            true
        } else {
            false
        };

        if delta.as_micros() >= MICROS_BETWEEN_CYCLES {
            if should_tick_timer {
                input.update_key_state(&window);
            }

            emulator.cycle(should_tick_timer, &input);
            last_instant = Instant::now();
        }

        if emulator.display().is_dirty()
            && last_redraw.elapsed().as_micros() >= MICROS_BETWEEN_DISPLAY_REFRESH
        {
            let buffer = emulator
                .display()
                .rgba_framebuffer()
                .into_iter()
                .map(|value| {
                    if value == 0x0 {
                        0x002C_50_66
                    } else {
                        0x00_68_BB_ED
                    }
                })
                .collect::<Vec<u32>>();

            window.update_with_buffer(&buffer)?;
        }

        if delta.as_micros() < MICROS_BETWEEN_CYCLES {
            let ms_to_sleep = (MICROS_BETWEEN_CYCLES - delta.as_micros()) / 1000;
            if ms_to_sleep > 0 {
                // std::thread::sleep(Duration::from_millis(ms_to_sleep as u64));
            }
        }
    }

    Ok(())
}
