use crate::bumpfile::{self, BumpFile};
use crate::cmd::changed::{ChangedResult, path_changed, resolve_if_changed_from};
use crate::cmd::{BumpError, load_bumpfile};
use crate::version::Version;
use clap::ArgMatches;

fn skip_if_unchanged(bumpfile: &BumpFile, matches: &ArgMatches) -> Result<bool, BumpError> {
    let Some(from) = resolve_if_changed_from(matches) else {
        return Ok(false);
    };
    match path_changed(bumpfile, from)? {
        ChangedResult::Unchanged { path, from } => {
            eprintln!("bump warning >> no changes under {path} from {from}; skipped");
            Ok(true)
        }
        ChangedResult::Changed => Ok(false),
    }
}

fn mutate(
    matches: &ArgMatches,
    apply: impl FnOnce(&mut Version) -> Result<(), BumpError>,
) -> Result<(), BumpError> {
    let mut bumpfile = load_bumpfile(matches)?;
    if skip_if_unchanged(&bumpfile, matches)? {
        return Ok(());
    }
    let mut version = bumpfile.version()?;
    apply(&mut version)?;
    bumpfile.save(&version)?;
    println!("{}", bumpfile::report("bumped", bumpfile.path(), &version)?);
    Ok(())
}

pub fn base(matches: &ArgMatches, component_name: &str) -> Result<(), BumpError> {
    mutate(matches, |version| {
        if component_name == "date" {
            version.date_bump()?;
            Ok(())
        } else {
            version.bump(component_name)
        }
    })
}

pub fn phase(matches: &ArgMatches) -> Result<(), BumpError> {
    let phase_name = matches.get_one::<String>("name").map(String::as_str);
    mutate(matches, |version| version.phase_bump(phase_name))
}
