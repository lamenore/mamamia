use crate::types::cell::SlopeType;

use crate::constants::CELL_SIZE;

use imageproc::point::Point;

use super::Vector;

pub const VEC_SLOPE_HALF_SOLIDH: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32 / 2,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: CELL_SIZE as i32 / 2,
    },
}];

pub const VEC_SLOPE_HALF_SOLIDV: [Vector; 1] = [Vector {
    start: Point {
        x: CELL_SIZE as i32 / 2,
        y: 0,
    },
    end: Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32,
    },
}];

pub const VEC_SLOPE_SMALL_TRIANGLE: [Vector; 2] = [
    Vector {
        start: Point {
            x: 0,
            y: CELL_SIZE as i32,
        },
        end: Point {
            x: CELL_SIZE as i32 / 2,
            y: CELL_SIZE as i32 / 2,
        },
    },
    Vector {
        start: Point {
            x: CELL_SIZE as i32,
            y: CELL_SIZE as i32,
        },
        end: Point {
            x: CELL_SIZE as i32 / 2,
            y: CELL_SIZE as i32 / 2,
        },
    },
];

pub const VEC_SLOPE_BIG_TRIANGLE: [Vector; 2] = [
    Vector {
        start: Point {
            x: 0,
            y: CELL_SIZE as i32,
        },
        end: Point {
            x: CELL_SIZE as i32 / 2,
            y: 0,
        },
    },
    Vector {
        start: Point {
            x: CELL_SIZE as i32,
            y: CELL_SIZE as i32,
        },
        end: Point {
            x: CELL_SIZE as i32 / 2,
            y: 0,
        },
    },
];

pub const VEC_SLOPE_CONCAVE_TRIANGLE: [Vector; 2] = [
    Vector {
        start: Point {
            x: 0,
            y: CELL_SIZE as i32,
        },
        end: Point {
            x: (CELL_SIZE as i32 / 2) + 2,
            y: (CELL_SIZE as i32 / 2) + 2,
        },
    },
    Vector {
        start: Point {
            x: (CELL_SIZE as i32 / 2) + 2,
            y: (CELL_SIZE as i32 / 2) + 2,
        },
        end: Point {
            x: CELL_SIZE as i32,
            y: 0,
        },
    },
];

pub const VEC_SLOPE45: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: 0,
    },
}];

pub const VEC_SLOPE_HILL_PART1: [Vector; 1] = [Vector {
    start: Point {
        x: CELL_SIZE as i32 / 2,
        y: CELL_SIZE as i32,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: CELL_SIZE as i32 / 2,
    },
}];

pub const VEC_SLOPE_HILL_PART2: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32 / 2,
    },
    end: Point {
        x: CELL_SIZE as i32 / 2,
        y: 0,
    },
}];

pub const VEC_SLOPE_SMOOTH_HILL_PART1: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: CELL_SIZE as i32 / 2,
    },
}];

pub const VEC_SLOPE_SMOOTH_HILL_PART2: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32 / 2,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: 0,
    },
}];

pub const VEC_SLOPE_SMOOTHER_HILL_PART1: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: CELL_SIZE as i32 * 2 / 3,
    },
}];

pub const VEC_SLOPE_SMOOTHER_HILL_PART2: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32 * 2 / 3,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: CELL_SIZE as i32 / 3,
    },
}];

pub const VEC_SLOPE_SMOOTHER_HILL_PART3: [Vector; 1] = [Vector {
    start: Point {
        x: 0,
        y: CELL_SIZE as i32 / 3,
    },
    end: Point {
        x: CELL_SIZE as i32,
        y: 0,
    },
}];

pub type SlopeVectors = &'static [Vector];

impl From<SlopeType> for SlopeVectors {
    fn from(slope: SlopeType) -> &'static [Vector] {
        match slope {
            SlopeType::None => &[],
            // SlopeType::HalfSolidH => &VEC_SLOPE_HALF_SOLIDH,
            // SlopeType::HalfSolidV => &VEC_SLOPE_HALF_SOLIDV,
            SlopeType::SmallTriangle => &VEC_SLOPE_SMALL_TRIANGLE,
            SlopeType::BigTriangle => &VEC_SLOPE_BIG_TRIANGLE,
            SlopeType::ConcaveTriangle => &VEC_SLOPE_CONCAVE_TRIANGLE,
            SlopeType::Slope45 => &VEC_SLOPE45,
            SlopeType::HillPart1 => &VEC_SLOPE_HILL_PART1,
            SlopeType::HillPart2 => &VEC_SLOPE_HILL_PART2,
            SlopeType::SmoothHillPart1 => &VEC_SLOPE_SMOOTH_HILL_PART1,
            SlopeType::SmoothHillPart2 => &VEC_SLOPE_SMOOTH_HILL_PART2,
            SlopeType::SmootherHillPart1 => &VEC_SLOPE_SMOOTHER_HILL_PART1,
            SlopeType::SmootherHillPart2 => &VEC_SLOPE_SMOOTHER_HILL_PART2,
            SlopeType::SmootherHillPart3 => &VEC_SLOPE_SMOOTHER_HILL_PART3,
            // _ => &VEC_SLOPE_TEST,
            // _ => &VEC_SLOPE_TEST3,
            _ => &[],
        }
    }
}
