use crate::cmd::BumpError;
use crate::bumpfile::BumpFile;
use std::process::ExitCode;

mod bumpfile;
mod cli;
mod cmd;
mod output;
mod print;
mod version;

fn egress(result: Result<(), BumpError>) -> ExitCode {
    if let Err(err) = result {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn bumpfile_path_from_args(args: &[String]) -> &str {
    if args.first().is_some_and(|cmd| cmd == "init") {
        return "bump.toml";
    }
    // `update PATH [BUMPFILE]` — PATH is often Cargo.toml / pyproject.toml.
    if args.first().is_some_and(|cmd| cmd == "update") {
        if args.len() >= 3 {
            if let Some(last) = args.last() {
                if last.ends_with(".toml") {
                    return last;
                }
            }
        }
        return "bump.toml";
    }
    if let Some(last) = args.last() {
        if last == "--help" || last == "-h" {
            return "bump.toml";
        }
        if last.ends_with(".toml") {
            return last;
        }
    }
    "bump.toml"
}

fn prescan_base_components(args: &[String]) -> Result<Vec<(String, u16)>, BumpError> {
    if args.first().is_some_and(|cmd| cmd == "init") {
        return Ok(Vec::new());
    }

    let path = bumpfile_path_from_args(args);
    match BumpFile::parse(path) {
        Ok(bumpfile) => Ok(bumpfile.base_components()?),
        Err(BumpError::LogicError(msg)) if msg.contains("not found") => Ok(Vec::new()),
        Err(err) => Err(err),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let base_components = match prescan_base_components(&args) {
        Ok(components) => components,
        Err(e) => {
            eprintln!("Error: {e}");
            // NOTE: should print "default" help
            return ExitCode::FAILURE;
        }
    };
    let cli = cli::cli(base_components);
    match cli.get_matches().subcommand() {
        Some(("print", sub_matches)) => egress(print::print(sub_matches)),
        Some(("meta", sub_matches)) => egress(cmd::meta(sub_matches)),
        Some(("emit", sub_matches)) => egress(cmd::emit(sub_matches)),
        Some(("init", sub_matches)) => egress(cmd::init(sub_matches)),
        Some(("tag", sub_matches)) => egress(cmd::tag(sub_matches)),
        Some(("update", sub_matches)) => egress(cmd::update(sub_matches)),
        Some(("phase", sub_matches)) => egress(cmd::mutate::phase(sub_matches)),
        Some((name, sub_matches)) => egress(cmd::mutate::base(sub_matches, name)),
        None => {
            eprintln!(
                "{}",
                BumpError::LogicError("No command provided. Try `bump --help`".to_string())
            );
            ExitCode::FAILURE
        }
    }
}
