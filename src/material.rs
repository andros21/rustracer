//! Material module.
//!
//! Provides:
//!  * Different pigments that implement [`GetColor`] trait ;
//!  * Different BRDF that implement both [`Eval`] and [`ScatterRay`] trait;
//!  * A [`Material`] thanks to pigments and BRDF.
use crate::color::{Color, BLACK, WHITE};
use crate::hdrimage::HdrImage;
use crate::misc::Vector2D;
use crate::normal::{create_onb_from_z, Normal};
use crate::point::Point;
use crate::random::Pcg;
use crate::ray::Ray;
use crate::vector::Vector;
use std::f32::consts::PI;

/// Trait that associates a [`Color`] with each point on a parametric surface `(u,v)`.
pub trait GetColor {
    fn get_color(&self, uv: Vector2D) -> Color;
}

/// A uniform pigment.
///
/// This is the most boring pigment: a uniform hue over the whole surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct UniformPigment {
    /// A [`Color`].
    pub color: Color,
}

impl GetColor for UniformPigment {
    fn get_color(&self, _uv: Vector2D) -> Color {
        self.color
    }
}

/// A textured pigment.
///
/// The texture is given through a [`HdrImage`], maybe read from pfm file.
#[derive(Clone, Debug)]
pub struct ImagePigment<'a> {
    /// An [`HdrImage`] reference.
    hdr_img: &'a HdrImage,
}

impl<'a> ImagePigment<'a> {
    /// Create a new [`ImagePigment`] from [`HdrImage`].
    pub fn new(hdr_img: &'a HdrImage) -> Self {
        Self { hdr_img }
    }
}

impl<'a> GetColor for ImagePigment<'a> {
    fn get_color(&self, uv: Vector2D) -> Color {
        let mut col = (uv.u * self.hdr_img.shape().0 as f32) as u32;
        let mut row = (uv.v * self.hdr_img.shape().1 as f32) as u32;
        if col >= self.hdr_img.shape().0 {
            col = self.hdr_img.shape().0 - 1;
        }
        if row >= self.hdr_img.shape().1 {
            row = self.hdr_img.shape().1 - 1
        }
        // TODO
        // A nicer solution would implement bilinear interpolation
        // to reduce pixelization artifacts.
        // See https://en.wikipedia.org/wiki/Bilinear_interpolation.
        self.hdr_img.get_pixel(col, row).unwrap()
    }
}

/// A checkered pigment.
///
/// The number of rows/columns in the checkered pattern is tunable,\
/// but you cannot have a different number of repetitions along the u/v directions.
#[derive(Clone, Copy, Debug)]
pub struct CheckeredPigment {
    /// First [`Color`].
    pub color1: Color,
    /// Second [`Color`].
    pub color2: Color,
    /// Number of steps.
    pub steps: u32,
}

impl GetColor for CheckeredPigment {
    fn get_color(&self, uv: Vector2D) -> Color {
        let int_u = f32::floor(uv.u * self.steps as f32) as u32;
        let int_v = f32::floor(uv.v * self.steps as f32) as u32;
        if int_u % 2 == int_v % 2 {
            self.color1
        } else {
            self.color2
        }
    }
}

/// Enum of pigments.
#[derive(Clone, Debug)]
pub enum Pigment<'a> {
    Uniform(UniformPigment),
    Image(ImagePigment<'a>),
    Checkered(CheckeredPigment),
}

impl<'a> GetColor for Pigment<'a> {
    /// Return a different [`Color`] as function of [`Pigment`] variant.
    fn get_color(&self, uv: Vector2D) -> Color {
        match self {
            Pigment::Uniform(uniform) => uniform.get_color(uv),
            Pigment::Image(image) => image.get_color(uv),
            Pigment::Checkered(checkered) => checkered.get_color(uv),
        }
    }
}

/// A trait for evaluating a particular BRDF on a parametric surface `(u,v)`.
pub trait Eval {
    fn eval(&self, normal: Normal, in_dir: Vector, out_dir: Vector, uv: Vector2D) -> Color;
}

/// A trait for scatter a [`Ray`] for a particular BRDF.
pub trait ScatterRay {
    fn scatter_ray(
        &self,
        pcg: &mut Pcg,
        incoming_dir: Vector,
        interaction_point: Point,
        normal: Normal,
        depth: u32,
    ) -> Ray;
}

/// A class representing an ideal diffuse BRDF (also called "Lambertian").
#[derive(Clone)]
pub struct DiffuseBRDF<'a> {
    /// A generic pigment that implement [`GetColor`].
    pub pigment: Pigment<'a>,
}

impl<'a> Default for DiffuseBRDF<'a> {
    fn default() -> Self {
        Self {
            pigment: Pigment::Uniform(UniformPigment { color: WHITE }),
        }
    }
}

