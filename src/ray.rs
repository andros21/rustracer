//! Light Ray module.
//!
//! Provides [`Ray`](struct@Ray) struct.
use crate::misc::IsClose;
use crate::point::Point;
use crate::vector::Vector;

#[derive(Clone, Copy, Debug, PartialEq)]
/// Light Ray struct.
pub struct Ray {
    /// Origin [`Point`] of the light ray.
    pub origin: Point,
    /// [`Vector`] propagation direction.
    pub dir: Vector,
    /// Minimum time of propagation.
    pub tmin: f32,
    /// Maximum time of propagation.
    pub tmax: f32,
    /// Counts the number of reflection occurred.\
    /// If a ray is produced by a reflection,
    /// its `depth` is increased by 1 with respect to the original ray.
    pub depth: u32,
}

impl Ray {
    /// Return the position of the [`Ray`] at time `t`.
    /// The time is measured in units of `dir` vector's length.
    pub fn at(self, t: f32) -> Point {
        self.origin + self.dir * t
    }
}

impl IsClose for Ray {
    /// Return `true` if all the members of two [`Ray`] are [close][trait@IsClose]
    fn is_close(&self, other: Self) -> bool {
        self.origin.is_close(other.origin) && self.dir.is_close(other.dir)
    }
}

impl Default for Ray {
    /// Return as default ray a [`Ray`] with:
    ///
    /// * the origin of axis as `origin`
    /// * an x-axis oriented unit vector as `dir`
    /// * `tmin = 1e-5`
    /// * `tmax = `[`f32::INFINITY`]
    /// * `depth = 0`
    fn default() -> Self {
        Ray {
            origin: Point::default(),
            dir: Vector::from((1.0, 0.0, 0.0)),
            tmin: 1e-5,
            tmax: f32::INFINITY,
            depth: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_close() {
        let ray1 = Ray {
            origin: Point::from((1.0, 2.0, 3.0)),
            dir: Vector::from((5.0, 4.0, -1.0)),
            ..Default::default()
        };
        let ray2 = Ray {
            origin: Point::from((1.0, 2.0, 3.0)),
            dir: Vector::from((5.0, 4.0, -1.0)),
            ..Default::default()
        };
        let ray3 = Ray {
            origin: Point::from((5.0, 1.0, 4.0)),
            dir: Vector::from((3.0, 9.0, 4.0)),
            ..Default::default()
        };

        assert!(ray1.is_close(ray2));
        assert!(!ray1.is_close(ray3))
    }

    #[test]
    fn test_at() {
        let ray = Ray {
            origin: Point::from((1.0, 2.0, 4.0)),
            dir: Vector::from((4.0, 2.0, 1.0)),
            ..Default::default()
        };

        assert!(ray.at(0.0).is_close(ray.origin));
        assert!(ray.at(1.0).is_close(Point::from((5.0, 4.0, 5.0))));
        assert!(ray.at(2.0).is_close(Point::from((9.0, 6.0, 6.0))))
    }
}
