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
    let n = table
        .get(key)
        .and_then(Item::as_integer)
        .ok_or_else(|| {
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
        if BASE_RESERVED.contains(&key.as_ref()) {
            continue;
        }
        let n = value.as_integer().ok_or_else(|| {
            bumpfile_parse_error(path, format!("Expected integer for [base].{key}"))
        })?;
        if !(0..=i64::from(u16::MAX)).contains(&n) {
            return Err(bumpfile_parse_error(
                path,
                format!(
                    "Expected [base].{key} in range 0..={}",
                    u16::MAX
                ),
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
        .ok_or_else(|| {
            bumpfile_parse_error(path, "Expected key 'prefix' not found in [(root)]")
        })?
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
        let content = include_str!("templates/bump.toml").replace("{timestamp}", &current_timestamp);

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

    #[test]
    fn parse_base_preserves_toml_order() {
        let content = r#"
prefix = "v"

[base]
delimiter = "."
alpha = 2
beta = 0

[phase]
separator = "-"
name = ""
delimiter = "."
distance = 0

[suffix]
mode = "git_sha"
separator = "+"

[timestamp]
format = "%Y"
last = "2020"

[label]
position = "after-base"
"#;
        let doc: DocumentMut = content.parse().unwrap();
        let path = Path::new("test.toml");
        let (_, components) = parse_base_components(doc.get("base").unwrap().as_table().unwrap(), path).unwrap();
        assert_eq!(components, vec![("alpha".to_string(), 2), ("beta".to_string(), 0)]);
    }
}
