use crate::cmd::BumpError;
use crate::bumpfile::BumpFile;
use clap_complete::aot::{Shell, generate};
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

fn prescan() -> Result<(bool, BumpFile), BumpError> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let help_requested = args.contains(&"--help".to_string()) || args.contains(&"-h".to_string());
    
    let bumpfile = match args.last() {
        Some(s) if s == "--help" || s == "-h" => "bump.toml",
        Some(bumpfile) => bumpfile,
        None => "bump.toml", // shouldn't happen
    };

    Ok((help_requested, BumpFile::load(bumpfile)?))
}

fn collect_base_components(bumpfile: &BumpFile) -> Vec<(String, u16)> {
    let base = bumpfile
        .doc
        .get("base")
        .and_then(|item| item.as_table())
        .unwrap();
    let mut components = Vec::new();
    for (key, value) in base.iter() {
        if key == "delimiter" || key == "mode" {
            continue;
        }
        let component = (key.to_string(), value.as_integer().unwrap() as u16);
        components.push(component);
    }
    components
}

fn main() -> ExitCode {
    let (help_requested, bumpfile) = match prescan() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {e}");
            // NOTE: should print "default" help
            return ExitCode::FAILURE;
        }
    };
    let base_components = collect_base_components(&bumpfile);
    let mut _cli = cli::cli(base_components.clone());
    match _cli.get_matches().subcommand() {
        Some(("print", sub_matches)) => egress(print::print(sub_matches)),
        Some(("meta", sub_matches)) => egress(cmd::meta(sub_matches)),
        Some(("emit", sub_matches)) => egress(cmd::emit(sub_matches)),
        Some(("init", sub_matches)) => egress(cmd::init(sub_matches)),
        Some(("tag", sub_matches)) => egress(cmd::tag(sub_matches)),
        Some(("update", sub_matches)) => egress(cmd::update(sub_matches)),
        // Some(("completion", sub_matches)) => {
        //     let shell = sub_matches
        //         .get_one::<Shell>("shell")
        //         .copied()
        //         .expect("SHELL not provided");
        //     let mut generate_cli = cli::cli(base_components);
        //     generate(shell, &mut generate_cli, "bump", &mut std::io::stdout());
        //     ExitCode::SUCCESS
        // }
        Some(("phase", sub_matches)) => {
            egress(cmd::mutate::phase(sub_matches))
        }
        Some((name, sub_matches)) if sub_matches.contains_id("mutate") => {
            egress(cmd::mutate::base(sub_matches, name))
        }
        Some((name, _)) => {
            eprintln!("bump error >> unknown subcommand: {name}");
            ExitCode::FAILURE
        }
        None => {
            eprintln!(
                "{}",
                BumpError::LogicError("No command provided. Try `bump --help`".to_string())
            );
            ExitCode::FAILURE
        }
    }
}
