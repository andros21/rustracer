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
mod scene;
mod shape;
mod transformation;
mod vector;
mod world;

use clap_complete::{generate, Shell};
use image::ImageFormat;
use std::f32::consts::PI;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::{env, io};

use crate::camera::{Camera, OrthogonalCamera, PerspectiveCamera};
use crate::cli::Cli;
use crate::color::{Color, BLACK, WHITE};
use crate::error::{CompletionErr, ConvertErr, DemoErr, HdrImageErr, RenderErr};
use crate::hdrimage::{HdrImage, Luminosity};
use crate::imagetracer::ImageTracer;
use crate::material::{
    CheckeredPigment, DiffuseBRDF, Material, Pigment, SpecularBRDF, UniformPigment, BRDF,
};
use crate::misc::ByteOrder;
use crate::render::{DummyRenderer, FlatRenderer, OnOffRenderer, PathTracer, Renderer};
use crate::scene::Scene;
use crate::shape::{Plane, Sphere};
use crate::transformation::{rotation_z, scaling, translation, Transformation};
use crate::vector::Vector;
use crate::world::World;
use colored::Colorize;

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
        Some("render") => exit!(render(cli_m.subcommand_matches("render").unwrap())),
        Some("completion") => {
            exit!(completion(cli_m.subcommand_matches("completion").unwrap()))
        }
        // This branch should not be triggered (exit 1).
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
        println!(
            "{} {:?} has been read from disk",
            "[info]".green(),
            hdr_file
        );
    }
    hdr_img.normalize_image(factor, Luminosity::AverageLuminosity);
    hdr_img.clamp_image();
    hdr_img
        .write_ldr_file(ldr_file, gamma)
        .map_err(ConvertErr::IoError)?;
    if sub_m.is_present("verbose") {
        println!(
            "{} {:?} has been written to disk",
            "[info]".green(),
            ldr_file
        );
    }
    Ok(())
}

/// Render a demo scene (hard-coded inside main).
///
/// Called when `rustracer-demo` subcommand is used.
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
    let antialiasing_level = u32::from_str(sub_m.value_of("anti-aliasing").unwrap())
        .map_err(|e| DemoErr::IntParseFailure(e, String::from("anti-aliasing")))?;
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
    if sub_m.is_present("verbose") {
        println!(
            "{} generating an image ({}, {})",
            "[info]".green(),
            width,
            height
        );
    }
    let mut hdr_img = HdrImage::new(width, height);
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
    let renderer = match algorithm {
        "onoff" => Renderer::OnOff(OnOffRenderer::new(&world, BLACK, WHITE)),
        "flat" => Renderer::Flat(FlatRenderer::new(&world, BLACK)),
        "pathtracer" => {
            Renderer::PathTracer(PathTracer::new(&world, BLACK, num_of_rays, max_depth, 3))
        }
        // This branch should not be triggered (dummy behaviour).
        _ => Renderer::Dummy(DummyRenderer),
    };
    tracer.fire_all_rays(&renderer, init_state, init_seq, antialiasing_level);
    if sub_m.is_present("output-pfm") {
        let hdr_file = ldr_file.with_extension("").with_extension("pfm");
        hdr_img
            .write_pfm_file(&hdr_file, ByteOrder::LittleEndian)
            .map_err(DemoErr::IoError)?;
        if sub_m.is_present("verbose") {
            println!(
                "{} {:?} has been written to disk",
                "[info]".green(),
                hdr_file
            );
        }
    }
    hdr_img.normalize_image(factor, Luminosity::AverageLuminosity);
    hdr_img.clamp_image();
    hdr_img
        .write_ldr_file(ldr_file, gamma)
        .map_err(DemoErr::IoError)?;
    if sub_m.is_present("verbose") {
        println!(
            "{} {:?} has been written to disk",
            "[info]".green(),
            ldr_file
        );
    }
    Ok(())
}

