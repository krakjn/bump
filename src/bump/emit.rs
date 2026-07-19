use crate::bump::{BumpError, ensure_directory_exists, load_bumpfile};
use crate::lang::{self, Language};
use crate::print::{self, PrintOptions};
use crate::version::Version;
use clap::ArgMatches;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
enum Case {
    Camel,
    Pascal,
    Snake,
    Kebab,
    Title,
    Lowercase,
    Uppercase,
}

impl Case {
    fn parse(s: &str) -> Result<Self, BumpError> {
        match s {
            "camel" => Ok(Self::Camel),
            "pascal" => Ok(Self::Pascal),
            "snake" => Ok(Self::Snake),
            "kebab" => Ok(Self::Kebab),
            "title" => Ok(Self::Title),
            "lowercase" => Ok(Self::Lowercase),
            "uppercase" => Ok(Self::Uppercase),
            other => Err(BumpError::LogicError(format!("Invalid case: {other}"))),
        }
    }

    fn apply(self, key: &str) -> String {
        let words: Vec<&str> = key.split('_').filter(|w| !w.is_empty()).collect();
        match self {
            Self::Snake => key.to_string(),
            Self::Kebab => words.join("-"),
            Self::Lowercase => key.to_lowercase(),
            Self::Uppercase => key.to_uppercase(),
            Self::Camel => {
                let mut out = String::new();
                for (i, w) in words.iter().enumerate() {
                    if i == 0 {
                        out.push_str(&w.to_lowercase());
                    } else {
                        out.push_str(&capitalize(w));
                    }
                }
                out
            }
            Self::Pascal | Self::Title => words.iter().map(|w| capitalize(w)).collect(),
        }
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut s = first.to_uppercase().collect::<String>();
            s.push_str(&chars.as_str().to_lowercase());
            s
        }
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn structured_fields(version: &Version) -> Result<Vec<(&'static str, String)>, BumpError> {
    let version_string = print::to_string(version, &PrintOptions::default())?;
    Ok(vec![
        ("version", version_string),
        ("prefix", version.prefix.clone()),
        ("mode", version.base.mode.as_str().to_string()),
        ("major", version.base.major.to_string()),
        (
            "minor",
            version
                .base
                .minor
                .map_or_else(String::new, |m| m.to_string()),
        ),
        (
            "patch",
            version
                .base
                .patch
                .map_or_else(String::new, |p| p.to_string()),
        ),
        ("phase", version.phase.name.clone()),
        ("distance", version.phase.distance.to_string()),
        ("timestamp", version.timestamp.last.clone()),
    ])
}

fn render_json(version: &Version, case: Case) -> Result<String, BumpError> {
    let fields = structured_fields(version)?;
    let mut out = String::from("{\n");
    for (i, (key, value)) in fields.iter().enumerate() {
        let keyed = case.apply(key);
        out.push_str("  ");
        out.push_str(&json_escape(&keyed));
        out.push_str(": ");
        out.push_str(&json_escape(value));
        if i + 1 != fields.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push('}');
    out.push('\n');
    Ok(out)
}

fn render_toml(version: &Version, case: Case) -> Result<String, BumpError> {
    let fields = structured_fields(version)?;
    let mut out = String::new();
    for (key, value) in fields {
        out.push_str(&case.apply(key));
        out.push_str(" = ");
        out.push_str(&json_escape(&value));
        out.push('\n');
    }
    Ok(out)
}

fn render_yaml(version: &Version, case: Case) -> Result<String, BumpError> {
    let fields = structured_fields(version)?;
    let mut out = String::new();
    for (key, value) in fields {
        out.push_str(&case.apply(key));
        out.push_str(": ");
        if value.is_empty() {
            out.push_str("\"\"");
        } else if value.contains(':') || value.contains('#') || value.contains('\n') {
            out.push_str(&json_escape(&value));
        } else {
            out.push_str(&value);
        }
        out.push('\n');
    }
    Ok(out)
}

fn render_payload(
    format: &str,
    version: &Version,
    case: Case,
) -> Result<String, BumpError> {
    match format {
        "raw" => print::to_string(version, &PrintOptions::default()),
        "json" => render_json(version, case),
        "toml" => render_toml(version, case),
        "yaml" => render_yaml(version, case),
        lang_str => {
            let Some(lang) = Language::from_str(lang_str) else {
                return Err(BumpError::LogicError(format!(
                    "Invalid emit format: {format}"
                )));
            };
            lang::render(lang, version)
        }
    }
}

fn write_outputs(paths: &[&String], content: &str) -> Result<(), BumpError> {
    for path_str in paths {
        let path = Path::new(path_str);
        ensure_directory_exists(path)?;
        fs::write(path, content).map_err(BumpError::IoError)?;
        println!("written to {}", path.display());
    }
    Ok(())
}

pub fn emit(matches: &ArgMatches) -> Result<(), BumpError> {
    let format = matches
        .get_one::<String>("format")
        .expect("FORMAT not provided");
    let case = Case::parse(
        matches
            .get_one::<String>("case")
            .map(String::as_str)
            .unwrap_or("snake"),
    )?;
    let output_prefix = matches
        .get_one::<String>("prefix")
        .map(String::as_str)
        .unwrap_or("");

    let bumpfile = load_bumpfile(matches)?;
    let version = bumpfile.version()?;

    // Language templates to files without an emit --prefix: keep lang writer messages.
    if output_prefix.is_empty()
        && let Some(lang) = Language::from_str(format)
        && let Some(outputs) = matches.get_many::<String>("output")
    {
        for output_file in outputs {
            let path = Path::new(output_file);
            ensure_directory_exists(path)?;
            lang::output_file(lang, &version, path)?;
        }
        return Ok(());
    }

    let mut payload = render_payload(format, &version, case)?;
    if !output_prefix.is_empty() {
        payload = format!("{output_prefix}{payload}");
    }

    if let Some(outputs) = matches.get_many::<String>("output") {
        let paths: Vec<&String> = outputs.collect();
        write_outputs(&paths, &payload)?;
    } else {
        print!("{payload}");
    }
    Ok(())
}
