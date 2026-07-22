use crate::{
    cmd::{BumpError, load_bumpfile, resolve_path},
    print::{self, PrintOptions},
    version::Version,
};
use clap::ArgMatches;
use std::{fs, path::Path};
use toml_edit::{DocumentMut, value};

fn load_toml(path: &Path) -> Result<DocumentMut, BumpError> {
    let content = fs::read_to_string(path).map_err(BumpError::IoError)?;
    content
        .parse::<DocumentMut>()
        .map_err(|e| BumpError::ParseError(format!("failed to parse {}: {e}", path.display())))
}

fn save_toml(path: &Path, doc: &DocumentMut) -> Result<(), BumpError> {
    fs::write(path, doc.to_string()).map_err(BumpError::IoError)
}

fn set_toml_field(
    doc: &mut DocumentMut,
    section: &str,
    key: &str,
    value_str: &str,
) -> Result<(), BumpError> {
    let Some(table) = doc.get_mut(section) else {
        return Err(BumpError::LogicError(format!(
            "no [{section}] section found"
        )));
    };
    table[key] = value(value_str);
    Ok(())
}

pub fn update(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;
    let path_str = matches.get_one::<String>("path").ok_or_else(|| {
        BumpError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path not provided",
        ))
    })?;
    let file_path = resolve_path(path_str);

    match path_str.as_str() {
        "Cargo.toml" => cargo_toml(&version, &file_path),
        "pyproject.toml" => pyproject_toml(&version, &file_path),
        _ => Err(BumpError::LogicError(format!(
            "Unsupported file type: {path_str}"
        ))),
    }
}

fn cargo_toml(version: &Version, path: &Path) -> Result<(), BumpError> {
    let mut doc = load_toml(path)?;

    let v_str = print::to_string(version, &PrintOptions::no_prefix())?;
    println!("cargo doesn't like a character prefix in Cargo.toml, stripping prefix");

    set_toml_field(&mut doc, "package", "version", &v_str)?;
    save_toml(path, &doc)?;
    println!("Cargo.toml updated to version {v_str}");
    Ok(())
}

fn pyproject_toml(version: &Version, path: &Path) -> Result<(), BumpError> {
    let mut doc = load_toml(path)?;

    let yellow = "\x1b[33m";
    let cyan = "\x1b[36m";
    let purple = "\x1b[35m";
    let reset = "\x1b[0m";
    println!(
        "{yellow}Warning: pyproject.toml version string must comply with the following scheme:{reset}"
    );
    println!("{purple} [N!]N(.N)*[{{a|b|rc}}N][.postN][.devN]{reset}");
    println!("{cyan}  N, N!, and N.N are numeric components.{reset}");
    println!("{cyan}  {{a|b|rc}} is the alpha, beta, or release candidate suffix.{reset}");
    println!("{cyan}  postN is the post-release version.{reset}");
    println!("{cyan}  devN is the development version.{reset}");
    println!(
        "{yellow}  Public version identifiers MUST NOT include leading or trailing whitespace.{reset}"
    );

    let v_str = print::to_string(version, &PrintOptions::default())?;
    let Some(project) = doc.get_mut("project") else {
        return Err(BumpError::LogicError(
            "no [project] section found in pyproject.toml".to_string(),
        ));
    };
    if project.as_table().is_none() {
        return Err(BumpError::LogicError(
            "[project] is not a table in pyproject.toml".to_string(),
        ));
    }
    set_toml_field(&mut doc, "project", "version", &v_str)?;
    save_toml(path, &doc)?;
    println!("pyproject.toml updated to version {v_str}");
    Ok(())
}
