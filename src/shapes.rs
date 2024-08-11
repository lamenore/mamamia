use babel::*;
use imageproc::point::Point;

use crate::constants::CELL_SIZE;
use crate::types::SlopeType;

pub struct Polygon {
    pub points: Vec<Point<i32>>,
}

impl Polygon {
    pub fn new(points: Vec<Point<i32>>) -> Polygon {
        Polygon { points }
    }

    pub fn mirror_x(&mut self) {
        for point in &mut self.points {
            point.x = (-point.x) + CELL_SIZE as i32 - 1;
        }
    }

    pub fn mirror_y(&mut self) {
        for point in &mut self.points {
            point.y = (-point.y) + CELL_SIZE as i32 - 1;
        }
    }

    pub fn clamp(&mut self) {
        for point in &mut self.points {
            point.x = point.x.clamp(0, CELL_SIZE as i32 - 1);
            point.y = point.y.clamp(0, CELL_SIZE as i32 - 1);
        }
    }

    pub fn shift(&mut self, x: i32, y: i32) {
        for point in &mut self.points {
            point.x += x;
            point.y += y;
        }
    }

    pub fn symmetrize(&mut self) {
        // first insert points into self.points where the edge
        // of the polygon crosses the y-axis center line
        for i in 0..self.points.len() {
            let p1 = self.points[i];
            let p2 = self.points[(i + 1) % self.points.len()];

            // check if the line crosses the y-axis center line
            if (p1.y - CELL_SIZE as i32 / 2) * (p2.y - CELL_SIZE as i32 / 2) < 0 {
                // calculate the intersection point
                let x = p1.x + (p2.x - p1.x) * (CELL_SIZE as i32 / 2 - p1.y) / (p2.y - p1.y);
                self.points.insert(
                    i + 1,
                    Point {
                        x,
                        y: CELL_SIZE as i32 / 2,
                    },
                );
            }
        }

        // delete the right half of the polygon
        let points_to_remove: Vec<usize> = self
            .points
            .iter()
            .enumerate()
            .filter(|(_, point)| point.x > CELL_SIZE as i32 / 2)
            .map(|(i, _)| i)
            .collect();

        for i in points_to_remove.iter().rev() {
            self.points.remove(*i);
        }

        // mirror the left half of the polygon
        // from the last element to the first, push the mirrored
        for i in (0..self.points.len()).rev() {
            let point = self.points[i];
            self.points.push(Point {
                x: -point.x + CELL_SIZE as i32 - 1,
                y: point.y,
            });
        }
    }

    pub fn symplify(&mut self) {
        // check if a point already exists
        // check if a point is on the same line as the previous and next point
        // if so, remove the point
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        for point in &mut self.points {
            point.x += x as i32;
            point.y += y as i32;
        }
    }
}

impl From<SlopeType> for Polygon {
    fn from(slope: SlopeType) -> Polygon {
        match slope {
            SlopeType::HalfSolidH => Polygon::new(babel::SLOPE_HALF_SOLIDH.to_vec()),
            SlopeType::HalfSolidV => Polygon::new(babel::SLOPE_HALF_SOLIDV.to_vec()),
            SlopeType::SmallTriangle => Polygon::new(babel::SLOPE_SMALL_TRIANGLE.to_vec()),
            SlopeType::BigTriangle => Polygon::new(babel::SLOPE_BIG_TRIANGLE.to_vec()),
            SlopeType::HalfPlat => Polygon::new(babel::SLOPE_HALF_PLAT.to_vec()),
            SlopeType::Slope45 => Polygon::new(SLOPE_45.to_vec()),
            SlopeType::Square => Polygon::new(SLOPE_SQUARE.to_vec()),
            SlopeType::HillPart1 => Polygon::new(SLOPE_HILL_PART1.to_vec()),
            SlopeType::HillPart2 => Polygon::new(SLOPE_HILL_PART2.to_vec()),
            SlopeType::SmoothHillPart1 => Polygon::new(SLOPE_SMOOTH_HILL_PART1.to_vec()),
            SlopeType::SmoothHillPart2 => Polygon::new(SLOPE_SMOOTH_HILL_PART2.to_vec()),
            SlopeType::SmootherHillPart1 => Polygon::new(SLOPE_SMOOTHER_HILL_PART1.to_vec()),
            SlopeType::SmootherHillPart2 => Polygon::new(SLOPE_SMOOTHER_HILL_PART2.to_vec()),
            SlopeType::SmootherHillPart3 => Polygon::new(SLOPE_SMOOTHER_HILL_PART3.to_vec()),
            SlopeType::SteepHillPart1 => Polygon::new(SLOPE_STEEP_HILL_PART1.to_vec()),
            SlopeType::SteepHillPart2 => Polygon::new(SLOPE_STEEP_HILL_PART2.to_vec()),
            _ => Polygon::new(Vec::new()),
        }
    }
}

