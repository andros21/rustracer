//! Render module.
//!
//! Provides different renderers that implement [`Solve`] trait.
use crate::color::Color;
use crate::material::{GetColor, ScatterRay};
use crate::random::Pcg;
use crate::ray::Ray;
use crate::world::World;

/// A trait for solving rendering equation.
///
/// Must accept a [`Ray`] and a [`Pcg`], and must return a [`Color`] instance telling the
/// color to assign to a pixel in the image.
///
/// **Note:** [`Pcg`] parameter will be used only with [`Renderer::PathTracer`].
pub trait Solve {
    fn solve(&self, ray: Ray, pcg: &mut Pcg) -> Color;
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
    pub fn new(world: &'a World, bg_color: Color, fg_color: Color) -> Self {
        Self {
            world,
            bg_color,
            fg_color,
        }
    }
}

/// A flat renderer.
///
/// This renderer is mostly useful for debugging purposes,
/// as it is fast, but it does not take into account light reflection and shadows.
pub struct FlatRenderer<'a> {
    /// A world instance.
    world: &'a World,
    /// Background color (usually [`BLACK`](../color/constant.BLACK.html)).
    bg_color: Color,
}

impl<'a> Solve for OnOffRenderer<'a> {
    /// Solve rendering with on/off strategy.
    ///
    /// If intersection happens return `fg_color` otherwise `bg_color`.
    fn solve(&self, ray: Ray, _pcg: &mut Pcg) -> Color {
        match self.world.ray_intersection(ray) {
            Some(_hit) => self.fg_color,
            None => self.bg_color,
        }
    }
}

impl<'a> FlatRenderer<'a> {
    /// Create a new [`FlatRenderer`] renderer.
    pub fn new(world: &'a World, bg_color: Color) -> Self {
        Self { world, bg_color }
    }
}

impl<'a> Solve for FlatRenderer<'a> {
    /// Solve rendering with flat colors.
    ///
    /// If intersection happens return the color of the hit shape, otherwise `bg_color`.
    fn solve(&self, ray: Ray, _pcg: &mut Pcg) -> Color {
        match self.world.ray_intersection(ray) {
            Some(hit) => {
                hit.material.emitted_radiance.get_color(hit.surface_point)
                    + hit.material.brdf.get_color(hit.surface_point)
            },
            None => self.bg_color,
        }
    }
}

/// A path tracing renderer.
///
/// It resolves the rendering equations by means
/// of a Monte Carlo numeric integration algorithm.
pub struct PathTracer<'a> {
    /// A world instance.
    world: &'a World,
    /// Background color (usually [`BLACK`](../color/constant.BLACK.html)).
    bg_color: Color,
    /// Number of scattered rays after every impact.
    num_of_rays: u32,
    /// Maximum depth of scattered rays,
    /// this should always be infinite if not for debugging purposes.
    max_depth: u32,
    /// After this level of depth the russian roulette algorithm came into play
    /// to eventually stop the rendering.
    russian_roulette_limit: u32,
}

impl<'a> PathTracer<'a> {
    /// Create a new [`PathTracer`] renderer.
    pub fn new(
        world: &'a World,
        bg_color: Color,
        num_of_rays: u32,
        max_depth: u32,
        russian_roulette_limit: u32,
    ) -> Self {
        PathTracer {
            world,
            bg_color,
            num_of_rays,
            max_depth,
            russian_roulette_limit,
        }
    }
}

impl<'a> Solve for PathTracer<'a> {
    /// Solve the rendering equation using a path tracing algorithm.
    ///
    /// The algorithm implemented here allows the caller to tune number of\
    /// rays thrown at each iteration,as well as the maximum depth.
    ///
    /// It implements Russian roulette, to avoid artefacts and speed up computation.
    fn solve(&self, ray: Ray, pcg: &mut Pcg) -> Color {
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
            if pcg.random_float() > q {
                hit_color = hit_color * (1.0 / (1. - q));
            } else {
                return emitted_radiance;
            }
        }
        let mut cum_radiance = Color::default();
        if hit_color_lum > 0. {
            for _ in 0..self.num_of_rays {
                let new_ray = hit_material.brdf.scatter_ray(
                    (pcg.random_float(), pcg.random_float()),
                    hit.ray.dir,
                    hit.world_point,
                    hit.normal,
                    ray.depth + 1,
                );
                let new_radiance = Self::solve(self, new_ray, pcg);
                cum_radiance = cum_radiance + (hit_color * new_radiance);
            }
        }
        emitted_radiance + cum_radiance * (1. / (self.num_of_rays as f32))
    }
}

/// A dummy renderer.
///
/// Test purpose and little else.
pub struct DummyRenderer;

impl Solve for DummyRenderer {
    /// Solve nothing! Only return a fixed [`Color`].
    fn solve(&self, _ray: Ray, _pcg: &mut Pcg) -> Color {
        Color::from((1.0, 2.0, 3.0))
    }
}

