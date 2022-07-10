//! 3D Point module.
//!
//! Provides [`Point`](struct@Point) struct.
use crate::{misc::IsClose, vector::Vector};
use std::{
    fmt,
    ops::{Add, Mul, Sub},
};

/// 3D Point struct.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// x component.
    pub x: f32,
    /// y component.
    pub y: f32,
    /// z component.
    pub z: f32,
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
    /// Return `true` if the three xyz components of two [`Point`] are [close](trait@IsClose).
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

    fn add(self, other: Vector) -> Self::Output {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Point) -> Self::Output {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, other: Vector) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::misc::EPSILON;

    #[test]
    fn test_is_close() {
        assert!(
            Point::from((1.0, 2.0 + EPSILON * 1e-1, 3.0)).is_close(Point::from((1.0, 2.0, 3.0)))
        );
        assert!(!Point::from((2.0, 1.0 + EPSILON, 3.0)).is_close(Point::from((2.0, 1.0, 3.0))))
    }

    #[test]
    fn test_add_vector() {
        assert_eq!(
            Point::from((1.0, 1.0, 1.0)) + Vector::from((2.0, 2.0, 2.0)),
            Point::from((3.0, 3.0, 3.0))
        )
    }

    #[test]
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
    fn test_mul_scalar() {
        assert_eq!(
            Point::from((1.0, 1.0, 1.0)) * 2.0,
            Point::from((2.0, 2.0, 2.0))
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
