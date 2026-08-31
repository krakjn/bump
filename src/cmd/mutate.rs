use crate::bumpfile;
use crate::cmd::changed::{ChangedResult, path_changed, resolve_if_changed_from};
use crate::cmd::{BumpError, load_bumpfile};
use clap::ArgMatches;

pub fn base(matches: &ArgMatches, component_name: &str) -> Result<(), BumpError> {
    let mut bumpfile = load_bumpfile(matches)?;

    if let Some(from) = resolve_if_changed_from(matches) {
        match path_changed(&bumpfile, from)? {
            ChangedResult::Unchanged { path, from } => {
                eprintln!("bump warning >> no changes under {path} from {from}; skipped");
                return Ok(());
            }
            ChangedResult::Changed => {}
        }
    }

    let mut version = bumpfile.version()?;
    version.bump(component_name)?;
    bumpfile.save(&version)?;
    println!("{}", bumpfile::report("bumped", bumpfile.path(), &version)?);
    Ok(())
}

pub fn phase(matches: &ArgMatches) -> Result<(), BumpError> {
    let mut bumpfile = load_bumpfile(matches)?;

    if let Some(from) = resolve_if_changed_from(matches) {
        match path_changed(&bumpfile, from)? {
            ChangedResult::Unchanged { path, from } => {
                eprintln!("bump warning >> no changes under {path} from {from}; skipped");
                return Ok(());
            }
            ChangedResult::Changed => {}
        }
    }

    let phase_name = matches.get_one::<String>("name").map(|s| s.as_str());
    let mut version = bumpfile.version()?;
    version.phase_bump(phase_name)?;
    bumpfile.save(&version)?;
    println!("{}", bumpfile::report("bumped", bumpfile.path(), &version)?);
    Ok(())
}
