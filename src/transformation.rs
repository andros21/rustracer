use crate::misc::IsClose;
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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Matrix {
    elements: [[f32; 4]; 4],
}

impl Default for Matrix {
    fn default() -> Self {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transformation {
    m: Matrix,
    invm: Matrix,
}

impl Transformation {
    pub fn is_consistent(self) -> bool {
        (self.m * self.invm).is_close(Matrix {
            ..Default::default()
        })
    }
    pub fn inverse(self) -> Transformation {
        Transformation {
            m: self.invm,
            invm: self.m,
        }
    }
}

impl Default for Transformation {
    fn default() -> Self {
        Transformation {
            m: Default::default(),
            invm: Default::default(),
        }
    }
}

impl IsClose for Transformation {
    fn is_close(&self, other: Transformation) -> bool {
        self.m.is_close(other.m) && self.invm.is_close(other.invm)
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

pub fn translation(vec: Vector) -> Transformation {
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

pub fn scaling(vec: Vector) -> Transformation {
    Transformation {
        m: Matrix {
            elements: [
                [vec.x, 0., 0., 0.],
                [0., vec.y, 0., 0.],
                [0., 0., vec.z, 0.],
                [0., 0., 0., 1.],
            ],
        },
        invm: Matrix {
            elements: [
                [1. / vec.x, 0., 0., 0.],
                [0., 1. / vec.y, 0., 0.],
                [0., 0., 1. / vec.z, 0.],
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::misc::EPSILON;
    use std::f32::consts::PI;

    #[test]
    fn test_index() {
        let mut matrix = Matrix {
            ..Default::default()
        };

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
    fn test_is_close_matrix() {
        assert!(Matrix {
            ..Default::default()
        }
        .is_close(Matrix {
            elements: [
                [1. + EPSILON / 2., 0., 0., 0.],
                [EPSILON / 2., 1., 0., 0.],
                [-EPSILON / 2., 0., 1. - EPSILON / 2., 0.],
                [0., 0., 0., 1. + EPSILON / 3.]
            ]
        }));
        assert!(
            !(Matrix {
                ..Default::default()
            }
            .is_close(Matrix {
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
        assert!(Transformation {
            ..Default::default()
        }
        .is_consistent())
    }

    #[test]
    fn test_is_close() {
        let m1 = Transformation {
            m: Matrix {
                elements: [
                    [1.0, 2.0, 3.0, 4.0],
                    [5.0, 6.0, 7.0, 8.0],
                    [9.0, 9.0, 8.0, 7.0],
                    [6.0, 5.0, 4.0, 1.0],
                ],
            },
            invm: Matrix {
                elements: [
                    [-3.75, 2.75, -1.0, 0.0],
                    [4.375, -3.875, 2.0, -0.5],
                    [0.5, 0.5, -1.0, 1.0],
                    [-1.375, 0.875, 0.0, -0.5],
                ],
            },
        };
        assert!(m1.is_consistent());
        assert!(m1.is_close(m1));

        let mut m2 = m1;
        m2.m[(2, 2)] += 1.0;
        assert!(!m1.is_close(m2));

        let mut m3 = m1;
        m3.invm[(2, 2)] += 1.0;
        assert!(!m1.is_close(m3))
    }

    #[test]
    #[ignore]
    fn test_mul_transformation() {
        let m1 = Transformation {
            m: Matrix {
                elements: [
                    [1.0, 2.0, 3.0, 4.0],
                    [5.0, 6.0, 7.0, 8.0],
                    [9.0, 9.0, 8.0, 7.0],
                    [6.0, 5.0, 4.0, 1.0],
                ],
            },
            invm: Matrix {
                elements: [
                    [-3.75, 2.75, -1.0, 0.0],
                    [4.375, -3.875, 2.0, -0.5],
                    [0.5, 0.5, -1.0, 1.0],
                    [-1.375, 0.875, 0.0, -0.5],
                ],
            },
        };
        assert!(m1.is_consistent());

        let m2 = Transformation {
            m: Matrix {
                elements: [
                    [3.0, 5.0, 2.0, 4.0],
                    [4.0, 1.0, 0.0, 5.0],
                    [6.0, 3.0, 2.0, 0.0],
                    [1.0, 4.0, 2.0, 1.0],
                ],
            },
            invm: Matrix {
                elements: [
                    [0.4, -0.2, 0.2, -0.6],
                    [2.9, -1.7, 0.2, -3.1],
                    [-5.55, 3.15, -0.4, 6.45],
                    [-0.9, 0.7, -0.2, 1.1],
                ],
            },
        };
        assert!(m2.is_consistent());

        let expected = Transformation {
            m: Matrix {
                elements: [
                    [33.0, 32.0, 16.0, 18.0],
                    [89.0, 84.0, 40.0, 58.0],
                    [118.0, 106.0, 48.0, 88.0],
                    [63.0, 51.0, 22.0, 50.0],
                ],
            },
            invm: Matrix {
                elements: [
                    [-1.45, 1.45, -1.0, 0.6],
                    [-13.95, 11.95, -6.5, 2.6],
                    [25.525, -22.025, 12.25, -5.2],
                    [4.825, -4.325, 2.5, -1.1],
                ],
            },
        };
        assert!(expected.is_consistent());

        assert!(expected.is_close(m1 * m2))
    }

    #[test]
    fn test_inverse() {
        let m1 = Transformation {
            m: Matrix {
                elements: [
                    [1.0, 2.0, 3.0, 4.0],
                    [5.0, 6.0, 7.0, 8.0],
                    [9.0, 9.0, 8.0, 7.0],
                    [6.0, 5.0, 4.0, 1.0],
                ],
            },
            invm: Matrix {
                elements: [
                    [-3.75, 2.75, -1.0, 0.0],
                    [4.375, -3.875, 2.0, -0.5],
                    [0.5, 0.5, -1.0, 1.0],
                    [-1.375, 0.875, 0.0, -0.5],
                ],
            },
        };

        let m2 = m1.inverse();
        assert!(m2.is_consistent());

        let prod = m1 * m2;
        assert!(prod.is_consistent());
        assert!(prod.is_close(Transformation {
            ..Default::default()
        }))
    }

    #[test]
    fn test_translation() {
        let tr1 = translation(Vector::from((1.0, 2.0, 3.0)));
        assert!(tr1.is_consistent());

        let tr2 = translation(Vector::from((4.0, 6.0, 8.0)));
        assert!(tr1.is_consistent());

        let prod = tr1 * tr2;
        assert!(prod.is_consistent());

        let expected = translation(Vector::from((5.0, 8.0, 11.0)));
        assert!(prod.is_close(expected))
    }

    #[test]
    fn test_rotations() {
        assert!(rotation_x(12.3).is_consistent());
        assert!(rotation_y(-0.14).is_consistent());
        assert!(rotation_z(1.).is_consistent());

        assert!(rotation_x(2. * PI).m.is_close(Matrix {
            ..Default::default()
        }));
        assert!(rotation_y(-4. * PI).invm.is_close(Matrix {
            ..Default::default()
        }));
        assert!(rotation_z(0.).m.is_close(Matrix {
            ..Default::default()
        }))
    }

    #[test]
    fn test_scaling() {
        let tr1 = scaling(Vector::from((2.0, 5.0, 10.0)));
        assert!(tr1.is_consistent());

        let tr2 = scaling(Vector::from((3.0, 2.0, 4.0)));
        assert!(tr2.is_consistent());

        let expected = scaling(Vector::from((6.0, 10.0, 40.0)));
        assert!(expected.is_close(tr1 * tr2))
    }

    #[test]
    fn test_mul_vector() {
        assert!(
            (rotation_x(PI / 3.) * Vector::from((1., 1., 0.))).is_close(Vector::from((
                1.,
                0.5,
                3_f32.sqrt() / 2.
            )))
        )
    }

    #[test]
    fn test_mul_normal() {
        assert!(
            (scaling(Vector::from((2., -3., 5.))) * Normal::from((2., 1., 0.)))
                .is_close(Normal::from((1., -1. / 3., 0.)))
        )
    }

    #[test]
    fn test_mul_point() {
        assert!(
            (translation(Vector::from((1., -2., 3.))) * Point::from((-3., 2., 0.)))
                .is_close(Point::from((-2., 0., 3.)))
        )
    }

    #[test]
    fn test_mul_vpn() {
        let m = Transformation {
            m: Matrix {
                elements: [
                    [1.0, 2.0, 3.0, 4.0],
                    [5.0, 6.0, 7.0, 8.0],
                    [9.0, 9.0, 8.0, 7.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
            invm: Matrix {
                elements: [
                    [-3.75, 2.75, -1.0, 0.0],
                    [5.75, -4.75, 2.0, 1.0],
                    [-2.25, 2.25, -1.0, -2.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        };
        assert!(m.is_consistent());

        let expected_v = Vector::from((14.0, 38.0, 51.0));
        assert!(expected_v.is_close(m * Vector::from((1.0, 2.0, 3.0))));

        let expected_p = Point::from((18.0, 46.0, 58.0));
        assert!(expected_p.is_close(m * Point::from((1.0, 2.0, 3.0))));

        let expected_n = Normal::from((-8.75, 7.75, -3.0));
        assert!(expected_n.is_close(m * Normal::from((3.0, 2.0, 4.0))))
    }
}
