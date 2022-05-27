#![doc = include_str!("../README.md")]

mod camera;
mod cli;
mod color;
mod error;
mod hdrimage;
mod imagetracer;
mod material;
mod misc;
mod normal;
mod point;
mod random;
mod ray;
mod render;
mod shape;
mod transformation;
mod vector;
mod world;

use image::ImageFormat;
use std::env;
use std::f32::consts::PI;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;

use crate::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use crate::color::{Color, BLACK, WHITE};
use crate::error::{ConvertErr, DemoErr, HdrImageErr};
use crate::hdrimage::{HdrImage, Luminosity};
use crate::imagetracer::ImageTracer;
use crate::material::{
    CheckeredPigment, DiffuseBRDF, Material, Pigment, SpecularBRDF, UniformPigment, BRDF,
};
use crate::misc::ByteOrder;
use crate::random::Pcg;
use crate::render::{DummyRenderer, OnOffRenderer, PathTracer, Renderer};
use crate::shape::{Plane, Sphere};
use crate::transformation::{rotation_z, scaling, translation, Transformation};
use crate::vector::Vector;
use crate::world::World;

/// Crate main function.
///
/// * parse subcommands and subcommands arguments
/// * call subcommands relative function
/// * check results
/// * print to stderr and exit 1 if error was detected
fn main() {
    let cli_m = cli::build_cli().get_matches_from(env::args_os());
    match cli_m.subcommand_name() {
        Some("convert") => exit!(convert(cli_m.subcommand_matches("convert").unwrap())),
        Some("demo") => exit!(demo(cli_m.subcommand_matches("demo").unwrap())),
        _ => exit(1),
    }
}

/// Convert High Dynamic Range (LDR) image to Low Dynamic Range (HDR).
///
/// Called when `rustracer-convert` subcommand is used.
fn convert(sub_m: &clap::ArgMatches) -> Result<(), ConvertErr> {
    let hdr_file = Path::new(sub_m.value_of("HDR").unwrap());
    let ldr_file = Path::new(sub_m.value_of("LDR").unwrap());
    let factor = f32::from_str(sub_m.value_of("factor").unwrap())
        .map_err(|e| ConvertErr::FloatParseFailure(e, String::from("factor")))?;
    let gamma = f32::from_str(sub_m.value_of("gamma").unwrap())
        .map_err(|e| ConvertErr::FloatParseFailure(e, String::from("gamma")))?;
    let mut hdr_img = HdrImage::read_pfm_file(hdr_file).map_err(ConvertErr::IoError)?;
    check!(ldr_file).map_err(ConvertErr::IoError)?;
    if sub_m.is_present("verbose") {
        println!("[info] {:?} has been read from disk", hdr_file);
    }
    hdr_img.normalize_image(factor, Luminosity::AverageLuminosity);
    hdr_img.clamp_image();
    hdr_img
        .write_ldr_file(ldr_file, gamma)
        .map_err(ConvertErr::IoError)?;
    if sub_m.is_present("verbose") {
        println!("[info] {:?} has been written to disk", ldr_file);
    }
    Ok(())
}

