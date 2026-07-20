use crate::cmd::{BumpError, resolve_path};
use crate::bumpfile::BumpFile;
use clap::ArgMatches;

pub fn init(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile_path = matches.get_one::<String>("bumpfile").unwrap();
    let filepath = resolve_path(bumpfile_path);
    let bumpfile = BumpFile::create(&filepath)?;
    println!(
        "Initialized new BUMPFILE at '{}'",
        bumpfile.path().display()
    );
    Ok(())
}