impl<'a> GetColor for DiffuseBRDF<'a> {
    fn get_color(&self, uv: Vector2D) -> Color {
        self.pigment.get_color(uv)
    }
}

impl<'a> Eval for DiffuseBRDF<'a> {
    fn eval(&self, _normal: Normal, _in_dir: Vector, _out_dir: Vector, uv: Vector2D) -> Color {
        self.pigment.get_color(uv) * (1.0 / PI)
    }
}

impl<'a> ScatterRay for DiffuseBRDF<'a> {
    /// Random scattering on semi-sphere using [`Pcg`] random generator.
    fn scatter_ray(
        &self,
        pcg: &mut Pcg,
        _incoming_dir: Vector,
        interaction_point: Point,
        normal: Normal,
        depth: u32,
    ) -> Ray {
        // Cosine-weighted distribution around the z (local) axis.
        let (e1, e2, e3) = create_onb_from_z(normal);
        let cos_theta_sq = pcg.random_float();
        let (cos_theta, sin_theta) = (f32::sqrt(cos_theta_sq), f32::sqrt(1.0 - cos_theta_sq));
        let phi = 2.0 * PI * pcg.random_float();

        Ray {
            origin: interaction_point,
            dir: e1 * f32::cos(phi) * cos_theta + e2 * f32::sin(phi) * cos_theta + e3 * sin_theta,
            tmin: 1.0e-3,
            depth,
            ..Default::default()
        }
    }
}

/// A class representing an ideal mirror BRDF.
#[derive(Clone)]
pub struct SpecularBRDF<'a> {
    /// A generic pigment that implement [`GetColor`] trait.
    pub pigment: Pigment<'a>,
    /// A threshold angle in radians.
    pub threshold_angle_rad: f32,
}

impl<'a> Default for SpecularBRDF<'a> {
    fn default() -> Self {
        Self {
            pigment: Pigment::Uniform(UniformPigment { color: WHITE }),
            threshold_angle_rad: PI / 1800.0,
        }
    }
}

impl<'a> GetColor for SpecularBRDF<'a> {
    fn get_color(&self, uv: Vector2D) -> Color {
        self.pigment.get_color(uv)
    }
}

impl<'a> Eval for SpecularBRDF<'a> {
    fn eval(&self, normal: Normal, in_dir: Vector, out_dir: Vector, uv: Vector2D) -> Color {
        let theta_in = f32::acos(
            Vector::from(normal)
                .normalize()
                .unwrap()
                .dot(in_dir.normalize().unwrap()),
        );
        let theta_out = f32::acos(
            Vector::from(normal)
                .normalize()
                .unwrap()
                .dot(out_dir.normalize().unwrap()),
        );

        if (theta_in - theta_out).abs() < self.threshold_angle_rad {
            self.pigment.get_color(uv)
        } else {
            BLACK
        }
    }
}

impl<'a> ScatterRay for SpecularBRDF<'a> {
    /// Perfect mirror behaviour.
    fn scatter_ray(
        &self,
        _pcg: &mut Pcg,
        incoming_dir: Vector,
        interaction_point: Point,
        normal: Normal,
        depth: u32,
    ) -> Ray {
        let ray_dir = incoming_dir.normalize().unwrap();
        let normal = Vector::from(normal).normalize().unwrap();
        let dot_prod = normal.dot(ray_dir);

        Ray {
            origin: interaction_point,
            dir: ray_dir - normal * 2.0 * dot_prod,
            depth,
            ..Default::default()
        }
    }
}

/// Enum of BRDFs.
#[derive(Clone)]
pub enum BRDF<'a> {
    Diffuse(DiffuseBRDF<'a>),
    Specular(SpecularBRDF<'a>),
}

impl<'a> Eval for BRDF<'a> {
    /// Eval a particular [`BRDF`] variant.
    fn eval(&self, normal: Normal, in_dir: Vector, out_dir: Vector, uv: Vector2D) -> Color {
        match self {
            BRDF::Diffuse(diffuse) => diffuse.eval(normal, in_dir, out_dir, uv),
            BRDF::Specular(specular) => specular.eval(normal, in_dir, out_dir, uv),
        }
    }
}

impl<'a> ScatterRay for BRDF<'a> {
    /// Scatter a [`Ray`] as a particular [`BRDF`] variant.
    fn scatter_ray(
        &self,
        pcg: &mut Pcg,
        incoming_dir: Vector,
        interaction_point: Point,
        normal: Normal,
        depth: u32,
    ) -> Ray {
        match self {
            BRDF::Diffuse(diffuse) => {
                diffuse.scatter_ray(pcg, incoming_dir, interaction_point, normal, depth)
            }
            BRDF::Specular(specular) => {
                specular.scatter_ray(pcg, incoming_dir, interaction_point, normal, depth)
            }
        }
    }
}

impl<'a> GetColor for BRDF<'a> {
    /// Return a different Color as function of Pigment variant.
    fn get_color(&self, uv: Vector2D) -> Color {
        match self {
            BRDF::Diffuse(diffuse) => diffuse.get_color(uv),
            BRDF::Specular(specular) => specular.get_color(uv),
        }
    }
}

/// A material with a particular pigment and BRDF.
#[derive(Clone)]
pub struct Material<'a> {
    /// A BRDF that implement both [`Eval`] and [`ScatterRay`] traits.
    pub brdf: BRDF<'a>,
    /// A pigment that implement [`GetColor`] trait.
    pub emitted_radiance: Pigment<'a>,
}

