mod c;
mod csharp;
mod go;
mod java;
mod json;
mod python;
mod raw;
mod toml;
mod yaml;

use super::{Fields, Format};
use crate::cmd::BumpError;

#[derive(Debug, Clone, Copy)]
pub enum Case {
    /// version_string
    Snake,
    /// versionString
    Camel,
    /// VersionString
    Pascal,
    /// VERSION_STRING
    Uppercase,
}

impl Case {
    pub fn parse(s: &str) -> Result<Self, BumpError> {
        match s {
            "snake" => Ok(Self::Snake),
            "camel" => Ok(Self::Camel),
            "pascal" => Ok(Self::Pascal),
            "uppercase" => Ok(Self::Uppercase),
            other => Err(BumpError::LogicError(format!(
                "Invalid case: '{other}'. Expected snake, camel, pascal, or uppercase."
            ))),
        }
    }

    pub fn apply(self, key: &str) -> String {
        let words: Vec<String> = key
            .split('_')
            .filter(|w| !w.is_empty())
            .map(|w| w.to_lowercase())
            .collect();
        match self {
            Self::Snake => words.join("_"),
            Self::Uppercase => words
                .iter()
                .map(|w| w.to_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
            Self::Camel => {
                let mut out = String::new();
                for (i, w) in words.iter().enumerate() {
                    if i == 0 {
                        out.push_str(w);
                    } else {
                        out.push_str(&capitalize(w));
                    }
                }
                out
            }
            Self::Pascal => words.iter().map(|w| capitalize(w)).collect(),
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

/// Substitute language-template placeholders.
#[rustfmt::skip]
pub fn substitute(tmpl: &str, fields: &Fields) -> String {
    tmpl.replace("{emit_prefix}", &fields.emit_prefix)
        .replace("{case_prefix}", &fields.case_prefix)
        .replace("{case_major}", &fields.case_major)
        .replace("{case_minor}", &fields.case_minor)
        .replace("{case_patch}", &fields.case_patch)
        .replace("{case_phase}", &fields.case_phase)
        .replace("{case_phase_distance}", &fields.case_phase_distance)
        .replace("{case_string}", &fields.case_string)
        .replace("{case_timestamp}", &fields.case_timestamp)
        .replace("{version_prefix}", &fields.version_prefix)
        .replace("{version_major}", &fields.version_major.to_string())
        .replace("{version_minor}", &fields.version_minor.to_string())
        .replace("{version_patch}", &fields.version_patch.to_string())
        .replace("{version_phase_distance}", &fields.version_phase_distance.to_string())
        .replace("{version_phase}", &fields.version_phase)
        .replace("{version_string}", &fields.version_string)
        .replace("{version_timestamp}", &fields.version_timestamp)
}

pub fn render(format: Format, fields: &Fields) -> Result<String, BumpError> {
    Ok(match format {
        Format::Raw => raw::render(fields),
        Format::C => c::render(fields),
        Format::Go => go::render(fields),
        Format::Java => java::render(fields),
        Format::CSharp => csharp::render(fields),
        Format::Python => python::render(fields),
        Format::Json => json::render(fields),
        Format::Toml => toml::render(fields),
        Format::Yaml => yaml::render(fields),
    })
}

pub(crate) fn nested_root_key(fields: &Fields) -> String {
    // Markup formats always use snake keys for stable, parse-friendly structure.
    format!("{}{}", fields.emit_prefix, Case::Snake.apply("version"))
}

/// Fields nested under the version object/table. Markup ignores --case (always snake).
pub(crate) fn nested_pairs(fields: &Fields) -> Vec<(String, String)> {
    let key = |name: &str| Case::Snake.apply(name);
    vec![
        (key("prefix"), fields.version_prefix.clone()),
        (key("major"), fields.version_major.to_string()),
        (key("minor"), fields.version_minor.to_string()),
        (key("patch"), fields.version_patch.to_string()),
        (key("phase"), fields.version_phase.clone()),
        (
            key("phase_distance"),
            fields.version_phase_distance.to_string(),
        ),
        (key("string"), fields.version_string.clone()),
        (key("timestamp"), fields.version_timestamp.clone()),
    ]
}

pub(crate) fn json_escape(s: &str) -> String {
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
