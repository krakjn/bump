use crate::cmd::{BumpError, git_tag_exists, is_git_repository, load_bumpfile};
use crate::print::{self, PrintOptions};
use crate::version::Version;
use clap::ArgMatches;
use std::process::Command as ProcessCommand;

fn git_cmd() -> ProcessCommand {
    ProcessCommand::new("git")
}

fn create_git_tag(version: &Version, message: Option<&str>) -> Result<(), BumpError> {
    if !is_git_repository() {
        return Err(BumpError::Git("Not a git repository".to_string()));
    }

    let tag_name = print::to_string(version, &PrintOptions::default())?;

    if git_tag_exists(&tag_name)? {
        return Err(BumpError::Git(format!("Tag '{tag_name}' already exists")));
    }

    let mut cmd = git_cmd();
    cmd.args(["tag", "-a", &tag_name]);

    if let Some(msg) = message {
        cmd.args(["-m", msg]);
    } else {
        let default_message = format!("chore(release): bump version to {tag_name}");
        cmd.args(["-m", &default_message]);
    }

    let output = cmd
        .output()
        .map_err(|e| BumpError::Git(format!("failed to create git tag: {e}")))?;

    if !output.status.success() {
        return Err(BumpError::Git(format!(
            "failed to create tag '{tag_name}': {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    println!("Created git tag: {tag_name}");
    Ok(())
}

pub fn tag(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let message = matches.get_one::<String>("message");
    create_git_tag(&version, message.map(String::as_str))
}
