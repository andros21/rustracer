use crate::color::IsClose;
use std::ops::Mul;
use crate::error::GeometryErr;
use crate::vector::Vector;

#[derive(Clone, Debug, Default, PartialEq)]
struct Matrix {
    elements: [f32; 9],
}

impl Matrix {
    pub fn new() -> Matrix {
        Matrix {
            // Identity matrix
            elements: [1., 0., 0., 0., 1., 0., 0., 0., 1.]
        }
    }
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &f32 {
        &self.elements[index.0 * 3 + index.1]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.elements[index.0 * 3 + index.1]
    }
}

impl IsClose for Matrix {
    fn is_close(&self, other: Matrix) -> bool {
        for i in 0..9 {
            if self.elements[i].is_close(other.elements[i]) {
            } else {
                return false;
            }
        }
        return true;
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        let mut matrix = Matrix{elements: [0.; 9]};
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    matrix[(i, j)] += self[(i, k)] * other[(k, j)]
                }
            }
        }
        return matrix;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Transformation {
    m: Matrix,
    inv_m: Matrix,
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::color::EPSILON;

    #[test]
    fn test_index() {
        let mut matrix = Matrix::new();
        for i in 0..3{
            for j in 0..3{
                if i==j{ assert_eq!(matrix[(i, j)], 1.) }
                else { assert_eq!(matrix[(i, j)], 0.) }
            }
        }
        matrix[(1, 2)] = 3.;
        assert_eq!(matrix[(1, 2)], 3.)
    }

    #[test]
    fn test_is_close() {
        assert!(Matrix::new().is_close(Matrix{elements: [1.+EPSILON/2., 0., 0., EPSILON/2., 1., 0., -EPSILON/2., 0., 1.-EPSILON/2.]}));
        assert!(!(Matrix::new().is_close(Matrix{elements: [1.+EPSILON, 0., 0., EPSILON, 1., 0., -EPSILON, 0., 1.-EPSILON]})))
    }

    #[test]
    fn test_mul_matrix() {
        let m1 = Matrix{elements: [1., 2., -1., 1., -2., 0., 1., -2., 0.]};
        let m2 = Matrix{elements: [-2., 2., 1., 1., -1., 0., 1., 2., -1.]};
        assert_eq!(m1*m2, Matrix{elements: [-1., -2., 2., -4., 4., 1., -4., 4., 1.]})
    }
}
