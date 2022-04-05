use crate::color::IsClose;
use crate::error::GeometryErr;
use crate::normal::Normal;
use crate::point::Point;
use crate::vector::Vector;
use std::ops::Mul;

pub const IDENTITY_MATRIX: [[f32; 4]; 4] = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];

#[derive(Clone, Debug, Default, PartialEq)]
struct Matrix {
    elements: [[f32; 4]; 4],
}

impl Matrix {
    pub fn new() -> Matrix {
        Matrix {
            elements: IDENTITY_MATRIX,
        }
    }
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.elements[index.0][index.1]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.elements[index.0][index.1]
    }
}

impl IsClose for Matrix {
    fn is_close(&self, other: Matrix) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if self.elements[i][j].is_close(other.elements[i][j]) {
                } else {
                    return false;
                }
            }
        }
        return true;
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Self::Output {
        let mut matrix = Matrix {
            elements: [[0.; 4]; 4],
        };
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    matrix[(i, j)] += self[(i, k)] * rhs[(k, j)]
                }
            }
        }
        return matrix;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transformation {
    m: Matrix,
    invm: Matrix,
}

impl Transformation {
    pub fn new() -> Transformation {
        Transformation {
            m: Matrix::new(),
            invm: Matrix::new(),
        }
    }

    fn is_consistent(self) -> bool {
        (self.m * self.invm).is_close(Matrix {
            elements: IDENTITY_MATRIX,
        })
    }
}

impl Mul<Transformation> for Transformation {
    type Output = Transformation;

    fn mul(self, rhs: Transformation) -> Self::Output {
        Transformation {
            m: self.m * rhs.m,
            invm: rhs.invm * self.invm,
        }
    }
}

pub fn traslation(vec: Vector) -> Transformation {
    Transformation {
        m: Matrix {
            elements: [
                [1., 0., 0., vec.x],
                [0., 1., 0., vec.y],
                [0., 0., 1., vec.z],
                [0., 0., 0., 1.],
            ],
        },
        invm: Matrix {
            elements: [
                [1., 0., 0., -vec.x],
                [0., 1., 0., -vec.y],
                [0., 0., 1., -vec.z],
                [0., 0., 0., 1.],
            ],
        },
    }
}

pub fn rotation_x(theta: f32) -> Transformation {
    Transformation {
        m: Matrix {
            elements: [
                [1., 0., 0., 0.],
                [0., theta.cos(), -theta.sin(), 0.],
                [0., theta.sin(), theta.cos(), 0.],
                [0., 0., 0., 1.],
            ],
        },
        invm: Matrix {
            elements: [
                [1., 0., 0., 0.],
                [0., theta.cos(), theta.sin(), 0.],
                [0., -theta.sin(), theta.cos(), 0.],
                [0., 0., 0., 1.],
            ],
        },
    }
}

pub fn rotation_y(theta: f32) -> Transformation {
    Transformation {
        m: Matrix {
            elements: [
                [theta.cos(), 0., theta.sin(), 0.],
                [0., 1., 0., 0.],
                [-theta.sin(), 0., theta.cos(), 0.],
                [0., 0., 0., 1.],
            ],
        },
        invm: Matrix {
            elements: [
                [theta.cos(), 0., -theta.sin(), 0.],
                [0., 1., 0., 0.],
                [theta.sin(), 0., theta.cos(), 0.],
                [0., 0., 0., 1.],
            ],
        },
    }
}

