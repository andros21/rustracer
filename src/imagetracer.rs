//! Image Tracer module.
//!
//! Provides [`ImageTracer`](struct@ImageTracer) struct.
use crate::camera::{Camera, FireRay};
use crate::hdrimage::HdrImage;
use crate::ray::Ray;
use crate::render::{Renderer, Solve};
use crate::{Pcg, BLACK};

/// Trace an image by shooting light rays through each of its pixels.
pub struct ImageTracer<'a> {
    /// An initialized [`HdrImage`].
    image: &'a mut HdrImage,
    /// A [`Camera`] enum that implement [`FireRay`] trait.
    camera: Camera,
}

impl<'a> ImageTracer<'a> {
    /// Initialize an ImageTracer object.
    ///
    /// The parameter `image` must be a [`HdrImage`] object that has already been initialized.\
    /// The parameter `camera` must be a [`Camera`] enum that implement [`FireRay`] trait.
    pub fn new(image: &'a mut HdrImage, camera: Camera) -> Self {
        Self { image, camera }
    }

    /// Shot one light [`Ray`] through image pixel `(col, row)`.
    ///
    /// The parameters `(col, row)` are measured in the same way as\
    /// they are in `HdrImage`the bottom left corner is placed at `(0, 0)`.
    ///
    /// The values of `u_pixel` and `v_pixel` are floating-point numbers in the range `[0, 1]`.\
    /// They specify where the ray should cross the pixel; passing `0.5` to both means that\
    /// the ray will pass through the pixel's center.
    fn fire_ray(&self, col: u32, row: u32, u_pixel: f32, v_pixel: f32) -> Ray {
        let u = (col as f32 + u_pixel) / self.image.shape().0 as f32;
        let v = 1. - (row as f32 + v_pixel) / self.image.shape().1 as f32;
        self.camera.fire_ray(u, v)
    }

    /// Shoot several light rays crossing each of the pixels in the image.
    ///
    /// If `antialiasing_level` is one, for each pixel in the [`HdrImage`] object fire one [`Ray`],\
    /// and pass it to a [`Renderer`] that implement a [`Solve`] trait to determine the `Color` of the pixel.
    ///
    /// If `antialiasing_level` is greater than one, then each pixel is divided in a N by N grid,\
    /// where N is the anti-aliasing level, and a [`Ray`] is thrown for each sub-pixel;\
    /// the color of the pixel in this case is obtained as the mean color of the N*N samples.
    pub fn fire_all_rays(&mut self, mut renderer: Renderer, antialiasing_level: u32) {
        let step = 1. / antialiasing_level as f32;
        let mut pcg = Pcg::default();
        for row in 0..self.image.shape().1 {
            for col in 0..self.image.shape().0 {
                let mut color = BLACK;
                for sub_row in 0..antialiasing_level {
                    for sub_col in 0..antialiasing_level {
                        let (sub_u, sub_v) = (pcg.random_float(), pcg.random_float());
                        let ray = self.fire_ray(
                            col,
                            row,
                            (sub_row as f32 + sub_u) * step,
                            (sub_col as f32 + sub_v) * step,
                        );
                        color = color + renderer.solve(ray);
                    }
                }
                color = color * (1. / antialiasing_level.pow(2) as f32);
                self.image.set_pixel(col, row, color).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::camera::PerspectiveCamera;
    use crate::color::Color;
    use crate::misc::IsClose;
    use crate::point::Point;
    use crate::render::DummyRenderer;
    use crate::transformation::Transformation;

    #[test]
    fn test_uv_sub_mapping() {
        let mut image = HdrImage::new(4, 2);
        let camera =
            Camera::Perspective(PerspectiveCamera::new(1.0, 2.0, Transformation::default()));
        let tracer = ImageTracer::new(&mut image, camera);

        let ray1 = tracer.fire_ray(0, 0, 2.5, 1.5);
        let ray2 = tracer.fire_ray(2, 1, 0.5, 0.5);
        assert!(ray1.is_close(ray2));
    }

    #[test]
    fn test_image_coverage() {
        let mut image = HdrImage::new(4, 2);
        let camera =
            Camera::Perspective(PerspectiveCamera::new(1.0, 2.0, Transformation::default()));
        let mut tracer = ImageTracer::new(&mut image, camera);

        tracer.fire_all_rays(Renderer::Dummy(DummyRenderer), 1);
        for row in 0..image.shape().1 {
            for col in 0..image.shape().0 {
                assert!(
                    matches!(image.get_pixel(col, row), Ok(col) if col == Color::from((1.0, 2.0, 3.0)))
                );
            }
        }
    }

    #[test]
    fn test_orientation() {
        let mut image = HdrImage::new(4, 2);
        let camera =
            Camera::Perspective(PerspectiveCamera::new(1.0, 2.0, Transformation::default()));
        let tracer = ImageTracer {
            image: &mut image,
            camera,
        };

        let top_left_ray = tracer.fire_ray(0, 0, 0., 0.);
        println!("{}", top_left_ray.at(1.));
        assert!(top_left_ray.at(1.).is_close(Point::from((0., 2., 1.))));

        let bottom_right_ray = tracer.fire_ray(3, 1, 1., 1.);
        assert!(bottom_right_ray
            .at(1.)
            .is_close(Point::from((0., -2., -1.))));
    }
}
