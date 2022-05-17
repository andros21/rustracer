//! Geometric Shapes module.
//!
//! Provides geometrical shape structs that implement the
//! [`RayIntersection`](trait@RayIntersection) trait.

use crate::misc::{IsClose, Vector2D};
use crate::normal::Normal;
use crate::point::Point;
use crate::ray::Ray;
use crate::transformation::Transformation;
use crate::vector::Vector;
use std::f32::consts::PI;

/// Trait to determine the intersections of an object with a [`Ray`](struct@Ray).
///
/// This trait is meant to be implemented by geometrical [shapes](#implementors) in order to
/// calculate how [light rays](struct@Ray) hit them.
pub trait RayIntersection {
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord>;
}

/// Struct used to store the results of [`RayIntersection`](trait@RayIntersection).
#[derive(Clone, Copy, Debug)]
pub struct HitRecord {
    /// Coordinates of the point of impact.
    pub world_point: Point,
    /// Normal of the shape surface on the impact point.
    pub normal: Normal,
    /// Coordinates of the point of impact in the frame of reference of the shape's surface.
    pub surface_point: Vector2D,
    /// Time the ray travelled before the impact.
    pub t: f32,
    /// The ray that impacted on the shape.
    pub ray: Ray,
}

impl IsClose for HitRecord {
    /// Return `true` if all the members of two [`HitRecord`](struct@HitRecord) are [close](trait@IsClose).
    fn is_close(&self, other: Self) -> bool {
        self.world_point.is_close(other.world_point)
            && self.normal.is_close(other.normal)
            && self.surface_point.is_close(other.surface_point)
            && self.t.is_close(other.t)
            && self.ray.is_close(other.ray)
    }
}

/// Geometrical shape corresponding to a sphere.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sphere {
    /// A generic sphere is defined by means of a
    /// [`Transformation`](struct@Transformation) on the
    /// unit sphere centered at the origin of axis.\
    /// This means that you can also get an ellipsis
    /// using the proper [`scaling`](fn@crate::transformation::scaling).
    transformation: Transformation,
}

impl Sphere {
    /// Provides a constructor for [`Sphere`](struct@Sphere).
    pub fn new(transformation: Transformation) -> Self {
        Sphere { transformation }
    }
}

/// Calculates normals to [`Sphere`](struct@Sphere)'s surface.
///
/// This Function is meant to be used inside [`Sphere`](struct@Sphere)'s
/// [`RayIntersection`](trait@RayIntersection) implementation.\
/// `ray_dir` is the direction of an impacting [`Ray`](struct@Ray) and\
/// is used to determine on which side of the surface the normal is calculated.
fn sphere_normal(point: Point, ray_dir: Vector) -> Normal {
    let result = Normal::from((point.x, point.y, point.z));
    if Vector::from(point).dot(ray_dir) < 0.0 {
        result
    } else {
        result.neg()
    }
}

/// Returns parametrization coordinates of a point on a sphere.
///
/// The sphere's surface is parametrized by two angles that correspond
/// to latitude and longitude.
fn sphere_point_to_uv(point: Point) -> Vector2D {
    let mut u = point.y.atan2(point.x) / (2.0 * PI);
    let v = point.z.acos() / PI;
    if u < 0.0 {
        u += 1.0
    };
    Vector2D { u, v }
}

impl RayIntersection for Sphere {
    /// Finds intersections between a [`Ray`](struct@Ray) and a [`Sphere`](struct@Sphere).
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let inv_ray = self.transformation.inverse() * ray;
        let origin_vec = Vector::from(inv_ray.origin);
        let a = inv_ray.dir.squared_norm();
        let b = 2.0 * origin_vec.dot(inv_ray.dir);
        let c = origin_vec.squared_norm() - 1.0;

