//! Command Line Interface module.
//!
//! Provides [`build_cli`] function with all cli
//! desired subcommands and flags, using [`clap`](https://github.com/clap-rs/clap)
//! library.
use clap::{Arg, Command};

/// Default normalization factor.
///
/// When no arguments are provided to `--factor` flag
const FACTOR: &str = "0.2";
/// Default transfer function parameter.
///
/// When no arguments are provided to `--gamma` flag
const GAMMA: &str = "1.0";
/// Default image width.
///
/// When no arguments are provided to `--width` flag
const WIDTH: &str = "640";
/// Default image height.
///
/// When no arguments are provided to `--height` flag
const HEIGHT: &str = "480";
/// Default angle of view (in degrees).
///
/// When no arguments are provided to `--angle-deg` flag
const ANGLE_DEG: &str = "0.0";
/// Default rendering algorithm.
///
/// When no arguments are provided to `--algorithm` flag
const ALGORITHM: &str = "pathtracer";
/// Default number of rays for pathtracer algorithm.
///
/// When no arguments are provided to `--num-of-rays` flag
const NUM_OF_RAYS: &str = "10";
/// Default max depth for pathtracer algorithm.
///
/// When no arguments are provided to `--max-depth` flag
const MAX_DEPTH: &str = "3";
/// Default init seed for random generator.
///
/// When no arguments are provided to `--init-state` flag
const INIT_STATE: &str = "45";
/// Default identifier for random generator sequence.
///
/// When no arguments are provided to `--init-seq` flag
const INIT_SEQ: &str = "45";
/// Default anti-aliasing level.
///
/// When no arguments are provided to `--anti-aliasing` flag
const ANTI_ALIASING: &str = "1";

