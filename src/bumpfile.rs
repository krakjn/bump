use crate::cmd::{BumpError, ensure_directory_exists};
use crate::print::{self, PrintSelection};
use crate::version::{Base, Label, Phase, Suffix, Timestamp, Version};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Table, Value, value};

const BASE_RESERVED: &[&str] = &["delimiter", "mode"];

pub struct BumpFile {
    path: PathBuf,
    doc: DocumentMut,
}

fn bumpfile_parse_error(path: &Path, message: impl fmt::Display) -> BumpError {
    BumpError::ParseError(format!(
        "{message} in {}. Recreate your bumpfile with 'bump init'.",
        path.display()
    ))
}

fn table<'a>(doc: &'a DocumentMut, section: &str, path: &Path) -> Result<&'a Table, BumpError> {
    doc.get(section)
        .and_then(Item::as_table)
        .ok_or_else(|| bumpfile_parse_error(path, format!("'{section}' table not found")))
}

fn table_mut<'a>(
    doc: &'a mut DocumentMut,
    section: &str,
    path: &Path,
) -> Result<&'a mut Table, BumpError> {
    doc.get_mut(section)
        .and_then(Item::as_table_mut)
        .ok_or_else(|| bumpfile_parse_error(path, format!("'{section}' table not found")))
}

fn set<V: Into<Value>>(
    table: &mut Table,
    key: &str,
    val: V,
    section: &str,
    path: &Path,
) -> Result<(), BumpError> {
    if !table.contains_key(key) {
        return Err(bumpfile_parse_error(
            path,
            format!("Expected key '{key}' not found in [{section}]"),
        ));
    }
    table[key] = value(val);
    Ok(())
}

fn str_field(table: &Table, key: &str, section: &str, path: &Path) -> Result<String, BumpError> {
    table
        .get(key)
        .and_then(Item::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            bumpfile_parse_error(
                path,
                format!("Expected string key '{key}' not found in [{section}]"),
            )
        })
}

fn u32_field(table: &Table, key: &str, section: &str, path: &Path) -> Result<u32, BumpError> {
    let n = table.get(key).and_then(Item::as_integer).ok_or_else(|| {
        bumpfile_parse_error(
            path,
            format!("Expected integer key '{key}' not found in [{section}]"),
        )
    })?;
    u32::try_from(n).map_err(|_| {
        bumpfile_parse_error(
            path,
            format!("Expected non-negative integer for [{section}].{key}"),
        )
    })
}

fn parse_base_components(
    table: &Table,
    path: &Path,
) -> Result<(String, Vec<(String, u16)>), BumpError> {
    let delimiter = str_field(table, "delimiter", "base", path)?;
    let mut components = Vec::new();
    for (key, value) in table.iter() {
        if BASE_RESERVED.contains(&key) {
            continue;
        }
        let n = value.as_integer().ok_or_else(|| {
            bumpfile_parse_error(path, format!("Expected integer for [base].{key}"))
        })?;
        if !(0..=i64::from(u16::MAX)).contains(&n) {
            return Err(bumpfile_parse_error(
                path,
                format!("Expected [base].{key} in range 0..={}", u16::MAX),
            ));
        }
        components.push((key.to_string(), n as u16));
    }
    Ok((delimiter, components))
}

fn version_from_doc(doc: &DocumentMut, path: &Path) -> Result<Version, BumpError> {
    let prefix = doc
        .get("prefix")
        .and_then(Item::as_str)
        .ok_or_else(|| bumpfile_parse_error(path, "Expected key 'prefix' not found in [(root)]"))?
        .to_string();

    let base_table = table(doc, "base", path)?;
    let (delimiter, components) = parse_base_components(base_table, path)?;

    let phase_table = table(doc, "phase", path)?;
    let phase = Phase {
        separator: str_field(phase_table, "separator", "phase", path)?,
        name: str_field(phase_table, "name", "phase", path)?,
        delimiter: str_field(phase_table, "delimiter", "phase", path)?,
        distance: u32_field(phase_table, "distance", "phase", path)?,
    };

    let suffix_table = table(doc, "suffix", path)?;
    let suffix = Suffix {
        mode: str_field(suffix_table, "mode", "suffix", path)?
            .parse()
            .map_err(|e| bumpfile_parse_error(path, e))?,
        separator: str_field(suffix_table, "separator", "suffix", path)?,
    };

    let timestamp_table = table(doc, "timestamp", path)?;
    let timestamp = Timestamp {
        format: str_field(timestamp_table, "format", "timestamp", path)?,
        last: str_field(timestamp_table, "last", "timestamp", path)?,
    };

    let label_table = table(doc, "label", path)?;
    let label = Label {
        position: str_field(label_table, "position", "label", path)?
            .parse()
            .map_err(|e| bumpfile_parse_error(path, e))?,
    };

    Ok(Version {
        prefix,
        base: Base {
            delimiter,
            components,
        },
        phase,
        suffix,
        timestamp,
        label,
    })
}

