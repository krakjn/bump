use crate::bumpfile::{self, BumpFile};
use crate::cmd::{BumpError, resolve_path};
use clap::ArgMatches;

pub fn init(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile_path = matches.get_one::<String>("bumpfile").unwrap();
    let force = matches.get_flag("force");
    let filepath = resolve_path(bumpfile_path);
    let bumpfile = BumpFile::create(&filepath, force)?;
    let version = bumpfile.version()?;
    println!(
        "{}",
        bumpfile::report("initialized", bumpfile.path(), &version)?
    );
    Ok(())
}