        let delta = b * b - 4.0 * a * c;
        if delta <= 0.0 {
            return None;
        }
        let sqrt_delta = delta.sqrt();
        let t1 = (-b - sqrt_delta) / (2.0 * a);
        let t2 = (-b + sqrt_delta) / (2.0 * a);
        let first_hit_t;
        if (t1 > inv_ray.tmin) && (t1 < inv_ray.tmax) {
            first_hit_t = t1;
        } else if (t2 > inv_ray.tmin) && (t2 < inv_ray.tmax) {
            first_hit_t = t2;
        } else {
            return None;
        }
        let hit_point = inv_ray.at(first_hit_t);
        Some(HitRecord {
            world_point: self.transformation * hit_point,
            normal: self.transformation * sphere_normal(hit_point, inv_ray.dir),
            surface_point: sphere_point_to_uv(hit_point),
            t: first_hit_t,
            ray,
        })
    }
}

/// Geometrical shape corresponding to a plane.
#[derive(Clone, Copy, Debug, Default)]
pub struct Plane {
    /// A generic plane is defined by means of a [`Transformation`](struct@Transformation)
    /// on the X-Y plane.\
    /// A [`scaling`](fn@crate::transformation::scaling) transformation has the
    /// effect to change the sides length of the basic rectangle in the plane's
    /// [parametrization](fn@plane_point_to_uv).
    transformation: Transformation,
}

impl Plane {
    /// Provides a constructor for [`Plane`](struct@Plane).
    pub fn new(transformation: Transformation) -> Self {
        Plane { transformation }
    }
}

/// Calculates normals to [`Plane`](struct@Plane)'s surface.
///
/// This Function is meant to be used inside [`Plane`](struct@Plane)'s
/// [`RayIntersection`](trait@RayIntersection) implementation.\
/// `ray_dir` is the direction of an impacting [`Ray`](struct@Ray)
/// and is used to determine on which side of the surface the normal is calculated.
fn plane_normal(ray_dir: Vector) -> Normal {
    let normal = Normal::from((0., 0., 1.));
    if Vector::from(normal).dot(ray_dir) < 0.0 {
        normal
    } else {
        normal.neg()
    }
}

/// Returns parametrization coordinates of a point on a plane.
///
/// The plane is parametrized by the `[0,1]x[0,1]` square with periodic boundary conditions.
fn plane_point_to_uv(point: Point) -> Vector2D {
    Vector2D {
        u: point.x - point.x.floor(),
        v: point.y - point.y.floor(),
    }
}

