//! RGB Color module.
//!
//! Provides [`Color`](struct@Color) struct.
use crate::misc::IsClose;
use std::ops::{Add, Mul};

pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
};
pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
};

/// RGB Color struct.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    /// Red.
    pub r: f32,
    /// Green.
    pub g: f32,
    /// Blue.
    pub b: f32,
}

impl Color {
    /// Return a rough measure of the luminosity associated with the color.
    pub fn luminosity(self) -> f32 {
        (self.into_iter().reduce(f32::max).unwrap() + self.into_iter().reduce(f32::min).unwrap())
            * 0.5
    }
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

    fn add(self, other: Color) -> Self::Output {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl IsClose for Color {
    /// Return `true` if the three RGB components of two [`Color`] are [close](trait@IsClose).
    fn is_close(&self, other: Color) -> bool {
        self.r.is_close(other.r) & self.g.is_close(other.g) & self.b.is_close(other.b)
    }
}

impl IntoIterator for Color {
    type Item = f32;
    type IntoIter = std::array::IntoIter<f32, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [self.r, self.g, self.b].into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::misc::EPSILON;

    #[test]
    fn test_add() {
        assert_eq!(
            Color::from((1.0, 1.0, 1.0)) + Color::from((2.0, 2.0, 2.0)),
            Color::from((3.0, 3.0, 3.0))
        )
    }

    #[test]
    fn test_mul_color() {
        assert_eq!(
            Color::from((1.0, 1.0, 1.0)) * Color::from((2.0, 1.0, 2.0)),
            Color::from((2.0, 1.0, 2.0))
        )
    }

    #[test]
    fn test_mul_float() {
        assert_eq!(
            Color::from((1.0, 1.0, 1.0)) * 2.0,
            Color::from((2.0, 2.0, 2.0))
        )
    }

    #[test]
    fn test_is_close_color() {
        assert!(
            (Color::from((1.23, 4.56, 7.89)) * Color::from((9.87, 6.54, 3.21)))
                .is_close(Color::from((12.1401, 29.8224, 25.3269)))
        );
        assert!(
            (Color::from((1.0, 2.0, 3.0)) + Color::from((1.0, 2.0 + EPSILON * 1e-1, 3.0)))
                .is_close(Color::from((2.0, 4.0, 6.0)))
        )
    }

    #[test]
    fn test_luminosity() {
        let col1 = Color::from((1.0, 2.0, 3.0));
        let col2 = Color::from((9.0, 5.0, 7.0));

        assert!(col1.luminosity().is_close(2.0));
        assert!(col2.luminosity().is_close(7.0))
    }
}
