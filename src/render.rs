//! Image Tracer module.
//!
//! Provides different renderers that implement [`Solve`] trait.
use crate::color::Color;
use crate::ray::Ray;

/// A trait for solving rendering equation.
///
/// Must accept a [`Ray`] as its only parameter and must return a [`Color`] instance telling the
/// color to assign to a pixel in the image.
pub trait Solve {
    fn solve(&self, ray: Ray) -> Color;
}