/// Render a scene from file.
///
/// Called when `rustracer-render` subcommand is used.
fn render(sub_m: &clap::ArgMatches) -> Result<(), RenderErr> {
    let scene_file = Path::new(sub_m.value_of("INPUT").unwrap());
    let ldr_file = Path::new(sub_m.value_of("OUTPUT").unwrap());
    let factor = f32::from_str(sub_m.value_of("factor").unwrap())
        .map_err(|e| RenderErr::FloatParseFailure(e, String::from("factor")))?;
    let gamma = f32::from_str(sub_m.value_of("gamma").unwrap())
        .map_err(|e| RenderErr::FloatParseFailure(e, String::from("gamma")))?;
    let width = u32::from_str(sub_m.value_of("width").unwrap())
        .map_err(|e| RenderErr::IntParseFailure(e, String::from("width")))?;
    let height = u32::from_str(sub_m.value_of("height").unwrap())
        .map_err(|e| RenderErr::IntParseFailure(e, String::from("height")))?;
    let angle_deg = f32::from_str(sub_m.value_of("angle-deg").unwrap())
        .map_err(|e| RenderErr::FloatParseFailure(e, String::from("angle-deg")))?;
    let algorithm = sub_m.value_of("algorithm").unwrap();
    let num_of_rays = u32::from_str(sub_m.value_of("num-of-rays").unwrap())
        .map_err(|e| RenderErr::IntParseFailure(e, String::from("num-of-rays")))?;
    let max_depth = u32::from_str(sub_m.value_of("max-depth").unwrap())
        .map_err(|e| RenderErr::IntParseFailure(e, String::from("max-depth")))?;
    let init_state = u64::from_str(sub_m.value_of("init-state").unwrap())
        .map_err(|e| RenderErr::IntParseFailure(e, String::from("init-state")))?;
    let init_seq = u64::from_str(sub_m.value_of("init-seq").unwrap())
        .map_err(|e| RenderErr::IntParseFailure(e, String::from("init-seq")))?;
    let antialiasing_level = u32::from_str(sub_m.value_of("anti-aliasing").unwrap())
        .map_err(|e| RenderErr::IntParseFailure(e, String::from("anti-aliasing")))?;
    check!(ldr_file).map_err(RenderErr::IoError)?;
    if sub_m.is_present("verbose") {
        println!(
            "{} reading scene from file {:?}",
            "[info]".green(),
            scene_file
        );
    }
    let scene = Scene::read_scene_file(
        scene_file,
        Cli {
            aspect_ratio: width as f32 / height as f32,
            angle_deg,
        },
    )
    .map_err(|err| RenderErr::SceneError(err, String::from(sub_m.value_of("INPUT").unwrap())))?;
    if sub_m.is_present("verbose") {
        println!(
            "{} generating an image ({}, {})",
            "[info]".green(),
            width,
            height
        );
    }
    let mut hdr_img = HdrImage::new(width, height);
    let mut tracer = ImageTracer::new(&mut hdr_img, scene.camera.unwrap());
    let world = scene.shapes.unwrap();
    let renderer = match algorithm {
        "onoff" => Renderer::OnOff(OnOffRenderer::new(&world, BLACK, WHITE)),
        "flat" => Renderer::Flat(FlatRenderer::new(&world, BLACK)),
        "pathtracer" => {
            Renderer::PathTracer(PathTracer::new(&world, BLACK, num_of_rays, max_depth, 3))
        }
        // This branch should not be triggered (dummy behaviour).
        _ => Renderer::Dummy(DummyRenderer),
    };
    tracer.fire_all_rays(&renderer, init_state, init_seq, antialiasing_level);
    if sub_m.is_present("output-pfm") {
        let hdr_file = ldr_file.with_extension("").with_extension("pfm");
        hdr_img
            .write_pfm_file(&hdr_file, ByteOrder::LittleEndian)
            .map_err(RenderErr::IoError)?;
        if sub_m.is_present("verbose") {
            println!(
                "{} {:?} has been written to disk",
                "[info]".green(),
                hdr_file
            );
        }
    }
    hdr_img.normalize_image(factor, Luminosity::AverageLuminosity);
    hdr_img.clamp_image();
    hdr_img
        .write_ldr_file(ldr_file, gamma)
        .map_err(RenderErr::IoError)?;
    if sub_m.is_present("verbose") {
        println!(
            "{} {:?} has been written to disk",
            "[info]".green(),
            ldr_file
        );
    }
    Ok(())
}

/// Generate shell completions file for `rustracer` command and its subcommands.
///
/// Called when `rustracer-completion` subcommand is used.
fn completion(sub_m: &clap::ArgMatches) -> Result<(), CompletionErr> {
    let shell = Shell::from_str(sub_m.value_of("SHELL").unwrap()).unwrap();
    let home = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
    if home.is_empty() {
        println!("{} HOME env variable is empty!", "[warn]".yellow());
    }
    let mut path_buf = match shell {
        Shell::Bash => {
            Path::new(&home).join(".local/share/bash-completion/completions/rustracer.bash")
        }
        Shell::Fish => Path::new(&home).join(".config/fish/completions/rustracer.fish"),
        Shell::Zsh => Path::new(&home).join(".zfunc/_rustracer.zsh"),
        // This branch should not be triggered (empty PathBuf).
        _ => Path::new("").to_path_buf(),
    };
    if sub_m.is_present("output") {
        path_buf = Path::new(sub_m.value_of("output").unwrap()).to_path_buf();
    }
    let mut answer;
    print!(
        "{} writing completions for {} shell, continue? [Y/n] ",
        "[info]".green(),
        sub_m.value_of("SHELL").unwrap().bold()
    );
    loop {
        answer = std::string::String::new();
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut answer) {
            Ok(n) => {
                if n == 1 || (n == 2 && answer.eq_ignore_ascii_case("y\n")) {
                    create_dir_all(&path_buf.parent().unwrap()).map_err(|e| {
                        CompletionErr::WriteCompletionFailure(
                            e,
                            String::from(
                                path_buf
                                    .as_path()
                                    .parent()
                                    .unwrap_or_else(|| Path::new(""))
                                    .as_os_str()
                                    .to_str()
                                    .unwrap_or(""),
                            ),
                        )
                    })?;
                    generate(
                        shell,
                        &mut cli::build_cli(),
                        env!("CARGO_PKG_NAME"),
                        &mut BufWriter::new(File::create(&path_buf).map_err(|e| {
                            CompletionErr::WriteCompletionFailure(
                                e,
                                String::from(path_buf.as_path().as_os_str().to_str().unwrap_or("")),
                            )
                        })?),
                    );
                    println!(
                        "{} shell completions generated at\n{:tab$}{:?}",
                        "[info]".green(),
                        "",
                        path_buf,
                        tab = 7
                    );
                    break;
                } else if n == 2 && answer.eq_ignore_ascii_case("n\n") {
                    println!("{} shell completions not generated", "[warn]".yellow());
                    break;
                } else {
                    continue;
                }
            }
            _ => continue,
        }
    }
    Ok(())
}
