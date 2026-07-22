use crate::cmd::{BumpError, ensure_directory_exists, load_bumpfile};
use crate::output::{self, Case, Format, Fields};
use clap::ArgMatches;
use std::path::Path;

pub fn emit(matches: &ArgMatches) -> Result<(), BumpError> {
    let mut format = *matches.get_one::<Format>("format").expect("FORMAT not provided");
    let case = matches
        .get_one::<Case>("case")
        .copied()
        .unwrap_or(Case::Uppercase);
    let emit_prefix = matches
        .get_one::<String>("prefix")
        .map(String::as_str)
        .unwrap_or("");

    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let fields = Fields::populate(emit_prefix, case, &version)?;

    if let Some(outputs) = matches.get_many::<String>("output") {
        for output_file in outputs {
            let path = Path::new(output_file);
            ensure_directory_exists(path)?;
            format = if format == Format::C {
                Format::CHeader
            } else {
                format
            };
            output::write(format, &fields, path)?;
        }
    } else {
        print!("{}", output::render(format, &fields)?);
    }
    Ok(())
}