pub fn rotation_z(theta: f32) -> Transformation {
    Transformation {
        m: Matrix {
            elements: [
                [theta.cos(), -theta.sin(), 0., 0.],
                [theta.sin(), theta.cos(), 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
        },
        invm: Matrix {
            elements: [
                [theta.cos(), theta.sin(), 0., 0.],
                [-theta.sin(), theta.cos(), 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
        },
    }
}

pub fn scale(scale_x: f32, scale_y: f32, scale_z: f32) -> Transformation {
    Transformation {
        m: Matrix {
            elements: [
                [scale_x, 0., 0., 0.],
                [0., scale_y, 0., 0.],
                [0., 0., scale_z, 0.],
                [0., 0., 0., 1.],
            ],
        },
        invm: Matrix {
            elements: [
                [1. / scale_x, 0., 0., 0.],
                [0., 1. / scale_y, 0., 0.],
                [0., 0., 1. / scale_z, 0.],
                [0., 0., 0., 1.],
            ],
        },
    }
}

impl Mul<Vector> for Transformation {
    type Output = Vector;

    fn mul(self, vector: Vector) -> Self::Output {
        Vector {
            x: self.m[(0, 0)] * vector.x + self.m[(0, 1)] * vector.y + self.m[(0, 2)] * vector.z,
            y: self.m[(1, 0)] * vector.x + self.m[(1, 1)] * vector.y + self.m[(1, 2)] * vector.z,
            z: self.m[(2, 0)] * vector.x + self.m[(2, 1)] * vector.y + self.m[(2, 2)] * vector.z,
        }
    }
}

impl Mul<Normal> for Transformation {
    type Output = Normal;

    fn mul(self, normal: Normal) -> Self::Output {
        Normal {
            x: self.invm[(0, 0)] * normal.x
                + self.invm[(1, 0)] * normal.y
                + self.invm[(2, 0)] * normal.z,
            y: self.invm[(0, 1)] * normal.x
                + self.invm[(1, 1)] * normal.y
                + self.invm[(2, 1)] * normal.z,
            z: self.invm[(0, 2)] * normal.x
                + self.invm[(1, 2)] * normal.y
                + self.invm[(2, 2)] * normal.z,
        }
    }
}

impl Mul<Point> for Transformation {
    type Output = Point;

    fn mul(self, point: Point) -> Self::Output {
        let new_point = Point {
            x: self.m[(0, 0)] * point.x
                + self.m[(0, 1)] * point.y
                + self.m[(0, 2)] * point.z
                + self.m[(0, 3)],
            y: self.m[(1, 0)] * point.x
                + self.m[(1, 1)] * point.y
                + self.m[(1, 2)] * point.z
                + self.m[(1, 3)],
            z: self.m[(2, 0)] * point.x
                + self.m[(2, 1)] * point.y
                + self.m[(2, 2)] * point.z
                + self.m[(2, 3)],
        };
        let w = point.x * self.m[(3, 0)]
            + point.y * self.m[(3, 1)]
            + point.z * self.m[(3, 2)]
            + self.m[(3, 3)];
        if w == 1. {
            new_point
        } else {
            Point {
                x: new_point.x / w,
                y: new_point.y / w,
                z: new_point.z / w,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::color::EPSILON;
    use crate::point;
    use std::f32::consts::PI;

    #[test]
    fn test_index() {
        let mut matrix = Matrix::new();
        for i in 0..4 {
            for j in 0..4 {
                if i == j {
                    assert_eq!(matrix[(i, j)], 1.)
                } else {
                    assert_eq!(matrix[(i, j)], 0.)
                }
            }
        }
        matrix[(1, 2)] = 3.;
        assert_eq!(matrix[(1, 2)], 3.)
    }

    #[test]
    fn test_is_close() {
        assert!(Matrix::new().is_close(Matrix {
            elements: [
                [1. + EPSILON / 2., 0., 0., 0.],
                [EPSILON / 2., 1., 0., 0.],
                [-EPSILON / 2., 0., 1. - EPSILON / 2., 0.],
                [0., 0., 0., 1. + EPSILON / 3.]
            ]
        }));
        assert!(
            !(Matrix::new().is_close(Matrix {
                elements: [
                    [1. + EPSILON, 0., 0., 0.],
                    [0., 1., 0., 0.],
                    [0., 0., 1., 0.],
                    [0., 0., 0., 1.]
                ]
            }))
        )
    }

    #[test]
    fn test_mul_matrix() {
        let m1 = Matrix {
            elements: [
                [1., 2., -1., 3.],
                [1., -2., 0., 2.],
                [1., -1., 0., 1.],
                [0., 3., -3., 1.],
            ],
        };
        let m2 = Matrix {
            elements: [
                [-2., 2., 1., 3.],
                [1., -1., 0., 1.],
                [1., 2., -1., 1.],
                [1., -2., 0., -3.],
            ],
        };
        assert_eq!(
            m1 * m2,
            Matrix {
                elements: [
                    [2., -8., 2., -5.],
                    [-2., 0., 1., -5.],
                    [-2., 1., 1., -1.],
                    [1., -11., 3., -3.]
                ]
            }
        )
    }

    #[test]
    fn test_is_consistent() {
        assert!(Transformation::new().is_consistent())
    }

    #[test]
    fn test_translation() {
        assert!(traslation(Vector::from((1.2, -7., 12.))).is_consistent())
    }

    #[test]
    fn test_rotations() {
        assert!(rotation_x(12.3).is_consistent());
        assert!(rotation_y(-0.14).is_consistent());
        assert!(rotation_z(1.).is_consistent());

        assert!(rotation_x(2. * PI).m.is_close(Matrix::new()));
        assert!(rotation_y(-4. * PI).invm.is_close(Matrix::new()));
        assert!(rotation_z(0.).m.is_close(Matrix::new()));
    }

    #[test]
    fn test_scale() {
        assert!(scale(2., -3., 2.2).is_consistent())
    }

    #[test]
    fn test_mul_vector() {
        assert!(
            (rotation_x(PI / 3.) * Vector::from((1., 1., 0.))).is_close(Vector::from((
                1.,
                0.5,
                3_f32.sqrt() / 2.
            )))
        );
    }

    #[test]
    fn test_mul_normal() {
        assert!(
            (scale(2., -3., 5.) * Normal::from((2., 1., 0.))).is_close(Normal::from((
                1.,
                -1. / 3.,
                0.
            )))
        )
    }

    #[test]
    fn test_mul_point() {
        assert!(
            (traslation(Vector::from((1., -2., 3.))) * Point::from((-3., 2., 0.)))
                .is_close(Point::from((-2., 0., 3.)))
        )
    }
}
