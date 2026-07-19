use crate::bump::{BumpError, load_bumpfile};
use crate::print::{self, PrintOptions};
use clap::ArgMatches;

pub fn show(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let opts = PrintOptions::parse(matches)?;
    let mut components = print::Components::from(&version, &opts)?;
    print!("{}", print::assemble(&version, &opts, &mut components)?);
    Ok(())
}
