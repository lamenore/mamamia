pub mod constants;

use crate::shapes::polygon::constants::*;
use crate::{constants::CELL_SIZE, types::cell::SlopeType};

use super::point::Point;

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
    fn from(slope_type: SlopeType) -> Polygon {
        let points = match slope_type {
            SlopeType::HalfSolidH => SLOPE_HALF_SOLIDH.to_vec(),
            SlopeType::HalfSolidV => SLOPE_HALF_SOLIDV.to_vec(),
            SlopeType::QuarterSolid => SLOPE_QUARTER_SOLID.to_vec(),
            SlopeType::QuarterAir => SLOPE_QUARTER_AIR.to_vec(),
            SlopeType::SmallTriangle => SLOPE_SMALL_TRIANGLE.to_vec(),
            SlopeType::BigTriangle => SLOPE_BIG_TRIANGLE.to_vec(),
            SlopeType::ConcaveTriangle => SLOPE_CONCAVE_TRIANGLE.to_vec(),
            SlopeType::HalfPlat => SLOPE_HALF_PLAT.to_vec(),
            SlopeType::Slope45 => SLOPE_45.to_vec(),
            SlopeType::Square => SLOPE_SQUARE.to_vec(),
            SlopeType::HillPart1 => SLOPE_HILL_PART1.to_vec(),
            SlopeType::HillPart2 => SLOPE_HILL_PART2.to_vec(),
            SlopeType::SmoothHillPart1 => SLOPE_SMOOTH_HILL_PART1.to_vec(),
            SlopeType::SmoothHillPart2 => SLOPE_SMOOTH_HILL_PART2.to_vec(),
            SlopeType::SmootherHillPart1 => SLOPE_SMOOTHER_HILL_PART1.to_vec(),
            SlopeType::SmootherHillPart2 => SLOPE_SMOOTHER_HILL_PART2.to_vec(),
            SlopeType::SmootherHillPart3 => SLOPE_SMOOTHER_HILL_PART3.to_vec(),
            SlopeType::SteepHillPart1 => SLOPE_STEEP_HILL_PART1.to_vec(),
            SlopeType::SteepHillPart2 => SLOPE_STEEP_HILL_PART2.to_vec(),
            _ => Vec::new(),
        };
        Polygon::new(points)
    }
}
