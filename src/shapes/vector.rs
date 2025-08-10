use super::point::Point;
pub mod constants;

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

    pub fn compare_direction(&self, other: &Vector) -> bool {
        let x1 = self.end.x - self.start.x;
        let y1 = self.end.y - self.start.y;
        let x2 = other.end.x - other.start.x;
        let y2 = other.end.y - other.start.y;

        if (y1 != 0 && y2 != 0) && x1 / y1 == x2 / y2 {
            return true;
        }

        // TODO: "support for y1 == 0 or y2 == 0"

        false
    }

    /// return 1 if the start of self is the same as the end of other.
    /// return -1 if the end of self is the same as the start of other.
    /// return 0 if both meet at the same point.
    pub fn end_points_meet(&self, other: &Vector) -> i8 {
        ((self.start == other.end) as i8) + -((self.end == other.start) as i8)
    }

    pub fn angle(&self, other: &Vector) -> f64 {
        let dot = self.dot(other);
        let len1 = self.length();
        let len2 = other.length();
        let cos_theta = dot as f64 / (len1 * len2);
        cos_theta.acos()
    }

    pub fn translate(self, x: i32, y: i32) -> Vector {
        Vector {
            start: Point {
                x: self.start.x + x,
                y: self.start.y + y,
            },
            end: Point {
                x: self.end.x + x,
                y: self.end.y + y,
            },
        }
    }
}
