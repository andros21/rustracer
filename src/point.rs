use std::fmt;
use std::ops::{Add, Sub};
use crate::color::IsClose;
use crate::vector::Vector;


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point{
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl From<(f32, f32, f32)> for Point {
    fn from(xyz: (f32, f32, f32)) -> Self {
        Self {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl IsClose for Point {
    fn is_close(&self, other: Point) -> bool {
        self.x.is_close(other.x) & self.y.is_close(other.y) & self.z.is_close(other.z)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, other: Vector) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Point) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, other: Vector) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_add_vector() {
        assert_eq!(
            Point::from((1.0, 1.0, 1.0)) + Vector::from((2.0, 2.0, 2.0)),
            Point::from((3.0, 3.0, 3.0))
        )
    }

    fn test_sub_vector() {
        assert_eq!(
            Point::from((1.0, 2.0, 3.0)) - Vector::from((2.0, 2.0, 2.0)),
            Point::from((-1.0, 0.0, 1.0))
        )
    }

    #[test]
    fn test_sub_point() {
        assert_eq!(
            Point::from((1.0, 2.0, 3.0)) - Point::from((2.0, 2.0, 2.0)),
            Vector::from((-1.0, 0.0, 1.0))
        )
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", Point::from((1.02, -2.00, 1e7))),
            "Point(1.02, -2, 10000000)"
        );
    }
}
