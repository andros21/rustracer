use crate::misc::IsClose;
use crate::point::Point;
use crate::vector::Vector;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ray {
    origin: Point,
    dir: Vector,
    tmin: f32,
    tmax: f32,
    depth: u32,
}

impl Ray {
    pub fn at(&self, t: f32) -> Point {
        self.origin + self.dir * t
    }
}

impl IsClose for Ray {
    fn is_close(&self, other: Self) -> bool {
        self.origin.is_close(other.origin) && self.dir.is_close(other.dir)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_close() {
        let ray1 = Ray {
            origin: Point {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            dir: Vector {
                x: 5.0,
                y: 4.0,
                z: -1.0,
            },
            tmin: 1e-5,
            tmax: f32::INFINITY,
            depth: 0,
        };
        let ray2 = Ray {
            origin: Point {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            dir: Vector {
                x: 5.0,
                y: 4.0,
                z: -1.0,
            },
            tmin: 1e-5,
            tmax: f32::INFINITY,
            depth: 0,
        };
        let ray3 = Ray {
            origin: Point {
                x: 5.0,
                y: 1.0,
                z: 4.0,
            },
            dir: Vector {
                x: 3.0,
                y: 9.0,
                z: 4.0,
            },
            tmin: 1e-5,
            tmax: f32::INFINITY,
            depth: 0,
        };

        assert!(ray1.is_close(ray2));
        assert!(!ray1.is_close(ray3))
    }

    #[test]
    fn test_at() {
        let ray = Ray {
            origin: Point {
                x: 1.0,
                y: 2.0,
                z: 4.0,
            },
            dir: Vector {
                x: 4.0,
                y: 2.0,
                z: 1.0,
            },
            tmin: 1e-5,
            tmax: f32::INFINITY,
            depth: 0,
        };

        assert!(ray.at(0.0).is_close(ray.origin));
        assert!(
            (ray.at(1.0).is_close(Point {
                x: 5.0,
                y: 4.0,
                z: 5.0
            }))
        );
        assert!(
            (ray.at(2.0).is_close(Point {
                x: 9.0,
                y: 6.0,
                z: 6.0
            }))
        );
    }
}
