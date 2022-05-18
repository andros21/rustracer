//! Render module.
//!
//! Provides different renderers that implement [`Solve`] trait.
use crate::color::Color;
use crate::material::{Eval, GetColor, ScatterRay};
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
pub struct OnOffRenderer<'a, B, P>
where
    B: ScatterRay + Eval + Clone,
    P: GetColor + Clone,
{
    /// A world instance.
    world: &'a World<B, P>,
    /// Background color (usually [`BLACK`](../color/constant.BLACK.html)).
    bg_color: Color,
    /// Foreground color (usually [`WHITE`](../color/constant.WHITE.html)).
    fg_color: Color,
}

impl<'a, B, P> OnOffRenderer<'a, B, P>
where
    B: ScatterRay + Eval + Clone,
    P: GetColor + Clone,
{
    /// Create a new [`OnOffRenderer`] renderer.
    pub fn new(world: &World<B, P>, bg_color: Color, fg_color: Color) -> OnOffRenderer<B, P> {
        OnOffRenderer {
            world,
            bg_color,
            fg_color,
        }
    }
}

impl<'a, B, P> Solve for OnOffRenderer<'a, B, P>
where
    B: ScatterRay + Eval + Clone,
    P: GetColor + Clone,
{
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