fn write_base(doc: &mut DocumentMut, version: &Version, path: &Path) -> Result<(), BumpError> {
    let base = table_mut(doc, "base", path)?;

    set(base, "delimiter", &version.base.delimiter, "base", path)?;

    for (name, value) in &version.base.components {
        set(base, name, i64::from(*value), "base", path)?;
    }
    Ok(())
}

fn write_version_into_doc(
    doc: &mut DocumentMut,
    version: &Version,
    path: &Path,
) -> Result<(), BumpError> {
    if !doc.contains_key("prefix") {
        return Err(bumpfile_parse_error(
            path,
            "Expected key 'prefix' not found in [(root)]",
        ));
    }
    doc["prefix"] = value(&version.prefix);

    let timestamp = table_mut(doc, "timestamp", path)?;
    set(
        timestamp,
        "format",
        &version.timestamp.format,
        "timestamp",
        path,
    )?;
    set(
        timestamp,
        "last",
        &version.timestamp.last,
        "timestamp",
        path,
    )?;

    write_base(doc, version, path)?;

    let phase = table_mut(doc, "phase", path)?;
    set(phase, "separator", &version.phase.separator, "phase", path)?;
    set(phase, "name", &version.phase.name, "phase", path)?;
    set(phase, "delimiter", &version.phase.delimiter, "phase", path)?;
    set(
        phase,
        "distance",
        i64::from(version.phase.distance),
        "phase",
        path,
    )?;

    let suffix = table_mut(doc, "suffix", path)?;
    set(suffix, "mode", version.suffix.mode.as_str(), "suffix", path)?;
    set(
        suffix,
        "separator",
        &version.suffix.separator,
        "suffix",
        path,
    )?;

    let label = table_mut(doc, "label", path)?;
    set(
        label,
        "position",
        version.label.position.as_str(),
        "label",
        path,
    )?;

    Ok(())
}

pub fn report(verb: &str, path: &Path, version: &Version) -> Result<String, BumpError> {
    Ok(format!(
        "{verb} {} to {}",
        path.display(),
        print::to_string(version, &PrintSelection::default().with_timestamp())?
    ))
}

