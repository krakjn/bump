use clap::builder::styling::{AnsiColor, Styles};
use clap::builder::StyledStr;
use clap::{Arg, Command, value_parser};
use clap_complete::aot::Shell;
use std::fmt::Write;

const HELP_STYLES: Styles = Styles::styled()
    .header(AnsiColor::Blue.on_default().bold())
    .usage(AnsiColor::Cyan.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Yellow.on_default())
    .context(AnsiColor::BrightBlack.on_default())
    .context_value(AnsiColor::Cyan.on_default());

fn root_usage() -> StyledStr {
    let literal = HELP_STYLES.get_literal();
    let placeholder = HELP_STYLES.get_placeholder();
    let mut usage = StyledStr::new();
    let _ = write!(
        usage,
        "{literal}bump{literal:#} {placeholder}[OPTIONS]{placeholder:#} {placeholder}[COMMAND]{placeholder:#} {placeholder}[BUMPFILE]{placeholder:#}"
    );
    usage
}

fn bumpfile_arg() -> Arg {
    Arg::new("bumpfile")
        .value_name("BUMPFILE")
        .value_parser(clap::value_parser!(String))
        .default_value("bump.toml")
        .display_order(100)
        .help("Path to the configuration file")
}

fn show_args() -> Vec<Arg> {
    vec![
        Arg::new("only-prefix")
            .long("only-prefix")
            .action(clap::ArgAction::SetTrue)
            .group("show-exclusive")
            .help("Show [prefix]"),
        Arg::new("only-phase")
            .long("only-phase")
            .action(clap::ArgAction::SetTrue)
            .group("show-exclusive")
            .help("Show [phase]"),
        Arg::new("only-base")
            .long("only-base")
            .action(clap::ArgAction::SetTrue)
            .group("show-exclusive")
            .help("Show [base]"),
        Arg::new("no-prefix")
            .long("no-prefix")
            .action(clap::ArgAction::SetTrue)
            .help("Show [base][phase]"),
        Arg::new("no-phase")
            .long("no-phase")
            .action(clap::ArgAction::SetTrue)
            .help("Show [prefix][base]"),
        Arg::new("with-suffix")
            .long("with-suffix")
            .action(clap::ArgAction::SetTrue)
            .help("Show [prefix][base][phase][suffix]"),
        Arg::new("with-timestamp")
            .long("with-timestamp")
            .action(clap::ArgAction::SetTrue)
            .help("Show [prefix][base][phase][timestamp]"),
        Arg::new("full")
            .long("full")
            .action(clap::ArgAction::SetTrue)
            .help("Show full output; overrides all show flags except --with-label"),
        Arg::new("with-label")
            .long("with-label")
            .value_name("LABEL")
            .allow_hyphen_values(true)
            .value_parser(clap::value_parser!(String))
            .num_args(1)
            .help("Inject LABEL at [label].position (not persisted)"),
    ]
}

#[allow(clippy::too_many_lines)]
pub fn cli() -> Command {
    let show_flags = show_args();

    Command::new("bump")
        .styles(HELP_STYLES)
        .version(env!("CARGO_PKG_VERSION"))
        .about("Automatic un-opinionated version bumping")
        .override_usage(root_usage())
        .args(show_flags.clone())
        .arg(bumpfile_arg())
        .subcommand(
            Command::new("show")
                .about("Show composed version from BUMPFILE without newline")
                .alias("p")
                .alias("print")
                .args(show_flags)
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("major")
                .about("Increment major version")
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("minor")
                .about("Increment minor version")
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("patch")
                .about("Increment patch version")
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("phase")
                .about("Increment phase distance, or set phase name and reset distance")
                .arg(
                    Arg::new("name")
                        .value_name("NAME")
                        .value_parser(clap::value_parser!(String))
                        .num_args(1)
                        .required(false)
                        .allow_hyphen_values(true)
                        .help("Phase name to set (omit to increment distance)"),
                )
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("calendar")
                .about("Update version based on current calendar date")
                .alias("cal")
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("meta")
                .about("Update bumpfile metadata fields")
                .arg(
                    Arg::new("prefix")
                        .long("prefix")
                        .value_name("PREFIX")
                        .value_parser(clap::value_parser!(String))
                        .allow_hyphen_values(true)
                        .num_args(1)
                        .help("Set prefix string (e.g. 'v', 'release-')"),
                )
                .arg(
                    Arg::new("suffix")
                        .long("suffix")
                        .value_name("MODE")
                        .value_parser(clap::builder::PossibleValuesParser::new([
                            "git_sha", "branch",
                        ]))
                        .num_args(1)
                        .help("Set suffix mode"),
                )
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("emit")
                .about("Emit version in a chosen format to stdout or files")
                .arg(
                    Arg::new("format")
                        .value_name("FORMAT")
                        .value_parser(clap::builder::PossibleValuesParser::new([
                            "raw", "c", "java", "csharp", "go", "python", "json", "toml", "yaml",
                        ]))
                        .num_args(1)
                        .required(true)
                        .help("Output format"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("OUTPUT")
                        .value_parser(clap::value_parser!(String))
                        .action(clap::ArgAction::Append)
                        .help("Write to file(s) instead of stdout"),
                )
                .arg(
                    Arg::new("case")
                        .short('c')
                        .long("case")
                        .value_name("CASE")
                        .value_parser(clap::builder::PossibleValuesParser::new([
                            "camel",
                            "pascal",
                            "snake",
                            "kebab",
                            "title",
                            "lowercase",
                            "uppercase",
                        ]))
                        .num_args(1)
                        .default_value("snake")
                        .help("Key case for structured formats (json/toml/yaml)"),
                )
                .arg(
                    Arg::new("prefix")
                        .long("prefix")
                        .value_name("PREFIX")
                        .value_parser(clap::value_parser!(String))
                        .allow_hyphen_values(true)
                        .num_args(1)
                        .help("Prepend PREFIX to emitted output (not bumpfile prefix)"),
                )
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("init")
                .about("Initialize a new version file with default values")
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("tag")
                .about("Create a conventional git tag based on the current bumpfile version")
                .arg(
                    Arg::new("message")
                        .short('m')
                        .long("message")
                        .value_name("MESSAGE")
                        .value_parser(clap::value_parser!(String))
                        .help("Custom tag message (defaults to conventional commit format)"),
                )
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("update")
                .about("Update version in known file types (e.g. Cargo.toml)")
                .arg(
                    Arg::new("path")
                        .value_name("PATH")
                        .num_args(1)
                        .value_parser(clap::builder::PossibleValuesParser::new([
                            "Cargo.toml",
                            "pyproject.toml",
                        ]))
                        .required(true)
                        .help("File type bump knows how to update"),
                )
                .arg(bumpfile_arg()),
        )
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion script")
                .arg(
                    Arg::new("shell")
                        .value_name("SHELL")
                        .value_parser(value_parser!(Shell))
                        .required(true)
                        .help("Output shell completion script for SHELL"),
                ),
        )
}
