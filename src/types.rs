use rand::Rng;

use image::Rgba;

pub mod address;
pub mod cell;
pub mod disjointed_set;
pub mod rom;
pub mod room;

#[allow(dead_code)]
fn get_random_color() -> Rgba<u8> {
    Rgba([
        rand::thread_rng().gen_range(0..=255),
        rand::thread_rng().gen_range(0..=255),
        rand::thread_rng().gen_range(0..=255),
        255,
    ])
}
