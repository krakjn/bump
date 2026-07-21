use crate::cmd::{BumpError, ensure_directory_exists};
use crate::version::{Version, VersionMode};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Table, Value, value};

const INIT_TEMPLATE_TIMESTAMP: &str = "1970-01-01 00:00:00 UTC";

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
        .and_then(|item| item.as_table())
        .ok_or_else(|| bumpfile_parse_error(path, format!("'{section}' table not found")))
}

fn table_mut<'a>(
    doc: &'a mut DocumentMut,
    section: &str,
    path: &Path,
) -> Result<&'a mut Table, BumpError> {
    doc.get_mut(section)
        .and_then(|item| item.as_table_mut())
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

const SEMVER_KEYS: &[&str] = &["major", "minor", "patch"];
const CALVER_KEYS: &[&str] = &["year", "month", "day"];

fn present_keys<'a>(base: &Table, keys: &'a [&str]) -> Vec<&'a str> {
    keys.iter()
        .copied()
        .filter(|key| base.contains_key(key))
        .collect()
}

fn write_base(doc: &mut DocumentMut, version: &Version, path: &Path) -> Result<(), BumpError> {
    let base = table_mut(doc, "base", path)?;

    set(base, "mode", version.base.mode.as_str(), "base", path)?;
    set(base, "delimiter", &version.base.delimiter, "base", path)?;

    match version.base.mode {
        VersionMode::Calver => {
            // first remove old keys
            base.remove("major");
            base.remove("minor");
            base.remove("patch");
            if !base.contains_key("year") {
                base.insert("year", value(i64::from(version.base.major)));
            } else {
                set(base, "year", i64::from(version.base.major), "base", path)?;
            }
            // optional value, if None remove key to allow more flexible versioning
            if let Some(minor) = version.base.minor {
                if !base.contains_key("month") {
                    base.insert("month", value(i64::from(minor)));
                } else {
                    set(base, "month", i64::from(minor), "base", path)?;
                }
            } else {
                base.remove("month");
            }
            if let Some(patch) = version.base.patch {
                if !base.contains_key("day") {
                    base.insert("day", value(i64::from(patch)));
                } else {
                    set(base, "day", i64::from(patch), "base", path)?;
                }
            } else {
                base.remove("day");
            }
        }
        VersionMode::Semver => {
            base.remove("year");
            base.remove("month");
            base.remove("day");
            if !base.contains_key("major") {
                base.insert("major", value(i64::from(version.base.major)));
            } else {
                set(base, "major", i64::from(version.base.major), "base", path)?;
            }
            if let Some(minor) = version.base.minor {
                if !base.contains_key("minor") {
                    base.insert("minor", value(i64::from(minor)));
                } else {
                    set(base, "minor", i64::from(minor), "base", path)?;
                }
            } else {
                base.remove("minor");
            }
            if let Some(patch) = version.base.patch {
                if !base.contains_key("patch") {
                    base.insert("patch", value(i64::from(patch)));
                } else {
                    set(base, "patch", i64::from(patch), "base", path)?;
                }
            } else {
                base.remove("patch");
            }
        }
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

impl BumpFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, BumpError> {
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

        Ok(Self {
            path: path.to_path_buf(),
            doc,
        })
    }

    pub fn create(path: impl AsRef<Path>) -> Result<Self, BumpError> {
        let path = path.as_ref();
        ensure_directory_exists(path)?;

        let template = include_str!("templates/bump.toml");
        let template_version: Version = {
            let content = template.replace("{timestamp}", INIT_TEMPLATE_TIMESTAMP);
            toml::from_str(&content).expect("init template must deserialize")
        };
        let current_timestamp = chrono::Utc::now()
            .format(&template_version.timestamp.format)
            .to_string();
        let content = template.replace("{timestamp}", &current_timestamp);

        fs::write(path, &content).map_err(BumpError::IoError)?;
        let doc = content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

        Ok(Self {
            path: path.to_path_buf(),
            doc,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn version(&self) -> Result<Version, BumpError> {
        toml::from_str(&self.doc.to_string()).map_err(|err| {
            BumpError::ParseError(format!(
                "Failed to parse version from '{}': {err}. \
                Recreate your bumpfile with 'bump init'.",
                self.path.display()
            ))
        })
    }

    /// Warn when [base] keys don't match mode. Always returns `Ok` after printing.
    pub fn mismatch(&self) -> Result<(), BumpError> {
        let base = table(&self.doc, "base", &self.path)?;
        let mode = base
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or(VersionMode::Semver.as_str());

        let (wrong, rewrite) = if mode == VersionMode::Calver.as_str() {
            (
                present_keys(base, SEMVER_KEYS),
                "major/minor/patch → year/month/day",
            )
        } else {
            (
                present_keys(base, CALVER_KEYS),
                "year/month/day → major/minor/patch",
            )
        };

        if !wrong.is_empty() {
            eprintln!(
                "bump warning: [base].mode is {mode}, but found mismatched keys {wrong:?} in {}.\n\
                 On save, keys will be rewritten ({rewrite}).",
                self.path.display(),
            );
        }

        Ok(())
    }

    pub fn save(&mut self, version: &Version) -> Result<(), BumpError> {
        write_version_into_doc(&mut self.doc, version, &self.path)?;
        fs::write(&self.path, self.doc.to_string()).map_err(BumpError::IoError)
    }
}
