mod changed;
mod emit;
mod init;
mod meta;
mod mutate;
mod tag;
mod update;

pub use emit::emit;
pub use init::init;
pub use meta::meta;
pub use mutate::{bump_type_from_phase, mutate};
pub use tag::tag;
pub use update::update;

use crate::bumpfile::BumpFile;
use clap::ArgMatches;
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};

pub enum BumpType {
    Epoch,
    Major,
    Minor,
    Patch,
    PhaseSet(String),
    PhaseIncrement,
    Calendar,
}

#[derive(Debug)]
pub enum BumpError {
    IoError(io::Error),
    ParseError(String),
    LogicError(String),
    Git(String),
}

impl fmt::Display for BumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    write!(f, "bump error >> file not found: {err}")
                } else {
                    write!(f, "bump error >> I/O: {err}")
                }
            }
            Self::ParseError(field) => write!(f, "bump error >> {field}"),
            Self::LogicError(msg) => write!(f, "bump error >> {msg}"),
            Self::Git(msg) => write!(f, "bump error >> {msg}"),
        }
    }
}

impl From<io::Error> for BumpError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

pub fn resolve_path(input_path: &str) -> PathBuf {
    let path = Path::new(input_path);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

pub fn ensure_directory_exists(path: &Path) -> Result<(), BumpError> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(BumpError::IoError)?;
    }
    Ok(())
}

pub fn load_bumpfile(matches: &ArgMatches) -> Result<BumpFile, BumpError> {
    let version_file_path = matches
        .get_one::<String>("bumpfile")
        .expect("BUMPFILE not provided");
    BumpFile::load(resolve_path(version_file_path))
}

fn git_cmd() -> ProcessCommand {
    let mut cmd = ProcessCommand::new("git");
    cmd.arg("-P");
    cmd
}

pub(crate) fn git_tag_exists(tag_name: &str) -> Result<bool, BumpError> {
    let output = git_cmd()
        .args([
            "rev-parse",
            "-q",
            "--verify",
            &format!("refs/tags/{tag_name}"),
        ])
        .output()
        .map_err(|e| BumpError::Git(format!("failed to check if tag exists: {e}")))?;
    Ok(output.status.success())
}

pub fn run_git(command: &str) -> Result<String, BumpError> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let output = git_cmd()
        .args(&args)
        .output()
        .map_err(|e| BumpError::Git(format!("git {command}: {e}")))?;
    if !output.status.success() {
        return Err(BumpError::Git(format!(
            "git {command}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Ok(stdout);
    }
    Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

pub fn is_git_repository() -> bool {
    git_cmd()
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_git_commit_sha() -> Result<String, BumpError> {
    run_git("rev-parse --short HEAD")
}

pub fn get_git_branch() -> Result<String, BumpError> {
    run_git("rev-parse --abbrev-ref HEAD")
}

pub(crate) fn git_ref_exists(rev: &str) -> Result<bool, BumpError> {
    let output = git_cmd()
        .args(["rev-parse", "-q", "--verify", rev])
        .output()
        .map_err(|e| BumpError::Git(format!("failed to resolve git ref '{rev}': {e}")))?;
    Ok(output.status.success())
}

pub fn git_diff_name_only(since: &str, watch_path: &str) -> Result<Vec<String>, BumpError> {
    let range = format!("{since}..HEAD");
    let output = git_cmd()
        .args(["diff", "--name-only", &range, "--", watch_path])
        .output()
        .map_err(|e| BumpError::Git(format!("git diff --name-only: {e}")))?;

    if !output.status.success() {
        return Err(BumpError::Git(format!(
            "git diff --name-only: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

pub(crate) fn repo_relative_watch_dir(bumpfile_path: &Path) -> Result<String, BumpError> {
    if !is_git_repository() {
        return Err(BumpError::Git("Not a git repository".to_string()));
    }

    let repo_root = PathBuf::from(run_git("rev-parse --show-toplevel")?);
    let bumpfile_abs = if bumpfile_path.is_absolute() {
        bumpfile_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(BumpError::IoError)?
            .join(bumpfile_path)
    };

    let watch_abs = bumpfile_abs
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo_root.clone());

    let repo_root = repo_root.canonicalize().map_err(BumpError::IoError)?;
    let watch_abs = watch_abs.canonicalize().map_err(BumpError::IoError)?;

    if watch_abs == repo_root {
        return Ok(".".to_string());
    }

    watch_abs
        .strip_prefix(&repo_root)
        .map(|relative| {
            let path = relative.to_string_lossy().replace('\\', "/");
            if path.is_empty() {
                ".".to_string()
            } else {
                path
            }
        })
        .map_err(|_| {
            BumpError::Git(format!(
                "bumpfile path {} is outside the git repository",
                bumpfile_path.display()
            ))
        })
}
