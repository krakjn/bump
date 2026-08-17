use crate::bumpfile;
use crate::cmd::changed::{ChangedResult, path_changed, resolve_if_changed_since};
use crate::cmd::{BumpError, BumpType, load_bumpfile};
use clap::ArgMatches;

pub fn mutate(matches: &ArgMatches, bump_type: BumpType) -> Result<(), BumpError> {
    let mut bumpfile = load_bumpfile(matches)?;

    if let Some(since) = resolve_if_changed_since(matches) {
        match path_changed(&bumpfile, since)? {
            ChangedResult::Unchanged { path, since } => {
                eprintln!("bump warning >> no changes under {path} since {since}; skipped");
                return Ok(());
            }
            ChangedResult::NoParentWarning => {
                eprintln!("bump warning >> no parent commit; bumping anyway");
            }
            ChangedResult::Changed => {}
        }
    }

    let mut version = bumpfile.version()?;

    version.bump(&bump_type)?;
    bumpfile.mismatch()?;
    bumpfile.save(&version)?;
    println!("{}", bumpfile::report("bumped", bumpfile.path(), &version)?);
    Ok(())
}

pub fn bump_type_from_phase(matches: &ArgMatches) -> BumpType {
    match matches.get_one::<String>("name") {
        Some(name) => BumpType::PhaseSet(name.clone()),
        None => BumpType::PhaseIncrement,
    }
}
