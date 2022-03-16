use std::ops::{Add, Mul};

const EPSILON: f32 = 1e-5;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<(f32, f32, f32)> for Color {
    fn from(rgb: (f32, f32, f32)) -> Self {
        Self {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
        }
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Color {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

pub trait IsClose<Rhs = Self> {
    fn is_close(self, other: Self) -> bool;
}

impl IsClose for f32 {
    fn is_close(self, other: f32) -> bool {
        (self - other).abs() < EPSILON
    }
}

impl IsClose for Color {
    fn is_close(self, other: Color) -> bool {
        self.r.is_close(other.r) & self.g.is_close(other.g) & self.b.is_close(other.b)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add() {
        assert_eq!(
            Color::from((1.0, 1.0, 1.0)) + Color::from((2.0, 2.0, 2.0)),
            Color::from((3.0, 3.0, 3.0))
        )
    }

    #[test]
    fn mul_color() {
        assert_eq!(
            Color::from((1.0, 1.0, 1.0)) * Color::from((2.0, 1.0, 2.0)),
            Color::from((2.0, 1.0, 2.0))
        )
    }

    #[test]
    fn mul_float() {
        assert_eq!(
            Color::from((1.0, 1.0, 1.0)) * 2.0,
            Color::from((2.0, 2.0, 2.0))
        )
    }

    #[test]
    fn is_close_float() {
        assert!((EPSILON * 1e-1 + 1.0).is_close(1.0))
    }

    #[test]
    fn neg_is_close_float() {
        assert!(!(EPSILON + 1.0).is_close(1.0))
    }

    #[test]
    fn is_close_color_0() {
        assert!(
            (Color::from((1.23, 4.56, 7.89)) * Color::from((9.87, 6.54, 3.21)))
                .is_close(Color::from((12.1401, 29.8224, 25.3269)))
        )
    }

    #[test]
    fn is_close_color_1() {
        assert!(
            (Color::from((1.0, 2.0, 3.0)) + Color::from((1.0, 2.0 + EPSILON * 1e-1, 3.0)))
                .is_close(Color::from((2.0, 4.0, 6.0)))
        )
    }
}