/// Enum of renderers.
pub enum Renderer<'a> {
    OnOff(OnOffRenderer<'a>),
    Dummy(DummyRenderer),
    PathTracer(PathTracer<'a>),
    Flat(FlatRenderer<'a>),
}

impl<'a> Solve for Renderer<'a> {
    /// Render the scene using a particular [`Renderer`] variants.
    fn solve(&self, ray: Ray, pcg: &mut Pcg) -> Color {
        match self {
            Renderer::OnOff(onoff) => onoff.solve(ray, pcg),
            Renderer::Dummy(dummy) => dummy.solve(ray, pcg),
            Renderer::PathTracer(pathtracer) => pathtracer.solve(ray, pcg),
            Renderer::Flat(flat) => flat.solve(ray, pcg),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::material::{DiffuseBRDF, Pigment, UniformPigment, BRDF};
    use crate::misc::IsClose;
    use crate::point::Point;
    use crate::transformation::Transformation;
    use crate::vector::E1;
    use crate::{translation, CheckeredPigment, Material, Sphere, BLACK, WHITE};

    #[test]
    fn test_flat() {
        let mut pcg = Pcg::default();
        let ray1 = Ray {
            origin: Point::from((-2., 3., 0.)),
            ..Default::default()
        };
        let ray_r = Ray {
            origin: Point::from((-2., 0.5, 0.5)),
            ..Default::default()
        };
        let ray_l = Ray {
            origin: Point::from((-2., -0.5, 0.5)),
            ..Default::default()
        };
        let red = Color::from((1., 0., 0.));
        let green = Color::from((0., 1., 0.));
        let blue = Color::from((0., 0., 1.));
        let mut world = World::default();
        let sphere_material = Material {
            brdf: BRDF::Diffuse(DiffuseBRDF {
                pigment: Pigment::Checkered(CheckeredPigment {
                    color1: red,
                    color2: blue,
                    steps: 2,
                }),
            }),
            emitted_radiance: Pigment::Uniform(UniformPigment { color: green }),
        };
        world.add(Box::new(Sphere::new(
            Transformation::default(),
            sphere_material,
        )));
        let flat_renderer = Renderer::Flat(FlatRenderer::new(&world, BLACK));
        assert!(flat_renderer.solve(ray1, &mut pcg).is_close(BLACK));
        assert!(flat_renderer.solve(ray_r, &mut pcg).is_close(red + green));
        assert!(flat_renderer.solve(ray_l, &mut pcg).is_close(blue + green));
    }

    #[test]
    fn test_onoff() {
        let mut pcg = Pcg::default();
        let ray1 = Ray {
            origin: Point::from((-2., 3., 0.)),
            ..Default::default()
        };
        let ray2 = Ray {
            origin: Point::from((-2., 0., 0.)),
            ..Default::default()
        };
        let mut world = World::default();
        world.add(Box::new(Sphere::default()));
        let onoff_renderer = Renderer::OnOff(OnOffRenderer::new(&world, BLACK, WHITE));
        assert!(onoff_renderer.solve(ray1, &mut pcg).is_close(BLACK));
        assert!(onoff_renderer.solve(ray2, &mut pcg).is_close(WHITE))
    }

    #[test]
    fn test_furnace() {
        let mut pcg = Pcg::default();
        for _ in 0..10 {
            let emitted_radiance = pcg.random_float();
            let reflectance = pcg.random_float() * 0.9;
            let furnace_material = Material {
                brdf: BRDF::Diffuse(DiffuseBRDF {
                    pigment: Pigment::Uniform(UniformPigment {
                        color: WHITE * reflectance,
                    }),
                }),
                emitted_radiance: Pigment::Uniform(UniformPigment {
                    color: WHITE * emitted_radiance,
                }),
            };
            let furnace = Sphere::new(Transformation::default(), furnace_material);
            let mut world = World::default();
            world.add(Box::new(furnace));
            let path_tracer = Renderer::PathTracer(PathTracer::new(&world, BLACK, 1, 100, 101));
            let color = path_tracer.solve(Ray::default(), &mut pcg);
            let expected = emitted_radiance / (1. - reflectance);
            assert!(expected.is_close(color.r));
            assert!(expected.is_close(color.g));
            assert!(expected.is_close(color.b));
        }
    }

    #[test]
    fn test_background() {
        let mut pcg = Pcg::default();
        let sphere = Sphere::new(translation(E1 * 2.), Material::default());
        let mut world = World::default();
        world.add(Box::new(sphere));
        let path_tracer = Renderer::PathTracer(PathTracer::new(&world, BLACK, 1000, 1000, 0));
        assert!(path_tracer.solve(Ray::default(), &mut pcg).is_close(BLACK))
    }
}
