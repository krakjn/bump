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

use super::{BaseComponentField, Fields, Format};
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

/// Substitute fixed-field placeholders (prefix, phase, string, timestamp).
pub fn substitute(tmpl: &str, fields: &Fields) -> String {
    tmpl.replace("{emit_prefix}", &fields.emit_prefix)
        .replace("{case_prefix}", &fields.case_prefix)
        .replace("{case_phase}", &fields.case_phase)
        .replace("{case_phase_distance}", &fields.case_phase_distance)
        .replace("{case_string}", &fields.case_string)
        .replace("{case_timestamp}", &fields.case_timestamp)
        .replace("{version_prefix}", &fields.version_prefix)
        .replace(
            "{version_phase_distance}",
            &fields.version_phase_distance.to_string(),
        )
        .replace("{version_phase}", &fields.version_phase)
        .replace("{version_string}", &fields.version_string)
        .replace("{version_timestamp}", &fields.version_timestamp)
}

pub(crate) fn base_int_lines(fields: &Fields, line: impl Fn(&str, &BaseComponentField) -> String) -> String {
    let mut out = String::new();
    for (i, component) in fields.base_components.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&line(&fields.emit_prefix, component));
    }
    out
}

fn append_block(out: &mut String, block: &str) {
    if block.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(block);
}

pub(crate) fn join_blocks(blocks: &[&str]) -> String {
    let mut out = String::new();
    for block in blocks {
        append_block(&mut out, block);
    }
    out
}

pub fn render(format: Format, fields: &Fields) -> Result<String, BumpError> {
    Ok(match format {
        Format::Raw => raw::render(fields),
        Format::C => c::render(fields),
        Format::CHeader => c_header::render(fields),
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
    format!("{}{}", fields.emit_prefix, Case::Snake.apply("version"))
}

pub(crate) fn nested_pairs(fields: &Fields) -> Vec<(String, String)> {
    let key = |name: &str| Case::Snake.apply(name);
    let mut pairs = vec![(key("prefix"), fields.version_prefix.clone())];
    for component in &fields.base_components {
        pairs.push((component.key.clone(), component.value.to_string()));
    }
    pairs.extend([
        (key("phase"), fields.version_phase.clone()),
        (
            key("phase_distance"),
            fields.version_phase_distance.to_string(),
        ),
        (key("string"), fields.version_string.clone()),
        (key("timestamp"), fields.version_timestamp.clone()),
    ]);
    pairs
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
    use crate::version::Version;

    #[test]
    fn case_apply_variants() {
        assert_eq!(Case::Snake.apply("VERSION_STRING"), "version_string");
        assert_eq!(Case::Camel.apply("VERSION_STRING"), "versionString");
        assert_eq!(Case::Pascal.apply("VERSION_STRING"), "VersionString");
        assert_eq!(Case::Uppercase.apply("VERSION_STRING"), "VERSION_STRING");
        assert_eq!(Case::Uppercase.apply("VERSION_ALPHA"), "VERSION_ALPHA");
    }

    #[test]
    fn substitute_replaces_placeholders() {
        let fields = Fields {
            emit_prefix: "APP_".to_string(),
            base_components: Vec::new(),
            case_string: "VERSION_STRING".to_string(),
            case_prefix: String::new(),
            case_phase: String::new(),
            case_phase_distance: String::new(),
            case_timestamp: String::new(),
            version_string: "v-1.0.0".to_string(),
            version_timestamp: String::new(),
            version_prefix: String::new(),
            version_phase: String::new(),
            version_phase_distance: 0,
        };
        let out = substitute("#define {emit_prefix}{case_string} \"{version_string}\"", &fields);
        assert_eq!(out, "#define APP_VERSION_STRING \"v-1.0.0\"");
    }

    #[test]
    fn nested_pairs_includes_dynamic_base_keys() {
        let version = Version::test_fixture();
        let fields = Fields::populate("", Case::Uppercase, &version).unwrap();
        let pairs = nested_pairs(&fields);
        assert_eq!(pairs[0].0, "prefix");
        assert_eq!(pairs[1], ("major".to_string(), "0".to_string()));
        assert_eq!(pairs[2], ("minor".to_string(), "1".to_string()));
        assert_eq!(pairs[3], ("patch".to_string(), "0".to_string()));
    }

    #[test]
    fn json_escape_quotes() {
        assert_eq!(json_escape(r#"say "hi""#), r#""say \"hi\"""#);
    }

    #[test]
    fn render_raw_contains_version_string() {
        let fields = Fields::populate("", Case::Uppercase, &Version::test_fixture()).unwrap();
        let out = render(Format::Raw, &fields).unwrap();
        assert!(out.contains("VERSION_STRING=\"v-0.1.0\""));
        assert!(out.contains("VERSION_MAJOR=0"));
    }
}
