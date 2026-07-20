use crate::bump::{BumpError, load_bumpfile};
use crate::compose::{self, ComposeOptions};
use crate::version::SuffixMode;
use clap::ArgMatches;

pub fn meta(matches: &ArgMatches) -> Result<(), BumpError> {
    let has_prefix = matches.get_one::<String>("prefix").is_some();
    let has_suffix = matches.get_one::<String>("suffix").is_some();
    if !has_prefix && !has_suffix {
        return Err(BumpError::LogicError(
            "meta requires at least one of --prefix or --suffix".to_string(),
        ));
    }

    let mut bumpfile = load_bumpfile(matches)?;
    let mut version = bumpfile.version()?;

    if let Some(prefix) = matches.get_one::<String>("prefix") {
        version.prefix.clone_from(prefix);
    }
    if let Some(suffix) = matches.get_one::<String>("suffix") {
        version.suffix.mode = SuffixMode::parse(suffix)?;
    }

    bumpfile.save(&version)?;
    print!("{}", compose::to_string(&version, &ComposeOptions::default())?);
    Ok(())
}