impl RayIntersection for Plane {
    /// Finds intersections between a [`Ray`](struct@Ray) and a [`Plane`](struct@Plane).
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord> {
        let inv_ray = self.transformation.inverse() * ray;
        if inv_ray.dir.z.abs() < 1e-5 {
            return None;
        }
        let t = -inv_ray.origin.z / inv_ray.dir.z;
        if (t <= inv_ray.tmin) || (t >= inv_ray.tmax) {
            return None;
        }
        let hit_point = inv_ray.at(t);
        Some(HitRecord {
            world_point: self.transformation * hit_point,
            normal: self.transformation * plane_normal(inv_ray.dir),
            surface_point: plane_point_to_uv(hit_point),
            t,
            ray,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transformation::{rotation_y, scaling, translation};

    #[test]
    fn test_hit_sphere() {
        let sphere = Sphere::default();

        let ray0 = Ray {
            origin: Point::from((0., 10., 2.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        assert!(sphere.ray_intersection(ray0).is_none());

        let ray1 = Ray {
            origin: Point::from((0., 0., 2.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        let intersection1 = sphere.ray_intersection(ray1);

        assert!(
            matches!(intersection1, Some(intersection) if intersection.is_close(HitRecord {
                world_point: Point::from((0., 0., 1.)),
                normal: Normal::from((0., 0., 1.)),
                surface_point: Vector2D { u: 0., v: 0. },
                t: 1.,
                ray: ray1,
            }))
        );

        let ray2 = Ray {
            origin: Point::from((3., 0., 0.)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        let intersection2 = sphere.ray_intersection(ray2);
        assert!(
            matches!(intersection2, Some(intersection) if intersection.is_close(HitRecord {
                world_point: Point::from((1., 0., 0.)),
                normal: Normal::from((1., 0., 0.)),
                surface_point: Vector2D { u: 0., v: 0.5 },
                t: 2.,
                ray: ray2,
            }))
        );

        let ray3 = Ray {
            origin: Point::default(),
            dir: Vector::from((1., 0., 0.)),
            ..Default::default()
        };
        let intersection3 = sphere.ray_intersection(ray3);
        assert!(
            matches!(intersection3, Some(intersection) if intersection.is_close(HitRecord {
                world_point: Point::from((1., 0., 0.)),
                normal: Normal::from((-1., 0., 0.)),
                surface_point: Vector2D { u: 0., v: 0.5 },
                t: 1.,
                ray: ray3,
            }))
        )
    }

    #[test]
    fn test_transform_sphere() {
        let sphere = Sphere::new(translation(Vector::from((10., 0., 0.))));

        let ray1 = Ray {
            origin: Point::from((10., 0., 2.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        let intersection1 = sphere.ray_intersection(ray1);
        assert!(
            matches!(intersection1, Some(intersection) if intersection.is_close(HitRecord {
                world_point: Point::from((10.0, 0.0, 1.0)),
                normal: Normal::from((0.0, 0.0, 1.0)),
                surface_point: Vector2D { u: 0., v: 0. },
                t: 1.0,
                ray: ray1,
            }))
        );
        let ray2 = Ray {
            origin: Point::from((13., 0., 0.)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        let intersection2 = sphere.ray_intersection(ray2);
        assert!(
            matches!(intersection2, Some(intersection) if intersection.is_close(HitRecord {
                world_point: Point::from((11.0, 0.0, 0.0)),
                normal: Normal::from((1.0, 0.0, 0.0)),
                surface_point: Vector2D { u: 0., v: 0.5 },
                t: 2.0,
                ray: ray2,
            }))
        );

        let ray3 = Ray {
            origin: Point::from((0., 0., 2.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        assert!(sphere.ray_intersection(ray3).is_none());
        let ray4 = Ray {
            origin: Point::from((-10., 0., 0.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        assert!(sphere.ray_intersection(ray4).is_none());
    }

    #[test]
    fn test_sphere_normal() {
        let sphere1 = Sphere::new(scaling(Vector::from((2., 1., 1.))));
        let ray1 = Ray {
            origin: Point::from((1., 1., 0.)),
            dir: Vector::from((-1., -1., 0.)),
            ..Default::default()
        };
        let intersection1 = sphere1.ray_intersection(ray1);
        assert!(matches!(intersection1, Some(intersection) if intersection
            .normal
            .normalize()
            .unwrap()
            .is_close(Normal::from((1., 4., 0.)).normalize().unwrap())));

        let sphere2 = Sphere::new(scaling(Vector::from((-1., -1., -1.))));
        let ray2 = Ray {
            origin: Point::from((0., 2., 0.)),
            dir: Vector::from((0., -1., 0.)),
            ..Default::default()
        };
        let intersection2 = sphere2.ray_intersection(ray2);
        assert!(matches!(intersection2, Some(intersection) if intersection
            .normal
            .normalize()
            .unwrap()
            .is_close(Normal::from((0., 1., 0.)).normalize().unwrap())))
    }

    #[test]
    fn test_sphere_point_to_uv() {
        let sphere = Sphere::default();
        let ray1 = Ray {
            origin: Point::from((2., 0., 0.)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        assert!(sphere
            .ray_intersection(ray1)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0., v: 0.5 }));

        let ray2 = Ray {
            origin: Point::from((0., 2., 0.)),
            dir: Vector::from((0., -1., 0.)),
            ..Default::default()
        };
        assert!(sphere
            .ray_intersection(ray2)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0.25, v: 0.5 }));

        let ray3 = Ray {
            origin: Point::from((-2., 0., 0.)),
            dir: Vector::from((1., 0., 0.)),
            ..Default::default()
        };
        assert!(sphere
            .ray_intersection(ray3)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0.5, v: 0.5 }));

        let ray4 = Ray {
            origin: Point::from((0., -2., 0.)),
            dir: Vector::from((0., 1., 0.)),
            ..Default::default()
        };
        assert!(sphere
            .ray_intersection(ray4)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0.75, v: 0.5 }));

        let ray5 = Ray {
            origin: Point::from((2., 0., 0.5)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        assert!(sphere
            .ray_intersection(ray5)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0., v: 1. / 3. }));

        let ray6 = Ray {
            origin: Point::from((2., 0., -0.5)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        assert!(sphere
            .ray_intersection(ray6)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0., v: 2. / 3. }));
    }

    #[test]
    fn test_hit_plane() {
        let plane = Plane::default();
        let ray1 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        let intersection1 = plane.ray_intersection(ray1);
        assert!(
            matches!(intersection1, Some(intersection) if intersection.is_close(HitRecord {
                world_point: Point::default(),
                normal: Normal::from((0., 0., 1.)),
                surface_point: Vector2D { u: 0., v: 0. },
                t: 1.,
                ray: ray1,
            }))
        );
        let ray2 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((0., 0., 1.)),
            ..Default::default()
        };
        let intersection2 = plane.ray_intersection(ray2);
        assert!(intersection2.is_none());
        let ray3 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((1., 0., 0.)),
            ..Default::default()
        };
        let intersection3 = plane.ray_intersection(ray3);
        assert!(intersection3.is_none());
        let ray4 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((0., 1., 0.)),
            ..Default::default()
        };
        let intersection4 = plane.ray_intersection(ray4);
        assert!(intersection4.is_none());
    }

    #[test]
    fn test_transform_plane() {
        let plane = Plane::new(rotation_y(PI / 2.));
        let ray1 = Ray {
            origin: Point::from((1., 0., 0.)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        let intersection1 = plane.ray_intersection(ray1);
        assert!(
            matches!(intersection1, Some(intersection) if intersection.is_close(HitRecord {
                world_point: Point::default(),
                normal: Normal::from((1., 0., 0.)),
                surface_point: Vector2D { u: 0., v: 0. },
                t: 1.,
                ray: ray1,
            }))
        );
        let ray2 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((0., 0., 1.)),
            ..Default::default()
        };
        let intersection2 = plane.ray_intersection(ray2);
        assert!(intersection2.is_none());
        let ray3 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((1., 0., 0.)),
            ..Default::default()
        };
        let intersection3 = plane.ray_intersection(ray3);
        assert!(intersection3.is_none());
        let ray4 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((0., 1., 0.)),
            ..Default::default()
        };
        let intersection4 = plane.ray_intersection(ray4);
        assert!(intersection4.is_none());
    }

    #[test]
    fn test_plane_point_to_uv() {
        let plane = Plane::default();

        let ray1 = Ray {
            origin: Point::from((0., 0., 1.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        assert!(plane
            .ray_intersection(ray1)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0., v: 0.0 }));

        let ray2 = Ray {
            origin: Point::from((0.25, 0.75, 1.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        assert!(plane
            .ray_intersection(ray2)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0.25, v: 0.75 }));

        let ray3 = Ray {
            origin: Point::from((4.25, 7.75, 1.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        assert!(plane
            .ray_intersection(ray3)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0.25, v: 0.75 }));

        let ray4 = Ray {
            origin: Point::from((-4.25, -7.75, 1.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        assert!(plane
            .ray_intersection(ray4)
            .unwrap()
            .surface_point
            .is_close(Vector2D { u: 0.75, v: 0.25 }));
    }
}
