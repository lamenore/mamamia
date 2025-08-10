use crate::{constants::CELL_SIZE, shapes::point::Point};

pub const SLOPE_HALF_SOLIDH: [Point<i32>; 4] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_HALF_SOLIDV: [Point<i32>; 4] = [
    Point {
        x: CELL_SIZE as i32 / 2,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
];

pub const SLOPE_QUARTER_SOLID: [Point<i32>; 4] = [
    Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_QUARTER_AIR: [Point<i32>; 6] = [
    Point {
        x: CELL_SIZE as i32 / 2,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: 0,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32 / 2,
    },
];

pub const SLOPE_SMALL_TRIANGLE: [Point<i32>; 3] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32 / 2,
    },
];

pub const SLOPE_BIG_TRIANGLE: [Point<i32>; 3] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 / 2,
        y: 0,
    },
];

pub const SLOPE_HALF_PLAT: [Point<i32>; 4] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_CONCAVE_TRIANGLE: [Point<i32>; 4] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 / 2 - 1 + 2,
        y: CELL_SIZE as i32 / 2 - 1 + 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_45: [Point<i32>; 3] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: (CELL_SIZE as i32) - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
];

pub const SLOPE_SQUARE: [Point<i32>; 4] = [
    Point { x: 0, y: 0 },
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
];

pub const SLOPE_HILL_PART1: [Point<i32>; 3] = [
    Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 / 2,
    },
];

// used when there is a slope45 on the left and anything on the right
pub const SLOPE_HILL_PART2: [Point<i32>; 5] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 / 2 - 1,
    },
    Point {
        x: CELL_SIZE as i32 / 2 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_SMOOTH_HILL_PART1: [Point<i32>; 3] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_SMOOTH_HILL_PART2: [Point<i32>; 4] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 / 2,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_SMOOTHER_HILL_PART1: [Point<i32>; 3] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 * 2 / 3,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_SMOOTHER_HILL_PART2: [Point<i32>; 4] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 * 2 / 3,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 / 3,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_SMOOTHER_HILL_PART3: [Point<i32>; 3] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 / 3,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_STEEP_HILL_PART1: [Point<i32>; 3] = [
    Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
];

pub const SLOPE_STEEP_HILL_PART2: [Point<i32>; 4] = [
    Point {
        x: 0,
        y: CELL_SIZE as i32 - 1,
    },
    Point {
        x: CELL_SIZE as i32 / 2,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: 0,
    },
    Point {
        x: CELL_SIZE as i32 - 1,
        y: CELL_SIZE as i32 - 1,
    },
];
