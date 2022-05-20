//! Render module.
//!
//! Provides different renderers that implement [`Solve`] trait.
use crate::color::Color;
use crate::material::{Eval, GetColor, ScatterRay};
use crate::random::Pcg;
use crate::ray::Ray;
use crate::world::World;

/// A trait for solving rendering equation.
///
/// Must accept a [`Ray`] as its only parameter and must return a [`Color`] instance telling the
/// color to assign to a pixel in the image.
pub trait Solve {
    fn solve(&mut self, ray: Ray) -> Color;
}

/// A on/off renderer.
///
/// This renderer is mostly useful for debugging purposes,
/// as it is really fast, but it produces boring images.
pub struct OnOffRenderer<'a, B, P>
where
    B: ScatterRay + Eval + GetColor + Clone,
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
    B: ScatterRay + Eval + GetColor + Clone,
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
    B: ScatterRay + Eval + GetColor + Clone,
    P: GetColor + Clone,
{
    /// Solve rendering with on/off strategy.
    ///
    /// If intersection happens return `fg_color` otherwise `bg_color`.
    fn solve(&mut self, ray: Ray) -> Color {
        match self.world.ray_intersection(ray) {
            Some(_hit) => self.fg_color,
            None => self.bg_color,
        }
    }
}

/// A path tracing renderer,
///
/// it resolves the rendering equations by means
/// of a Monte Carlo numeric integration algorithm.
pub struct PathTracer<'a, B, P>
where
    B: ScatterRay + Eval + GetColor + Clone,
    P: GetColor + Clone,
{
    /// A world instance.
    world: &'a World<B, P>,
    /// Background color (usually [`BLACK`](../color/constant.BLACK.html)).
    bg_color: Color,
    /// Random number generator used by [`ScatterRay`](http://localhost:8080/material/trait.ScatterRay.html)
    pcg: Pcg,
    /// Number of scattered rays after every impact.
    num_of_rays: u32,
    /// Maximum depth of scattered rays,
    /// this should always be infinite if not for debugging purposes.
    max_depth: u32,
    /// After this level of depth the russian roulette algorithm came into play
    /// to eventually stop the rendering.
    russian_roulette_limit: u32,
}

impl<'a, B, P> PathTracer<'a, B, P>
where
    B: ScatterRay + Eval + GetColor + Clone,
    P: GetColor + Clone,
{
    /// Create a new [`PathTracer`] renderer.
    pub fn new(
        world: &World<B, P>,
        bg_color: Color,
        pcg: Pcg,
        num_of_rays: u32,
        max_depth: u32,
        russian_roulette_limit: u32,
    ) -> PathTracer<B, P> {
        PathTracer {
            world,
            bg_color,
            pcg,
            num_of_rays,
            max_depth,
            russian_roulette_limit,
        }
    }
}

impl<'a, B, P> Solve for PathTracer<'a, B, P>
where
    B: ScatterRay + Eval + GetColor + Clone,
    P: GetColor + Clone,
{
    /// Solve the rendering equation using a path tracing algorithm.
    ///
    /// The algorithm implemented here allows the caller to tune number of
    /// rays thrown at each iteration, as well as the maximum depth.
    /// It implements Russian roulette, so in principle it will take a finite
    /// time to complete the calculation even if you set `max_depth` to infinity.
    fn solve(&mut self, ray: Ray) -> Color {
        if ray.depth > self.max_depth {
            return Color::default();
        }
        let hit_record = self.world.ray_intersection(ray);
        if hit_record.is_none() {
            return self.bg_color;
        }
        let hit = hit_record.unwrap();
        let hit_material = hit.material;
        let mut hit_color = hit_material.brdf.get_color(hit.surface_point);
        let emitted_radiance = hit_material.emitted_radiance.get_color(hit.surface_point);
        let hit_color_lum = hit_color.r.max(hit_color.g.max(hit_color.b));
        if ray.depth >= self.russian_roulette_limit {
            let q = (1. - hit_color_lum).max(0.05);
            if self.pcg.random_float() > q {
                hit_color = hit_color * (1.0 / (1. - q));
            } else {
                return emitted_radiance;
            }
        }
        let mut cum_radiance = Color::default();
        if hit_color_lum > 0. {
            for _ in 0..self.num_of_rays {
                let new_ray = hit_material.brdf.scatter_ray(
                    &mut self.pcg,
                    hit.ray.dir,
                    hit.world_point,
                    hit.normal,
                    ray.depth + 1,
                );
                let new_radiance = Self::solve(self, new_ray);
                cum_radiance = cum_radiance + (hit_color * new_radiance);
            }
        }
        emitted_radiance + cum_radiance * (1. / (self.num_of_rays as f32))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::material::{DiffuseBRDF, UniformPigment};
    use crate::misc::IsClose;
    use crate::transformation::Transformation;
    use crate::{Material, Sphere, WHITE};

    #[test]
    fn test_furnace() {
        let mut pcg = Pcg::default();
        for _ in 0..10 {
            let emitted_radiance = pcg.random_float();
            let reflectance = pcg.random_float() * 0.9;
            let furnace_material = Material {
                brdf: DiffuseBRDF {
                    pigment: UniformPigment {
                        color: WHITE * reflectance,
                    },
                },
                emitted_radiance: UniformPigment {
                    color: WHITE * emitted_radiance,
                },
            };
            let furnace = Sphere::new(Transformation::default(), furnace_material);
            let mut world = World::default();
            world.add(Box::new(furnace));
            let mut path_tracer = PathTracer {
                pcg,
                num_of_rays: 1,
                world: &world,
                max_depth: 100,
                russian_roulette_limit: 101,
                bg_color: Default::default(),
            };
            let color = path_tracer.solve(Ray::default());
            let expected = emitted_radiance / (1. - reflectance);
            assert!(expected.is_close(color.r));
            assert!(expected.is_close(color.g));
            assert!(expected.is_close(color.b));
        }
    }
}