impl<'a> Default for Material<'a> {
    fn default() -> Self {
        Self {
            brdf: BRDF::Diffuse(DiffuseBRDF::default()),
            emitted_radiance: Pigment::Uniform(UniformPigment::default()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::normal::{E1, E2, E3};
    use crate::vector::{E1 as vE1, E2 as vE2, E3 as vE3};

    #[test]
    fn test_pigment() {
        let uniform0 = Pigment::Uniform(UniformPigment::default());
        let uniform1 = Pigment::Uniform(UniformPigment { color: WHITE });
        let checkered = Pigment::Checkered(CheckeredPigment {
            color1: BLACK,
            color2: WHITE,
            steps: 10,
        });
        let mut hdr_img = HdrImage::new(3, 3);
        hdr_img.set_pixel(0, 2, WHITE).unwrap();
        hdr_img.set_pixel(2, 0, WHITE).unwrap();
        hdr_img.set_pixel(2, 2, WHITE).unwrap();
        let image = ImagePigment::new(&hdr_img);

        assert_eq!(uniform0.get_color(Vector2D { u: 0.1, v: 3.0 }), BLACK);
        assert_eq!(uniform1.get_color(Vector2D { u: 0.5, v: 0.3 }), WHITE);
        assert_eq!(checkered.get_color(Vector2D { u: 0.0, v: 0.0 }), BLACK);
        assert_eq!(checkered.get_color(Vector2D { u: 2.0, v: 2.0 }), BLACK);
        assert_eq!(checkered.get_color(Vector2D { u: 0.0, v: 0.9 }), WHITE);
        assert_eq!(checkered.get_color(Vector2D { u: 0.9, v: 0.0 }), WHITE);
        assert_eq!(image.get_color(Vector2D { u: 0.0, v: 0.0 }), BLACK);
        assert_eq!(image.get_color(Vector2D { u: 0.0, v: 1.0 }), WHITE);
        assert_eq!(image.get_color(Vector2D { u: 1.0, v: 0.0 }), WHITE);
        assert_eq!(image.get_color(Vector2D { u: 1.0, v: 1.0 }), WHITE)
    }

    #[test]
    fn test_brdf() {
        let diff_brdf = BRDF::Diffuse(DiffuseBRDF::default());
        let spec_brdf = BRDF::Specular(SpecularBRDF::default());
        let mut pcg = Pcg::default();
        let uv = Vector2D { u: 1., v: 2. };

        assert!(matches!(&diff_brdf, BRDF::Diffuse(diff) if diff.get_color(uv)==WHITE));
        assert!(
            matches!(&spec_brdf, BRDF::Specular(spec) if (spec.get_color(uv), spec.threshold_angle_rad) == (WHITE, PI/1800.))
        );

        assert_eq!(diff_brdf.eval(E1, vE2, vE3, uv), WHITE * (1.0 / PI));
        assert_eq!(
            spec_brdf.eval(E3, vE1 + vE2 + vE3, vE1 + vE2 + vE3, uv),
            WHITE
        );
        assert_eq!(spec_brdf.eval(E2, vE1 + vE2 + vE3, vE1 + vE3, uv), BLACK);
        assert_eq!(
            spec_brdf
                .scatter_ray(
                    &mut pcg,
                    Vector::from((-1.0, 0.0, -1.0)),
                    Point::default(),
                    E3,
                    10
                )
                .dir,
            Vector::from((-1.0, 0.0, 1.0)).normalize().unwrap()
        );

        let mut sum = Vector {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let num = 5e5;
        let eps = 1.0 / f32::sqrt(num);
        for _n in 1..(num as u32) {
            sum = sum
                + diff_brdf
                    .scatter_ray(&mut pcg, vE2, Point::default(), E3, 10)
                    .dir
                    .normalize()
                    .unwrap()
        }
        sum = sum * (1.0 / num);
        assert!(
            sum.x.abs() <= eps && sum.y.abs() <= eps && (sum.z.abs() - (2.0 / 3.0)).abs() <= eps
        )
    }
}
