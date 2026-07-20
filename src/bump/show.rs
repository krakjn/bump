use crate::bump::{BumpError, load_bumpfile};
use crate::compose::{self, ComposeOptions};
use clap::ArgMatches;

fn options_from_matches(matches: &ArgMatches) -> Result<ComposeOptions, BumpError> {
    let opts = ComposeOptions {
        only_prefix: matches.get_flag("only-prefix"),
        only_phase: matches.get_flag("only-phase"),
        only_base: matches.get_flag("only-base"),
        no_prefix: matches.get_flag("no-prefix"),
        no_phase: matches.get_flag("no-phase"),
        with_suffix: matches.get_flag("with-suffix"),
        with_timestamp: matches.get_flag("with-timestamp"),
        with_label: matches.get_one::<String>("with-label").cloned(),
        full: matches.get_flag("full"),
    };

    let only = [opts.only_prefix, opts.only_phase, opts.only_base]
        .into_iter()
        .filter(|&b| b)
        .count();
    if only > 1 {
        return Err(BumpError::ParseError(
            "Only one type of --only* allowed".to_string(),
        ));
    }
    Ok(opts)
}

pub fn show(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let opts = options_from_matches(matches)?;
    print!("{}", compose::to_string(&version, &opts)?);
    Ok(())
}
