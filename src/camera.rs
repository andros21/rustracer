//! Camera module.
//!
//! Provides two camera structs:
//!  * [`OrthogonalCamera`](struct@OrthogonalCamera);
//!  * [`PerspectiveCamera`](struct@PerspectiveCamera).\
//!
//! That implement [`FireRay`] trait.
//!
//! And [`Camera`] enum that wrap them.
use crate::point::Point;
use crate::ray::Ray;
use crate::transformation::Transformation;
use crate::vector::Vector;

/// Trait for fire a [`Ray`] through a [`camera`](.).
///
/// You should redefine it in derived classes.\
/// Fire a ray that goes through the screen at the position `(u, v)`.\
/// The exact meaning of these coordinates depend on the projection used by the camera.
pub trait FireRay {
    fn fire_ray(&self, u: f32, v: f32) -> Ray;
}

/// A camera implementing an orthogonal 3D -> 2D projection.
///
/// This class implements an observer seeing the world through an orthogonal projection.
#[derive(Clone, Copy, Debug, Default)]
pub struct OrthogonalCamera {
    /// Aspect Ratio.
    aspect_ratio: f32,
    /// [`Transformation`] to apply to [`Ray`].
    tranformation: Transformation,
}

impl OrthogonalCamera {
    /// Create a new orthographic camera.
    ///
    /// The parameter `aspect_ratio` defines how larger than the height is the image.\
    /// For full screen images, you should probably set `aspect_ratio` to `16/9`,\
    /// as this is the most used aspect ratio used in modern monitors.
    ///
    /// The `transformation` parameter is an instance of the [`Transformation`].
    pub fn new(aspect_ratio: f32, tranformation: Transformation) -> OrthogonalCamera {
        OrthogonalCamera {
            aspect_ratio,
            tranformation,
        }
    }
}

impl FireRay for OrthogonalCamera {
    /// Shoot a [`Ray`] through the camera's screen.
    ///
    /// The coordinates `(u, v)` specify the point on the screen where the ray crosses it.\
    ///
    /// Coordinates:
    ///  * `(0, 0)` represent the bottom-left corner
    ///  * `(0, 1)` the top-left corner
    ///  * `(1, 0)` the bottom-right corner
    ///  * `(1, 1)` the top-right corner
    ///
    /// ```text
    ///  (0, 1)                          (1, 1)
    ///     +------------------------------+
    ///     |                              |
    ///     |                              |
    ///     |                              |
    ///     +------------------------------+
    ///  (0, 0)                          (1, 0)
    /// ```
    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        self.tranformation
            * Ray {
                origin: Point::from((-1.0, (1.0 - 2.0 * u) * self.aspect_ratio, 2.0 * v - 1.0)),
                dir: Vector::from((1.0, 0.0, 0.0)),
                ..Default::default()
            }
    }
}

/// A camera implementing a perspective 3D -> 2D projection.
///
/// This class implements an observer seeing the world through a perspective projection.
#[derive(Clone, Copy, Debug, Default)]
pub struct PerspectiveCamera {
    /// Screen distance.
    distance: f32,
    /// Aspect ratio.
    aspect_ratio: f32,
    /// [`Transformation`] to apply to [`Ray`].
    transformation: Transformation,
}

impl PerspectiveCamera {
    /// Create a new perspective camera.
    ///
    /// The parameter `distance` tells how much far from the eye\
    /// of the observer is the screen, and it influences the so-called «aperture»\
    /// (the field-of-view angle along the horizontal direction).
    ///
    /// The parameter `aspect_ratio` defines how larger than the height is the image.\
    /// For fullscreen images, you should probably set `aspect_ratio` to `16/9`,\
    /// as this is the most used aspect ratio used in modern monitors.
    ///
    /// The `transformation` parameter is an instance of the [`Transformation`].
    pub fn new(
        distance: f32,
        aspect_ratio: f32,
        transformation: Transformation,
    ) -> PerspectiveCamera {
        PerspectiveCamera {
            distance,
            aspect_ratio,
            transformation,
        }
    }
}

impl FireRay for PerspectiveCamera {
    /// Shoot a [`Ray`] through the camera's screen.
    ///
    /// The coordinates `(u, v)` specify the point on the screen where the ray crosses it.\
    ///
    /// Coordinates:
    ///  * `(0, 0)` represent the bottom-left corner
    ///  * `(0, 1)` the top-left corner
    ///  * `(1, 0)` the bottom-right corner
    ///  * `(1, 1)` the top-right corner
    ///
    /// ```text
    ///  (0, 1)                          (1, 1)
    ///     +------------------------------+
    ///     |                              |
    ///     |                              |
    ///     |                              |
    ///     +------------------------------+
    ///  (0, 0)                          (1, 0)
    /// ```
    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        self.transformation
            * Ray {
                origin: Point::from((-self.distance, 0.0, 0.0)),
                dir: Vector::from((
                    self.distance,
                    (1.0 - 2.0 * u) * self.aspect_ratio,
                    2.0 * v - 1.0,
                )),
                ..Default::default()
            }
    }
}

/// Enum of cameras.
pub enum Camera {
    Orthogonal(OrthogonalCamera),
    Perspective(PerspectiveCamera),
}

impl FireRay for Camera {
    /// Shoot a [`Ray`] through the camera's screen as the variant that [`Camera`] contain will do.
    fn fire_ray(&self, u: f32, v: f32) -> Ray {
        match self {
            Camera::Orthogonal(orthogonal) => orthogonal.fire_ray(u, v),
            Camera::Perspective(perspective) => perspective.fire_ray(u, v),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::misc::IsClose;

    #[test]
    fn test_orthogonal_camera() {
        let cam = Camera::Orthogonal(OrthogonalCamera::new(2.0, Transformation::default()));
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
        let cam = Camera::Perspective(PerspectiveCamera::new(1.0, 2.0, Transformation::default()));
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
