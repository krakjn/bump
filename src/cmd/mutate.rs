use crate::bumpfile;
use crate::cmd::{BumpError, BumpType, load_bumpfile};
use clap::ArgMatches;

pub fn mutate(matches: &ArgMatches, bump_type: BumpType) -> Result<(), BumpError> {
    let mut bumpfile = load_bumpfile(matches)?;
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
