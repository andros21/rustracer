use clap::{Arg, Command};

const FACTOR: &str = "0.2";
const GAMMA: &str = "1.0";

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
        );

    cli
}
