use crate::cmd::{BumpError, BumpType};
use clap_complete::aot::{Shell, generate};
use std::process::ExitCode;

mod bumpfile;
mod cli;
mod cmd;
mod compose;
mod output;
mod version;

fn egress(result: Result<(), BumpError>) -> ExitCode {
    if let Err(err) = result {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let matches = cli::cli().get_matches();
    match matches.subcommand() {
        None => egress(cmd::show(&matches)),
        Some(("show", sub_matches)) => egress(cmd::show(sub_matches)),
        Some(("major", sub_matches)) => egress(cmd::mutate(sub_matches, BumpType::Major)),
        Some(("minor", sub_matches)) => egress(cmd::mutate(sub_matches, BumpType::Minor)),
        Some(("patch", sub_matches)) => egress(cmd::mutate(sub_matches, BumpType::Patch)),
        Some(("calendar", sub_matches)) => egress(cmd::mutate(sub_matches, BumpType::Calendar)),
        Some(("phase", sub_matches)) => {
            let bump_type = cmd::bump_type_from_phase(sub_matches);
            egress(cmd::mutate(sub_matches, bump_type))
        }
        Some(("meta", sub_matches)) => egress(cmd::meta(sub_matches)),
        Some(("emit", sub_matches)) => egress(cmd::emit(sub_matches)),
        Some(("init", sub_matches)) => egress(cmd::init(sub_matches)),
        Some(("tag", sub_matches)) => egress(cmd::tag(sub_matches)),
        Some(("update", sub_matches)) => egress(cmd::update(sub_matches)),
        Some(("completion", sub_matches)) => {
            let shell = sub_matches
                .get_one::<Shell>("shell")
                .copied()
                .expect("SHELL not provided");
            let mut cmd = cli::cli();
            generate(shell, &mut cmd, "bump", &mut std::io::stdout());
            ExitCode::SUCCESS
        }
        Some((name, _)) => egress(Err(BumpError::LogicError(format!(
            "Unknown command: {name}"
        )))),
    }
}