#[allow(dead_code)]
mod babel {
    use imageproc::point::Point;

    use crate::constants::CELL_SIZE;

    type PolygonShape = &'static [Point<i32>];

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

    pub const SLOPE_45: [Point<i32>; 3] = [
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
}

pub mod vectors {
    use imageproc::point::Point;

    use crate::{constants::CELL_SIZE, types::SlopeType};

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Vector {
        pub start: Point<i32>,
        pub end: Point<i32>,
    }

    impl Vector {
        pub fn new(start: Point<i32>, end: Point<i32>) -> Vector {
            Vector { start, end }
        }

        pub fn length(&self) -> f64 {
            let x = self.end.x - self.start.x;
            let y = self.end.y - self.start.y;
            ((x * x + y * y) as f64).sqrt()
        }

        pub fn dot(&self, other: &Vector) -> i32 {
            let x1 = self.end.x - self.start.x;
            let y1 = self.end.y - self.start.y;
            let x2 = other.end.x - other.start.x;
            let y2 = other.end.y - other.start.y;
            x1 * x2 + y1 * y2
        }

        pub fn angle(&self, other: &Vector) -> f64 {
            let dot = self.dot(other);
            let len1 = self.length();
            let len2 = other.length();
            let cos_theta = dot as f64 / (len1 * len2);
            cos_theta.acos()
        }
    }

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

    pub type SlopeVectors = &'static [Vector];

    impl From<SlopeType> for SlopeVectors {
        fn from(slope: SlopeType) -> &'static [Vector] {
            match slope {
                SlopeType::HalfSolidH => &VEC_SLOPE_HALF_SOLIDH,
                SlopeType::HalfSolidV => &VEC_SLOPE_HALF_SOLIDV,
                SlopeType::SmallTriangle => &VEC_SLOPE_SMALL_TRIANGLE,
                SlopeType::BigTriangle => &VEC_SLOPE_BIG_TRIANGLE,
                SlopeType::Slope45 => &VEC_SLOPE45,
                SlopeType::HillPart1 => &VEC_SLOPE_HILL_PART1,
                SlopeType::HillPart2 => &VEC_SLOPE_HILL_PART2,
                // _ => &VEC_SLOPE_TEST,
                // _ => &VEC_SLOPE_TEST3,
                _ => &[],
            }
        }
    }

    // pub const VEC_SLOPE_TEST: [Vector; 1] = [Vector {
    //     start: Point { x: 0, y: 0 },
    //     end: Point {
    //         x: CELL_SIZE as i32,
    //         y: CELL_SIZE as i32,
    //     },
    // }];

    // pub const VEC_SLOPE_TEST2: [Vector; 1] = [Vector {
    //     start: Point { x: 0, y: 0 },
    //     end: Point {
    //         x: 0,
    //         y: CELL_SIZE as i32,
    //     },
    // }];

    // pub const VEC_SLOPE_TEST3: [Vector; 1] = [Vector {
    //     start: Point { x: 0, y: 0 },
    //     end: Point {
    //         x: CELL_SIZE as i32,
    //         y: 0,
    //     },
    // }];

    // pub const VEC_SLOPE_TEST4: [Vector; 1] = [Vector {
    //     start: Point {
    //         x: CELL_SIZE as i32,
    //         y: 0,
    //     },
    //     end: Point { x: 0, y: 0 },
    // }];
}