impl BumpFile {
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, BumpError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                BumpError::LogicError(format!(
                    "Configuration file not found at '{}'. Create one with 'bump init'",
                    path.display()
                ))
            } else {
                BumpError::IoError(err)
            }
        })?;

        let doc = content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

        let base_table = table(&doc, "base", path)?;
        parse_base_components(base_table, path)?;

        Ok(Self {
            path: path.to_path_buf(),
            doc,
        })
    }

    pub fn create(path: impl AsRef<Path>, force: bool) -> Result<Self, BumpError> {
        let path = path.as_ref();
        ensure_directory_exists(path)?;

        if path.exists() && !force {
            return Err(BumpError::LogicError(format!(
                "bumpfile already exists at '{}'; pass --force to overwrite",
                path.display()
            )));
        }

        let current_timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string();
        let content =
            include_str!("templates/bump.toml").replace("{timestamp}", &current_timestamp);

        fs::write(path, &content).map_err(BumpError::IoError)?;
        Self::parse(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn base_components(&self) -> Result<Vec<(String, u16)>, BumpError> {
        let base_table = table(&self.doc, "base", &self.path)?;
        Ok(parse_base_components(base_table, &self.path)?.1)
    }

    pub fn version(&self) -> Result<Version, BumpError> {
        version_from_doc(&self.doc, &self.path)
    }

    pub fn save(&mut self, version: &Version) -> Result<(), BumpError> {
        write_version_into_doc(&mut self.doc, version, &self.path)?;
        fs::write(&self.path, self.doc.to_string()).map_err(BumpError::IoError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tables_after_base() -> &'static str {
        r#"
[phase]
separator = "-"
name = ""
delimiter = "."
distance = 0

[suffix]
mode = "git_sha"
separator = "+"

[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"
last = "2026-01-01 00:00:00 UTC"

[label]
position = "after-base"
"#
    }

    fn write_bumpfile(body: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bump.toml");
        fs::write(&path, body).unwrap();
        (dir, path)
    }

    fn parse_err(path: &Path) -> String {
        match BumpFile::parse(path) {
            Ok(_) => panic!("expected parse error for {}", path.display()),
            Err(err) => err.to_string(),
        }
    }

    fn version_err(path: &Path) -> String {
        match BumpFile::parse(path).unwrap().version() {
            Ok(_) => panic!("expected version error for {}", path.display()),
            Err(err) => err.to_string(),
        }
    }

    #[test]
    fn parse_base_preserves_toml_order_and_skips_reserved() {
        let content = r#"
prefix = "v"

[base]
mode = "semver"
delimiter = "."
year = 2026
alpha = 2
month = 4
beta = 1
"#;
        let doc: DocumentMut = content.parse().unwrap();
        let path = Path::new("test.toml");
        let (_, components) =
            parse_base_components(doc.get("base").unwrap().as_table().unwrap(), path).unwrap();
        assert_eq!(
            components,
            vec![
                ("year".to_string(), 2026),
                ("alpha".to_string(), 2),
                ("month".to_string(), 4),
                ("beta".to_string(), 1),
            ]
        );
    }

    #[test]
    fn parse_version_save_roundtrip() {
        let body = format!(
            "prefix = \"v-\"\n\n[base]\ndelimiter = \".\"\nmajor = 0\nminor = 1\npatch = 0\n{}",
            tables_after_base()
        );
        let (_dir, path) = write_bumpfile(&body);
        let mut bumpfile = BumpFile::parse(&path).unwrap();
        let mut version = bumpfile.version().unwrap();
        assert_eq!(version.prefix, "v-");
        assert_eq!(
            version.base.components,
            vec![
                ("major".to_string(), 0),
                ("minor".to_string(), 1),
                ("patch".to_string(), 0),
            ]
        );
        version.bump("minor").unwrap();
        bumpfile.save(&version).unwrap();

        let reloaded = BumpFile::parse(&path).unwrap().version().unwrap();
        assert_eq!(
            reloaded.base.components,
            vec![
                ("major".to_string(), 0),
                ("minor".to_string(), 2),
                ("patch".to_string(), 0),
            ]
        );
    }

    #[test]
    fn missing_base_table() {
        let body = format!("prefix = \"v\"\n{}", tables_after_base());
        let (_dir, path) = write_bumpfile(&body);
        let err = parse_err(&path);
        assert!(err.contains("'base' table not found"), "{err}");
    }

    #[test]
    fn missing_prefix() {
        let body = format!(
            "[base]\ndelimiter = \".\"\nmajor = 0\n{}",
            tables_after_base()
        );
        let (_dir, path) = write_bumpfile(&body);
        let err = version_err(&path);
        assert!(
            err.contains("Expected key 'prefix' not found in [(root)]"),
            "{err}"
        );
    }

    #[test]
    fn non_integer_component() {
        let body = format!(
            "prefix = \"v\"\n\n[base]\ndelimiter = \".\"\nalpha = \"nope\"\n{}",
            tables_after_base()
        );
        let (_dir, path) = write_bumpfile(&body);
        let err = parse_err(&path);
        assert!(err.contains("Expected integer for [base].alpha"), "{err}");
    }

    #[test]
    fn bad_suffix_mode() {
        let body = r#"
prefix = "v"

[base]
delimiter = "."
major = 0

[phase]
separator = "-"
name = ""
delimiter = "."
distance = 0

[suffix]
mode = "nope"
separator = "+"

[timestamp]
format = "%Y"
last = "2020"

[label]
position = "after-base"
"#;
        let (_dir, path) = write_bumpfile(body);
        let err = version_err(&path);
        assert!(
            err.contains("Invalid suffix mode 'nope' (expected 'git_sha' or 'branch')"),
            "{err}"
        );
    }

    #[test]
    fn bad_label_position() {
        let body = format!(
            "prefix = \"v\"\n\n[base]\ndelimiter = \".\"\nmajor = 0\n{}",
            tables_after_base().replace("after-base", "middle")
        );
        let (_dir, path) = write_bumpfile(&body);
        let err = version_err(&path);
        assert!(err.contains("Invalid label position 'middle'"), "{err}");
    }

    #[test]
    fn missing_file() {
        let err = parse_err(Path::new("/tmp/bump-does-not-exist-xyz.toml"));
        assert!(err.contains("Configuration file not found"), "{err}");
    }
}
