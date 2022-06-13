//! World module.
//!
//! Provides [`World`](struct@World) struct.
use crate::ray::Ray;
use crate::shape::{HitRecord, RayIntersection};

/// A class holding a list of [`shapes`](../shape)
/// (e.g. [`Plane`](../shape/struct.Plane.html), [`Sphere`](../shape/struct.Sphere.html)).
///
/// You can add shapes to a world using [`add`](#method.add).\
/// Typically, you call [`ray_intersection`](#method.ray_intersection) to check whether
/// a light ray intersects any of the shapes in the world.
#[derive(Debug, Default)]
pub struct World {
    /// A [`std::vec::Vec`] of [`std::boxed::Box`]-ed [`shapes`](../shape)
    /// that implement [`RayIntersection`] trait
    /// ([vector of traits](https://doc.rust-lang.org/stable/book/ch17-02-trait-objects.html)).
    shapes: Vec<Box<dyn RayIntersection>>,
}

impl World {
    /// Append a new boxed shape to this [`World`].
    pub fn add(&mut self, shape: Box<dyn RayIntersection>) {
        self.shapes.push(shape);
    }

    /// Determine whether a ray intersects any of the objects in this [`World`].
    ///
    /// Return [`HitRecord`] wrapped inside [`std::option::Option`].
    pub fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let mut closest: Option<HitRecord> = None;
        for shape in self.shapes.iter() {
            let old_closest = closest;
            closest = match (old_closest, shape.ray_intersection(ray)) {
                (Some(closest_hit), Some(shape_hit)) => {
                    if shape_hit.t < closest_hit.t {
                        Some(shape_hit)
                    } else {
                        Some(closest_hit)
                    }
                }
                (None, Some(shape_hit)) => Some(shape_hit),
                (Some(closest_hit), None) => Some(closest_hit),
                _ => None,
            }
        }
        closest
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::material::GetColor;
    use crate::misc::IsClose;
    use crate::point::Point;
    use crate::vector::E1;
    use crate::{
        scaling, translation, DiffuseBRDF, Material, Pigment, Sphere, UniformPigment, Vector,
        BLACK, BRDF, WHITE,
    };

    #[test]
    fn test_world() {
        let mut world = World::default();
        world.add(Box::new(Sphere::default()));
        world.add(Box::new(Sphere::new(
            translation(E1 * 4.) * scaling(Vector::from((2., 2., 2.))),
            Material {
                brdf: BRDF::Diffuse(DiffuseBRDF::default()),
                emitted_radiance: Pigment::Uniform(UniformPigment { color: WHITE }),
            },
        )));
        let ray1 = Ray {
            origin: Point::from((-2., 3., 0.)),
            ..Default::default()
        };
        let ray2 = Ray {
            origin: Point::from((-2., 0., 0.)),
            ..Default::default()
        };
        let ray3 = Ray {
            origin: Point::from((-2., 1.5, 0.)),
            ..Default::default()
        };

        assert!(world.ray_intersection(ray1).is_none());
        assert!(
            matches!(world.ray_intersection(ray2), Some(hit) if hit.material.emitted_radiance.get_color(hit.surface_point).is_close(BLACK))
        );
        assert!(
            matches!(world.ray_intersection(ray3), Some(hit) if hit.material.emitted_radiance.get_color(hit.surface_point).is_close(WHITE))
        )
    }
}