fn demo(sub_m: &clap::ArgMatches) -> Result<(), DemoErr> {
    let ldr_file = Path::new(sub_m.value_of("OUTPUT").unwrap());
    let factor = f32::from_str(sub_m.value_of("factor").unwrap())
        .map_err(|e| DemoErr::FloatParseFailure(e, String::from("factor")))?;
    let gamma = f32::from_str(sub_m.value_of("gamma").unwrap())
        .map_err(|e| DemoErr::FloatParseFailure(e, String::from("gamma")))?;
    let width = u32::from_str(sub_m.value_of("width").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("width")))?;
    let height = u32::from_str(sub_m.value_of("height").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("height")))?;
    let angle_deg = f32::from_str(sub_m.value_of("angle-deg").unwrap())
        .map_err(|e| DemoErr::FloatParseFailure(e, String::from("angle-deg")))?;
    let algorithm = sub_m.value_of("algorithm").unwrap();
    let num_of_rays = u32::from_str(sub_m.value_of("num-of-rays").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("num-of-rays")))?;
    let max_depth = u32::from_str(sub_m.value_of("max-depth").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("max-depth")))?;
    let init_state = u64::from_str(sub_m.value_of("init-state").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("init-state")))?;
    let init_seq = u64::from_str(sub_m.value_of("init-seq").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("init-seq")))?;
    let _samples_per_pixel = u32::from_str(sub_m.value_of("samples-per-pixel").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("samples-per-pixel")))?;
    check!(ldr_file).map_err(DemoErr::IoError)?;
    let sky_material = Material {
        brdf: BRDF::Diffuse(DiffuseBRDF {
            pigment: Pigment::Uniform(UniformPigment::default()),
        }),
        emitted_radiance: Pigment::Uniform(UniformPigment {
            color: Color::from((1.0, 0.9, 0.5)),
        }),
    };
    let ground_material = Material {
        brdf: BRDF::Diffuse(DiffuseBRDF {
            pigment: Pigment::Checkered(CheckeredPigment {
                color1: Color::from((0.3, 0.5, 0.1)),
                color2: Color::from((0.1, 0.2, 0.5)),
                steps: 10,
            }),
        }),
        emitted_radiance: Pigment::Uniform(UniformPigment::default()),
    };
    let sphere_material = Material {
        brdf: BRDF::Diffuse(DiffuseBRDF {
            pigment: Pigment::Uniform(UniformPigment {
                color: Color::from((0.3, 0.4, 0.8)),
            }),
        }),
        emitted_radiance: Pigment::Uniform(UniformPigment::default()),
    };
    let mirror_material = Material {
        brdf: BRDF::Specular(SpecularBRDF {
            pigment: Pigment::Uniform(UniformPigment {
                color: Color::from((0.6, 0.2, 0.3)),
            }),
            threshold_angle_rad: PI / 1800.0,
        }),
        emitted_radiance: Pigment::Uniform(UniformPigment::default()),
    };
    let mut hdr_img = HdrImage::new(width, height);
    if sub_m.is_present("verbose") {
        println!("[info] generating an image ({}, {})", width, height);
    }
    let mut world = World::default();
    world.add(Box::new(Sphere::new(
        translation(Vector::from((0.0, 0.0, 0.4))) * scaling(Vector::from((200.0, 200.0, 200.0))),
        sky_material,
    )));
    world.add(Box::new(Plane::new(
        Transformation::default(),
        ground_material,
    )));
    world.add(Box::new(Sphere::new(
        translation(Vector::from((0.0, 0.0, 0.1))),
        sphere_material,
    )));
    world.add(Box::new(Sphere::new(
        translation(Vector::from((1.0, 2.5, 0.0))),
        mirror_material,
    )));
    let camera_tr = rotation_z(f32::to_radians(angle_deg))
        * rotation_z(f32::to_radians(angle_deg))
        * translation(Vector::from((-2.0, 0.0, 0.5)));
    let mut tracer = ImageTracer::new(
        &mut hdr_img,
        if sub_m.is_present("orthogonal") {
            Camera::Orthogonal(OrthogonalCamera::new(
                width as f32 / height as f32,
                camera_tr,
            ))
        } else {
            Camera::Perspective(PerspectiveCamera::new(
                1.0,
                width as f32 / height as f32,
                camera_tr,
            ))
        },
    );
    tracer.fire_all_rays(match algorithm {
        "onoff" => Renderer::OnOff(OnOffRenderer::new(&world, BLACK, WHITE)),
        "pathtracer" => Renderer::PathTracer(PathTracer::new(
            &world,
            BLACK,
            Pcg::new(init_state, init_seq),
            num_of_rays,
            max_depth,
            3,
        )),
        // Otherwise dummy behaviour.
        _ => Renderer::Dummy(DummyRenderer),
    });
    if sub_m.is_present("output-pfm") {
        let hdr_file = ldr_file.with_extension("").with_extension("pfm");
        hdr_img
            .write_pfm_file(&hdr_file, ByteOrder::LittleEndian)
            .map_err(DemoErr::IoError)?;
        if sub_m.is_present("verbose") {
            println!("[info] {:?} has been written to disk", hdr_file);
        }
    }
    hdr_img.normalize_image(factor, Luminosity::AverageLuminosity);
    hdr_img.clamp_image();
    hdr_img
        .write_ldr_file(ldr_file, gamma)
        .map_err(DemoErr::IoError)?;
    if sub_m.is_present("verbose") {
        println!("[info] {:?} has been written to disk", ldr_file);
    }
    Ok(())
}
