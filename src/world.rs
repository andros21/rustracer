//! World module.
//!
//! Provides [`World`](struct@World) struct.
use crate::material::{Eval, GetColor, ScatterRay};
use crate::ray::Ray;
use crate::shape::{HitRecord, RayIntersection};

/// A class holding a list of [`shapes`](../shape)
/// (e.g. [`Plane`](../shape/struct.Plane.html), [`Sphere`](../shape/struct.Sphere.html)).
///
/// You can add shapes to a world using [`add`](#method.add).\
/// Typically, you call [`ray_intersection`](#method.ray_intersection) to check whether
/// a light ray intersects any of the shapes in the world.
#[derive(Default)]
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
