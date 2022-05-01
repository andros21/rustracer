//! Image Tracer module.
//!
//! Provides different renderers that implement [`Solve`] trait.
use crate::color::Color;
use crate::ray::Ray;
use crate::world::World;

/// A trait for solving rendering equation.
///
/// Must accept a [`Ray`] as its only parameter and must return a [`Color`] instance telling the
/// color to assign to a pixel in the image.
pub trait Solve {
    fn solve(&self, ray: Ray) -> Color;
}

pub struct OnOffRenderer<'a> {
    world: &'a World,
    background_color: Color,
    color: Color,
}

impl<'a> OnOffRenderer<'a> {
    pub fn new(world: &World, background_color: Color, color: Color) -> OnOffRenderer {
        OnOffRenderer {
            world,
            background_color,
            color,
        }
    }
}
impl<'a> Solve for OnOffRenderer<'a> {
    fn solve(&self, ray: Ray) -> Color {
        match self.world.ray_intersection(ray) {
            Some(_hit) => self.color,
            None => self.background_color,
        }
    }
}
