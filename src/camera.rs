use crate::point::Point;
use crate::ray::Ray;
use crate::vector::Vector;

trait FireRay {
    fn fire_ray(&self, u: f32, v: f32) -> Ray;
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct OrthogonalCamera {
    aspect_ratio: f32,
}

impl FireRay for OrthogonalCamera {
    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: Point::from((-1.0, (1.0 - 2.0 * u) * self.aspect_ratio, 2.0 * v - 1.0)),
            dir: Vector::from((1.0, 0.0, 0.0)),
            tmin: 1e-5,
            tmax: f32::INFINITY,
            depth: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PerspectiveCamera {
    distance: f32,
    aspect_ratio: f32,
}

impl FireRay for PerspectiveCamera {
    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: Point::from((-self.distance, 0.0, 0.0)),
            dir: Vector::from((
                self.distance,
                (1.0 - 2.0 * u) * self.aspect_ratio,
                2.0 * v - 1.0,
            )),
            tmin: 1e-5,
            tmax: f32::INFINITY,
            depth: 0,
        }
    }
}

pub enum Camera {
    Orthogonal(OrthogonalCamera),
    Perspective(PerspectiveCamera),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::misc::IsClose;

    #[test]
    fn test_orthogonal_camera() {
        let cam = OrthogonalCamera { aspect_ratio: 2.0 };
        let ray1 = cam.fire_ray(0.0, 0.0);
        let ray2 = cam.fire_ray(1.0, 0.0);
        let ray3 = cam.fire_ray(0.0, 1.0);
        let ray4 = cam.fire_ray(1.0, 1.0);

        assert!((ray1.dir * ray2.dir).squared_norm().is_close(0.0));
        assert!((ray1.dir * ray3.dir).squared_norm().is_close(0.0));
        assert!((ray1.dir * ray4.dir).squared_norm().is_close(0.0));

        assert!(ray1.at(1.0).is_close(Point::from((0.0, 2.0, -1.0))));
        assert!(ray2.at(1.0).is_close(Point::from((0.0, -2.0, -1.0))));
        assert!(ray3.at(1.0).is_close(Point::from((0.0, 2.0, 1.0))));
        assert!(ray4.at(1.0).is_close(Point::from((0.0, -2.0, 1.0))));
    }

    #[test]
    fn test_perspective_camera() {
        let cam = PerspectiveCamera {
            distance: 1.0,
            aspect_ratio: 2.0,
        };
        let ray1 = cam.fire_ray(0.0, 0.0);
        let ray2 = cam.fire_ray(1.0, 0.0);
        let ray3 = cam.fire_ray(0.0, 1.0);
        let ray4 = cam.fire_ray(1.0, 1.0);

        assert!(ray1.origin.is_close(ray2.origin));
        assert!(ray1.origin.is_close(ray3.origin));
        assert!(ray1.origin.is_close(ray4.origin));

        assert!(ray1.at(1.0).is_close(Point::from((0.0, 2.0, -1.0))));
        assert!(ray2.at(1.0).is_close(Point::from((0.0, -2.0, -1.0))));
        assert!(ray3.at(1.0).is_close(Point::from((0.0, 2.0, 1.0))));
        assert!(ray4.at(1.0).is_close(Point::from((0.0, -2.0, 1.0))));
    }
}
