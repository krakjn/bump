use crate::cmd::{BumpError, ensure_directory_exists, load_bumpfile};
use crate::output::{self, Case, Format};
use clap::ArgMatches;
use std::path::Path;

pub fn emit(matches: &ArgMatches) -> Result<(), BumpError> {
    let format_str = matches
        .get_one::<String>("format")
        .expect("FORMAT not provided");
    let Some(mut format) = Format::parse(format_str) else {
        return Err(BumpError::LogicError(format!(
            "Invalid emit format: {format_str}"
        )));
    };
    let case = Case::parse(
        matches
            .get_one::<String>("case")
            .map(String::as_str)
            .unwrap_or("uppercase"),
    )?;
    let emit_prefix = matches
        .get_one::<String>("prefix")
        .map(String::as_str)
        .unwrap_or("");

    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let fields = output::Fields::populate(emit_prefix, case, &version)?;

    if let Some(outputs) = matches.get_many::<String>("output") {
        for output_file in outputs {
            let path = Path::new(output_file);
            ensure_directory_exists(path)?;
            format = if format == Format::C { Format::CHeader } else { format };
            output::write(format, &fields, path)?;
        }
    } else {
        print!("{}", output::render(format, &fields)?);
    }
    Ok(())
}
