use rand::Rng;

use image::Rgba;

pub mod cell;
pub mod disjointed_set;
pub mod room;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Flip {
    None = 0x0,
    Horizontal,
    Vertical,
    Both,
}

impl From<u8> for Flip {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Flip::None,
            0x1 => Flip::Horizontal,
            0x2 => Flip::Vertical,
            0x3 => Flip::Both,
            _ => Flip::None,
        }
    }
}

#[allow(dead_code)]
fn get_random_color() -> Rgba<u8> {
    Rgba([
        rand::thread_rng().gen_range(0..=255),
        rand::thread_rng().gen_range(0..=255),
        rand::thread_rng().gen_range(0..=255),
        255,
    ])
}
