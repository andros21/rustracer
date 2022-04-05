use crate::color::IsClose;
use crate::error::GeometryErr;
use crate::normal::Normal;
use crate::point::Point;
use std::fmt;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<(f32, f32, f32)> for Vector {
    fn from(xyz: (f32, f32, f32)) -> Self {
        Self {
            x: xyz.0,
            y: xyz.1,
            z: xyz.2,
        }
    }
}

impl From<Point> for Vector {
    fn from(point: Point) -> Self {
        Self {
            x: point.x,
            y: point.z,
            z: point.y,
        }
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector({}, {}, {})", self.x, self.y, self.z)
    }
}

impl IsClose for Vector {
    fn is_close(&self, other: Vector) -> bool {
        self.x.is_close(other.x) & self.y.is_close(other.y) & self.z.is_close(other.z)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Self::Output {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Self::Output {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

fn dot_product(lhs: Vector, rhs: Vector) -> f32 {
    lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
}

impl Mul<Vector> for Vector {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Vector {
    pub fn neg(&self) -> Vector {
        Vector {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
    pub fn squared_norm(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn norm(&self) -> f32 {
        f32::sqrt(self.squared_norm())
    }
    pub fn normalize(mut self) -> Result<(), GeometryErr> {
        if self.norm() > 0_f32 {
            self = self * (1_f32 / self.norm());
            Ok(())
        } else {
            Err(GeometryErr::UnableToNormalize(self.norm()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::color::EPSILON;

    #[test]
    fn test_is_close() {
        assert!(
            (Vector::from((1.23, 4.56, 7.89)) * Vector::from((9.87, 6.54, 3.21)))
                .is_close(Vector::from((-36.963, 73.926, -36.963)))
        );
        assert!(
            (Vector::from((1.0, 2.0, 3.0)) + Vector::from((1.0, 2.0 + EPSILON * 1e-1, 3.0)))
                .is_close(Vector::from((2.0, 4.0, 6.0)))
        )
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Vector::from((1.0, 1.0, 1.0)) + Vector::from((2.0, 2.0, 2.0)),
            Vector::from((3.0, 3.0, 3.0))
        )
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Vector::from((1.0, 2.0, 3.0)) - Vector::from((2.0, 2.0, 2.0)),
            Vector::from((-1.0, 0.0, 1.0))
        )
    }

    #[test]
    fn test_dot_product() {
        assert_eq!(
            dot_product(Vector::from((1.0, 1.0, 1.0)), Vector::from((2.0, 1.0, 2.0))),
            5.0
        )
    }

    #[test]
    fn test_cross_product() {
        assert_eq!(
            Vector::from((1.0, 1.0, 1.0)) * Vector::from((2.0, 1.0, 2.0)),
            Vector::from((1.0, 0.0, -1.0))
        )
    }

    #[test]
    fn test_mul_scalar() {
        assert_eq!(
            Vector::from((1.0, 1.0, 1.0)) * 2.0,
            Vector::from((2.0, 2.0, 2.0))
        )
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", Vector::from((1.02, -2.00, 1e7))),
            "Vector(1.02, -2, 10000000)"
        );
    }

    #[test]
    fn test_neg() {
        assert_eq!(
            Vector::from((0.0, 0.1, -2.0)).neg(),
            Vector::from((0.0, -0.1, 2.0))
        )
    }

    #[test]
    fn test_squared_norm() {
        assert_eq!(Vector::from((1.0, -2.0, 3.0)).squared_norm(), 14.0)
    }

    #[test]
    fn test_norm() {
        assert_eq!(Vector::from((0.0, -4.0, 3.0)).norm(), 5.0)
    }

    #[test]
    fn test_normalize() {
        let vector = Vector::from((-6. / 7., 2. / 7., -3. / 7.));
        assert!(matches!(
            Vector::from((-6.0, 2.0, -3.0)).normalize(), Ok(v) if v.is_close(vector)
        ));
        assert!(matches!(
            Vector::from((0.0, 0.0, 0.0)).normalize(), Err(GeometryErr::UnableToNormalize(a)) if a == 0_f32
        ))
    }
}
