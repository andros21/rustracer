use crate::misc::{IsClose, Vector2D};
use crate::normal::Normal;
use crate::point::Point;
use crate::ray::Ray;
use crate::transformation::Transformation;
use crate::vector::Vector;
use std::f32::consts::PI;

trait RayIntersection {
    fn ray_intersection(&self, ray: Ray) -> Option<HitRecord>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct HitRecord {
    world_point: Point,
    normal: Normal,
    surface_point: Vector2D,
    t: f32,
    ray: Ray,
}

impl IsClose for HitRecord {
    fn is_close(&self, other: Self) -> bool {
        self.world_point.is_close(other.world_point)
            && self.normal.is_close(other.normal)
            && self.surface_point.is_close(other.surface_point)
            && self.t.is_close(other.t)
            && self.ray.is_close(other.ray)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sphere {
    transformation: Transformation,
}

fn sphere_normal(point: Point, ray_dir: Vector) -> Normal {
    let result = Normal::from((point.x, point.y, point.z));
    if Vector::from(point).dot(ray_dir) < 0.0 {
        result
    } else {
        result * -1.
    }
}

fn sphere_point_to_uv(point: Point) -> Vector2D {
    let mut u = point.y.atan2(point.x) / (2.0 * PI);
    let v = point.z.acos() / PI;
    if u < 0.0 {
        u += 1.0
    };
    Vector2D { u, v }
}

impl RayIntersection for Sphere {
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plane {
    transformation: Transformation,
}

impl RayIntersection for Plane {
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

fn plane_normal(ray_dir: Vector) -> Normal {
    let normal = Normal::from((0., 0., 1.));
    if Vector::from(normal).dot(ray_dir) < 0.0 {
        normal
    } else {
        normal * -1.
    }
}

fn plane_point_to_uv(point: Point) -> Vector2D {
    Vector2D {
        u: point.x - point.x.floor(),
        v: point.y - point.y.floor(),
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
        assert!(intersection1.is_some());
        assert!(HitRecord {
            world_point: Point::from((0., 0., 1.)),
            normal: Normal::from((0., 0., 1.)),
            surface_point: Vector2D { u: 0., v: 0. },
            t: 1.,
            ray: ray1,
        }
        .is_close(intersection1.unwrap()));

        let ray2 = Ray {
            origin: Point::from((3., 0., 0.)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        let intersection2 = sphere.ray_intersection(ray2);
        assert!(intersection2.is_some());
        assert!(HitRecord {
            world_point: Point::from((1., 0., 0.)),
            normal: Normal::from((1., 0., 0.)),
            surface_point: Vector2D { u: 0., v: 0.5 },
            t: 2.,
            ray: ray2,
        }
        .is_close(intersection2.unwrap()));

        let ray3 = Ray {
            origin: Point::from((0., 0., 0.)),
            dir: Vector::from((1., 0., 0.)),
            ..Default::default()
        };
        let intersection3 = sphere.ray_intersection(ray3);
        assert!(intersection3.is_some());
        assert!(HitRecord {
            world_point: Point::from((1., 0., 0.)),
            normal: Normal::from((-1., 0., 0.)),
            surface_point: Vector2D { u: 0., v: 0.5 },
            t: 1.,
            ray: ray3,
        }
        .is_close(intersection3.unwrap()))
    }

    #[test]
    fn test_transform_sphere() {
        let sphere = Sphere {
            transformation: translation(Vector::from((10., 0., 0.))),
        };

        let ray1 = Ray {
            origin: Point::from((10., 0., 2.)),
            dir: Vector::from((0., 0., -1.)),
            ..Default::default()
        };
        let intersection1 = sphere.ray_intersection(ray1);
        assert!(intersection1.is_some());
        assert!(HitRecord {
            world_point: Point::from((10.0, 0.0, 1.0)),
            normal: Normal::from((0.0, 0.0, 1.0)),
            surface_point: Vector2D { u: 0., v: 0. },
            t: 1.0,
            ray: ray1,
        }
        .is_close(intersection1.unwrap()));
        let ray2 = Ray {
            origin: Point::from((13., 0., 0.)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        let intersection2 = sphere.ray_intersection(ray2);
        assert!(intersection2.is_some());
        assert!(HitRecord {
            world_point: Point::from((11.0, 0.0, 0.0)),
            normal: Normal::from((1.0, 0.0, 0.0)),
            surface_point: Vector2D { u: 0., v: 0.5 },
            t: 2.0,
            ray: ray2,
        }
        .is_close(intersection2.unwrap()));

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
        let sphere1 = Sphere {
            transformation: scaling(Vector::from((2., 1., 1.))),
        };
        let ray1 = Ray {
            origin: Point::from((1., 1., 0.)),
            dir: Vector::from((-1., -1., 0.)),
            ..Default::default()
        };
        let intersection1 = sphere1.ray_intersection(ray1);
        assert!(intersection1.is_some());
        assert!(intersection1
            .unwrap()
            .normal
            .normalize()
            .unwrap()
            .is_close(Normal::from((1., 4., 0.)).normalize().unwrap()));

        let sphere2 = Sphere {
            transformation: scaling(Vector::from((-1., -1., -1.))),
        };
        let ray2 = Ray {
            origin: Point::from((0., 2., 0.)),
            dir: Vector::from((0., -1., 0.)),
            ..Default::default()
        };
        let intersection2 = sphere2.ray_intersection(ray2);
        assert!(intersection2.is_some());
        assert!(intersection2
            .unwrap()
            .normal
            .normalize()
            .unwrap()
            .is_close(Normal::from((0., 1., 0.)).normalize().unwrap()))
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
        assert!(intersection1.is_some());
        assert!(HitRecord {
            world_point: Point::from((0., 0., 0.)),
            normal: Normal::from((0., 0., 1.)),
            surface_point: Vector2D { u: 0., v: 0. },
            t: 1.,
            ray: ray1,
        }
        .is_close(intersection1.unwrap()));
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
        let plane = Plane {
            transformation: rotation_y(PI / 2.),
        };
        let ray1 = Ray {
            origin: Point::from((1., 0., 0.)),
            dir: Vector::from((-1., 0., 0.)),
            ..Default::default()
        };
        let intersection1 = plane.ray_intersection(ray1);
        assert!(intersection1.is_some());
        assert!(HitRecord {
            world_point: Point::from((0., 0., 0.)),
            normal: Normal::from((1., 0., 0.)),
            surface_point: Vector2D { u: 0., v: 0. },
            t: 1.,
            ray: ray1,
        }
        .is_close(intersection1.unwrap()));
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
