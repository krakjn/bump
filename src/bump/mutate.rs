use crate::bump::{BumpError, BumpType, load_bumpfile};
use crate::compose::{self, ComposeOptions};
use clap::ArgMatches;

pub fn mutate(matches: &ArgMatches, bump_type: BumpType) -> Result<(), BumpError> {
    let mut bumpfile = load_bumpfile(matches)?;
    let mut version = bumpfile.version()?;

    version.bump(&bump_type)?;
    bumpfile.mismatch()?;
    println!(
        "bumped {} to {}",
        bumpfile.path().display(),
        compose::to_string(&version, &ComposeOptions::with_timestamp())?
    );
    bumpfile.save(&version)?;
    Ok(())
}

pub fn bump_type_from_phase(matches: &ArgMatches) -> BumpType {
    match matches.get_one::<String>("name") {
        Some(name) => BumpType::Phase(name.clone()),
        None => BumpType::Phase("__increment__".to_string()),
    }
}
