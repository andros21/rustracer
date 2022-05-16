//! Render module.
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

/// A on/off renderer.
///
/// This renderer is mostly useful for debugging purposes,
/// as it is really fast, but it produces boring images.
pub struct OnOffRenderer<'a> {
    /// A world instance.
    world: &'a World,
    /// Background color (usually [`BLACK`](../color/constant.BLACK.html)).
    bg_color: Color,
    /// Foreground color (usually [`WHITE`](../color/constant.WHITE.html)).
    fg_color: Color,
}

impl<'a> OnOffRenderer<'a> {
    /// Create a new [`OnOffRenderer`] renderer.
    pub fn new(world: &World, bg_color: Color, fg_color: Color) -> OnOffRenderer {
        OnOffRenderer {
            world,
            bg_color,
            fg_color,
        }
    }
}

impl<'a> Solve for OnOffRenderer<'a> {
    /// Solve rendering with on/off strategy.
    ///
    /// If intersection happens return `fg_color` otherwise `bg_color`.
    fn solve(&self, ray: Ray) -> Color {
        match self.world.ray_intersection(ray) {
            Some(_hit) => self.fg_color,
            None => self.bg_color,
        }
    }
}
