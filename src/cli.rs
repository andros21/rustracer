//! Command Line Interface module.
//!
//! Provides [`build_cli`] function with all cli
//! desired subcommands and flags, using [`clap`](https://github.com/clap-rs/clap)
//! library.
use clap::{Arg, Command};

/// Default normalization factor.
///
/// When no arguments are provided to `--factor` flag of `convert` subcommand.
const FACTOR: &str = "0.2";
/// Default transfer function parameter.
///
/// When no arguments are provided to `--gamma` flag of `convert` subcommand.
const GAMMA: &str = "1.0";

const WIDTH: &str = "640";
const HEIGHT: &str = "480";
const ANGLE_DEG: &str = "0.0";

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
                        .help("Input image")
                        .long_help("Input pfm file path"),
                )
                .arg(
                    Arg::new("LDR")
                        .required(true)
                        .help("Output image")
                        .long_help("Output (ff|png) file path"),
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
                .about("Render subcommand demo (devel in progress)")
                .arg(
                    Arg::new("OUTPUT")
                        .required(true)
                        .help("Output image")
                        .long_help("Output ldr image (ff|png) file path"),
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
                        .value_name("angle-deg")
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
                ),
        );

    cli
}