/// Build a [`clap::Command`](https://docs.rs/clap/latest/clap/type.Command.html)
/// for [`rustracer`](..) crate.
pub fn build_cli() -> Command<'static> {
    let cli = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg_required_else_help(true)
        .disable_help_subcommand(true)
        .dont_collapse_args_in_usage(true)
        .propagate_version(true)
        .subcommand_required(true)
        .subcommand(
            Command::new("convert")
                .arg_required_else_help(true)
                .dont_collapse_args_in_usage(true)
                .about("Convert HDR (pfm) image to LDR (ff|png) image")
                .arg(
                    Arg::new("HDR")
                        .required(true)
                        .help("Input pfm image")
                        .long_help("Input pfm file path"),
                )
                .arg(
                    Arg::new("LDR")
                        .required(true)
                        .help("Output image [possible formats: ff, png]")
                        .long_help("Output file path [possible formats: ff, png]"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Print stdout information")
                        .long_help("Print stdout information"),
                )
                .arg(
                    Arg::new("factor")
                        .short('f')
                        .long("factor")
                        .value_name("FACTOR")
                        .default_value(FACTOR)
                        .number_of_values(1)
                        .help("Normalization factor")
                        .long_help("Luminosity normalization factor"),
                )
                .arg(
                    Arg::new("gamma")
                        .short('g')
                        .long("gamma")
                        .value_name("GAMMA")
                        .default_value(GAMMA)
                        .number_of_values(1)
                        .help("Gamma parameter")
                        .long_help("Gamma transfer function parameter"),
                ),
        )
        .subcommand(
            Command::new("demo")
                .arg_required_else_help(true)
                .dont_collapse_args_in_usage(true)
                .about("Render a demo scene (hard-coded in main)")
                .arg(
                    Arg::new("OUTPUT")
                        .required(true)
                        .help("Output image [possible formats: ff, png]")
                        .long_help("Output ldr image file path [possible formats: ff, png]"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Print stdout information")
                        .long_help("Print stdout information"),
                )
                .arg(
                    Arg::new("output-pfm")
                        .long("output-pfm")
                        .help("Output also hdr image")
                        .long_help("Output also pfm file in combination with (ff|png) file"),
                )
                .arg(
                    Arg::new("orthogonal")
                        .long("orthogonal")
                        .help("Use orthogonal camera instead of perspective camera")
                        .long_help("Render image with orthogonal view of the scene"),
                )
                .arg(
                    Arg::new("width")
                        .long("width")
                        .value_name("WIDTH")
                        .default_value(WIDTH)
                        .number_of_values(1)
                        .help("Image width")
                        .long_help("Width of the image to render"),
                )
                .arg(
                    Arg::new("height")
                        .long("height")
                        .value_name("HEIGHT")
                        .default_value(HEIGHT)
                        .number_of_values(1)
                        .help("Image height")
                        .long_help("Height of the image to render"),
                )
                .arg(
                    Arg::new("angle-deg")
                        .long("angle-deg")
                        .value_name("ANGLE_DEG")
                        .default_value(ANGLE_DEG)
                        .number_of_values(1)
                        .help("View angle (in degrees)")
                        .long_help("Render the image with this angle (in degrees) of view"),
                )
                .arg(
                    Arg::new("factor")
                        .short('f')
                        .long("factor")
                        .value_name("FACTOR")
                        .default_value(FACTOR)
                        .number_of_values(1)
                        .help("Normalization factor")
                        .long_help("Luminosity normalization factor"),
                )
                .arg(
                    Arg::new("gamma")
                        .short('g')
                        .long("gamma")
                        .value_name("GAMMA")
                        .default_value(GAMMA)
                        .number_of_values(1)
                        .help("Gamma parameter")
                        .long_help("Gamma transfer function parameter"),
                )
                .arg(
                    Arg::new("algorithm")
                        .short('a')
                        .long("algorithm")
                        .value_name("ALGORITHM")
                        .default_value(ALGORITHM)
                        .number_of_values(1)
                        .possible_values(["onoff", "flat", "pathtracer"])
                        .help("Rendering algorithm")
                        .long_help("Algorithm to use for render the scene: [onoff, flat, pathtracer]"),
                )
                .arg(
                    Arg::new("num-of-rays")
                        .short('n')
                        .long("--num-of-rays")
                        .value_name("NUM_OF_RAYS")
                        .default_value(NUM_OF_RAYS)
                        .number_of_values(1)
                        .requires_if("pathtracer", "algorithm")
                        .help("Number of rays")
                        .long_help("Number of rays departing from each surface point"),
                )
                .arg(
                    Arg::new("max-depth")
                        .short('m')
                        .long("--max-depth")
                        .value_name("MAX_DEPTH")
                        .default_value(MAX_DEPTH)
                        .number_of_values(1)
                        .requires_if("pathtracer", "algorithm")
                        .help("Maximum depth")
                        .long_help("Maximum allowed ray depth"),
                )
                .arg(
                    Arg::new("init-state")
                        .long("--init-state")
                        .value_name("INIT_STATE")
                        .default_value(INIT_STATE)
                        .number_of_values(1)
                        .help("Initial random seed (positive number)")
                        .long_help(
                            "Initial seed for the random number generator (positive number)",
                        ),
                )
                .arg(
                    Arg::new("init-seq")
                        .long("--init-seq")
                        .value_name("INIT_SEQ")
                        .default_value(INIT_SEQ)
                        .number_of_values(1)
                        .help("Identifier of the random sequence (positive number)")
                        .long_help(
                            "Identifier of the sequence produced by the random number generator (positive number)",
                        ),
                )
                .arg(
                    Arg::new("anti-aliasing")
                        .long("--anti-aliasing")
                        .value_name("ANTI_ALIASING")
                        .default_value(ANTI_ALIASING)
                        .number_of_values(1)
                        .help("Anti-aliasing level")
                        .long_help(
                            "Anti-aliasing level, corresponds to the square-root of the number of samples per pixel",
                        ),
                ),
        )
        .subcommand(
            Command::new("render")
                .arg_required_else_help(true)
                .dont_collapse_args_in_usage(true)
                .about("Render a scene from file")
                .arg(
                    Arg::new("INPUT")
                        .required(true)
                        .help("Input scene file")
                        .long_help("Input scene file (formatted as yaml) to build up the scene"),
                )
                .arg(
                    Arg::new("OUTPUT")
                        .required(true)
                        .help("Output image [possible formats: ff, png]")
                        .long_help("Output ldr image file path [possible formats: ff, png]"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Print stdout information")
                        .long_help("Print stdout information"),
                )
                .arg(
                    Arg::new("output-pfm")
                        .long("output-pfm")
                        .help("Output also hdr image")
                        .long_help("Output also pfm file in combination with (ff|png) file"),
                )
                .arg(
                    Arg::new("width")
                        .long("width")
                        .value_name("WIDTH")
                        .default_value(WIDTH)
                        .number_of_values(1)
                        .help("Image width")
                        .long_help("Width of the image to render"),
                )
                .arg(
                    Arg::new("height")
                        .long("height")
                        .value_name("HEIGHT")
                        .default_value(HEIGHT)
                        .number_of_values(1)
                        .help("Image height")
                        .long_help("Height of the image to render"),
                )
                .arg(
                    Arg::new("angle-deg")
                        .long("angle-deg")
                        .value_name("ANGLE_DEG")
                        .default_value(ANGLE_DEG)
                        .number_of_values(1)
                        .help("View angle (in degrees)")
                        .long_help("Render the image with this angle (in degrees) of view"),
                )
                .arg(
                    Arg::new("factor")
                        .short('f')
                        .long("factor")
                        .value_name("FACTOR")
                        .default_value(FACTOR)
                        .number_of_values(1)
                        .help("Normalization factor")
                        .long_help("Luminosity normalization factor"),
                )
                .arg(
                    Arg::new("gamma")
                        .short('g')
                        .long("gamma")
                        .value_name("GAMMA")
                        .default_value(GAMMA)
                        .number_of_values(1)
                        .help("Gamma parameter")
                        .long_help("Gamma transfer function parameter"),
                )
                .arg(
                    Arg::new("algorithm")
                        .short('a')
                        .long("algorithm")
                        .value_name("ALGORITHM")
                        .default_value(ALGORITHM)
                        .number_of_values(1)
                        .possible_values(["onoff", "flat", "pathtracer"])
                        .help("Rendering algorithm")
                        .long_help("Algorithm to use for render the scene: [onoff, flat, pathtracer]"),
                )
                .arg(
                    Arg::new("num-of-rays")
                        .short('n')
                        .long("--num-of-rays")
                        .value_name("NUM_OF_RAYS")
                        .default_value(NUM_OF_RAYS)
                        .number_of_values(1)
                        .requires_if("pathtracer", "algorithm")
                        .help("Number of rays")
                        .long_help("Number of rays departing from each surface point"),
                )
                .arg(
                    Arg::new("max-depth")
                        .short('m')
                        .long("--max-depth")
                        .value_name("MAX_DEPTH")
                        .default_value(MAX_DEPTH)
                        .number_of_values(1)
                        .requires_if("pathtracer", "algorithm")
                        .help("Maximum depth")
                        .long_help("Maximum allowed ray depth"),
                )
                .arg(
                    Arg::new("init-state")
                        .long("--init-state")
                        .value_name("INIT_STATE")
                        .default_value(INIT_STATE)
                        .number_of_values(1)
                        .help("Initial random seed (positive number)")
                        .long_help(
                            "Initial seed for the random number generator (positive number)",
                        ),
                )
                .arg(
                    Arg::new("init-seq")
                        .long("--init-seq")
                        .value_name("INIT_SEQ")
                        .default_value(INIT_SEQ)
                        .number_of_values(1)
                        .help("Identifier of the random sequence (positive number)")
                        .long_help(
                            "Identifier of the sequence produced by the random number generator (positive number)",
                        ),
                )
                .arg(
                    Arg::new("anti-aliasing")
                        .long("--anti-aliasing")
                        .value_name("ANTI_ALIASING")
                        .default_value(ANTI_ALIASING)
                        .number_of_values(1)
                        .help("Anti-aliasing level")
                        .long_help(
                            "Anti-aliasing level, corresponds to the square-root of the number of samples per pixel",
                        ),
                ),
        );

    cli
}

/// Inherits some useful cli parameters.
///
/// Use it inside [`read_scene_file`](../../scene/struct.Scene.html#method.read_scene_file) to
/// set some standard [`f32`] values.
#[derive(Copy, Clone)]
pub struct Cli {
    // Aspect ratio usually `width/height`.
    pub aspect_ratio: f32,
    // View angle (in degrees) of the scene.
    pub angle_deg: f32,
}
