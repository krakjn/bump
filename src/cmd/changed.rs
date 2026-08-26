use crate::bumpfile::BumpFile;
use crate::cmd::{
    BumpError, git_diff_name_only, git_ref_exists, is_git_repository, repo_relative_watch_dir,
};

pub enum ChangedResult {
    Changed,
    Unchanged { path: String, from: String },
}

pub fn path_changed(bumpfile: &BumpFile, from: &str) -> Result<ChangedResult, BumpError> {
    if !is_git_repository() {
        return Err(BumpError::Git("Not a git repository".to_string()));
    }

    let watch_dir = repo_relative_watch_dir(bumpfile.path())?;

    if !git_ref_exists(from)? {
        return Err(BumpError::Git(format!("unknown git ref: {from}")));
    }

    let changed_files = git_diff_name_only(from, &watch_dir)?;
    if changed_files.is_empty() {
        Ok(ChangedResult::Unchanged {
            path: watch_dir,
            from: from.to_string(),
        })
    } else {
        Ok(ChangedResult::Changed)
    }
}

pub fn resolve_if_changed_from(matches: &clap::ArgMatches) -> Option<&str> {
    matches
        .get_one::<String>("if-changed-from")
        .map(String::as_str)
}
