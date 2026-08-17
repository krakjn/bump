use crate::bumpfile::BumpFile;
use crate::cmd::{BumpError, git_diff_name_only, git_ref_exists, is_git_repository, repo_relative_watch_dir};

pub enum ChangedResult {
    Changed,
    Unchanged { path: String, since: String },
    NoParentWarning,
}

const DEFAULT_SINCE: &str = "HEAD~1";

pub fn path_changed(bumpfile: &BumpFile, since: &str) -> Result<ChangedResult, BumpError> {
    if !is_git_repository() {
        return Err(BumpError::Git("Not a git repository".to_string()));
    }

    let watch_dir = repo_relative_watch_dir(bumpfile.path())?;

    if since == DEFAULT_SINCE && !git_ref_exists(DEFAULT_SINCE)? {
        return Ok(ChangedResult::NoParentWarning);
    }

    if !git_ref_exists(since)? {
        return Err(BumpError::Git(format!("unknown git ref: {since}")));
    }

    let changed_files = git_diff_name_only(since, &watch_dir)?;
    if changed_files.is_empty() {
        Ok(ChangedResult::Unchanged {
            path: watch_dir,
            since: since.to_string(),
        })
    } else {
        Ok(ChangedResult::Changed)
    }
}

pub fn resolve_if_changed_since(matches: &clap::ArgMatches) -> Option<&str> {
    if matches.value_source("if-changed-since").is_some() {
        matches
            .get_one::<String>("if-changed-since")
            .map(String::as_str)
    } else if matches.value_source("if-changed").is_some() {
        Some(DEFAULT_SINCE)
    } else {
        None
    }
}
