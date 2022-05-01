use crate::ray::Ray;
use crate::shape::{HitRecord, RayIntersection};

#[derive(Default)]
pub struct World {
    shapes: Vec<Box<dyn RayIntersection>>,
}

impl World {
    pub fn add(&mut self, shape: Box<dyn RayIntersection>) {
        self.shapes.push(shape);
    }

    pub fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let mut closest: Option<HitRecord> = None;
        for shape in self.shapes.iter() {
            closest = match (closest, shape.ray_intersection(ray)) {
                (Some(closest_hit), Some(shape_hit)) => {
                    if shape_hit.t < closest_hit.t {
                        Some(shape_hit)
                    } else {
                        Some(closest_hit)
                    }
                }
                (None, Some(shape_hit)) => Some(shape_hit),
                _ => closest,
            }
        }
        closest
    }
}
