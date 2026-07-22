mod c;
mod c_header;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
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
        Format::CHeader => c_header::render(fields), // only used on output
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{Fields, Format};
    use crate::version::{
        Base, Label, LabelPosition, Phase, Suffix, SuffixMode, Timestamp, Version, VersionMode,
    };

    #[test]
    fn case_apply_variants() {
        assert_eq!(Case::Snake.apply("VERSION_STRING"), "version_string");
        assert_eq!(Case::Camel.apply("VERSION_STRING"), "versionString");
        assert_eq!(Case::Pascal.apply("VERSION_STRING"), "VersionString");
        assert_eq!(Case::Uppercase.apply("VERSION_STRING"), "VERSION_STRING");
    }

    #[test]
    fn substitute_replaces_placeholders() {
        let fields = Fields {
            emit_prefix: "APP_".to_string(),
            case_string: "VERSION_STRING".to_string(),
            case_prefix: String::new(),
            case_major: String::new(),
            case_minor: String::new(),
            case_patch: String::new(),
            case_phase: String::new(),
            case_phase_distance: String::new(),
            case_timestamp: String::new(),
            version_string: "v-1.0.0".to_string(),
            version_timestamp: String::new(),
            version_prefix: String::new(),
            version_major: 1,
            version_minor: 0,
            version_patch: 0,
            version_phase: String::new(),
            version_phase_distance: 0,
            version_mode: VersionMode::Semver,
        };
        let out = substitute("#define {emit_prefix}{case_string} \"{version_string}\"", &fields);
        assert_eq!(out, "#define APP_VERSION_STRING \"v-1.0.0\"");
    }

    #[test]
    fn json_escape_quotes() {
        assert_eq!(json_escape(r#"say "hi""#), r#""say \"hi\"""#);
    }

    #[test]
    fn render_raw_contains_version_string() {
        let version = Version {
            prefix: "v-".to_string(),
            base: Base {
                mode: VersionMode::Semver,
                delimiter: ".".to_string(),
                major: 0,
                minor: Some(1),
                patch: Some(0),
            },
            phase: Phase {
                separator: "-".to_string(),
                name: String::new(),
                delimiter: ".".to_string(),
                distance: 0,
            },
            suffix: Suffix {
                mode: SuffixMode::GitSha,
                separator: "+".to_string(),
            },
            timestamp: Timestamp {
                format: String::new(),
                last: String::new(),
            },
            label: Label {
                position: LabelPosition::AfterBase,
            },
        };
        let fields = Fields::populate("", Case::Uppercase, &version).unwrap();
        let out = render(Format::Raw, &fields).unwrap();
        assert!(out.contains("VERSION_STRING=\"v-0.1.0\""));
    }
}
