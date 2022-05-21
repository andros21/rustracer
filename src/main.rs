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
use std::path::Path;
use std::process::exit;
use std::str::FromStr;

use crate::camera::{OrthogonalCamera, PerspectiveCamera};
use crate::color::{BLACK, WHITE};
use crate::error::{ConvertErr, DemoErr, HdrImageErr};
use crate::hdrimage::{HdrImage, Luminosity};
use crate::imagetracer::ImageTracer;
use crate::misc::ByteOrder;
use crate::render::OnOffRenderer;
use crate::shape::Sphere;
use crate::transformation::{rotation_z, scaling, translation};
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
    check!(ldr_file).map_err(DemoErr::IoError)?;
    let mut hdr_img = HdrImage::new(width, height);
    if sub_m.is_present("verbose") {
        println!("[info] generating an image ({}, {})", width, height);
    }
    let mut world = World::default();
    for x in [-0.5, 0.5].into_iter() {
        for y in [-0.5, 0.5].into_iter() {
            for z in [-0.5, 0.5].into_iter() {
                world.add(Box::new(Sphere::new(
                    translation(Vector::from((x, y, z))) * scaling(Vector::from((0.1, 0.1, 0.1))),
                )));
            }
        }
    }
    world.add(Box::new(Sphere::new(
        translation(Vector::from((0.0, 0.0, -0.5))) * scaling(Vector::from((0.1, 0.1, 0.1))),
    )));
    world.add(Box::new(Sphere::new(
        translation(Vector::from((0.0, 0.5, 0.0))) * scaling(Vector::from((0.1, 0.1, 0.1))),
    )));
    let camera_tr =
        rotation_z(f32::to_radians(angle_deg)) * translation(Vector::from((-1.0, 0.0, 0.0)));
    let mut tracer = ImageTracer::new(&mut hdr_img, {
        if sub_m.is_present("orthogonal") {
            Box::new(OrthogonalCamera::new(
                width as f32 / height as f32,
                camera_tr,
            ))
        } else {
            Box::new(PerspectiveCamera::new(
                1.0,
                width as f32 / height as f32,
                camera_tr,
            ))
        }
    });
    tracer.fire_all_rays(OnOffRenderer::new(&world, BLACK, WHITE));
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
