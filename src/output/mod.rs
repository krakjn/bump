mod format;

pub use format::{Case, render};

use crate::cmd::BumpError;
use crate::print::{self, PrintOptions};
use crate::version::{Version, VersionMode};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Raw,
    C,
    Go,
    Java,
    CSharp,
    Python,
    Json,
    Toml,
    Yaml,
}

impl Format {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "raw" => Some(Self::Raw),
            "c" => Some(Self::C),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "csharp" => Some(Self::CSharp),
            "python" => Some(Self::Python),
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }

    const fn file_description(self) -> Option<&'static str> {
        match self {
            Self::C => Some("C header file"),
            Self::Go => Some("Go source file"),
            Self::Java => Some("Java source file"),
            Self::CSharp => Some("C# source file"),
            Self::Python => Some("Python source file"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fields {
    pub emit_prefix: String,
    pub case_prefix: String,
    pub case_major: String,
    pub case_minor: String,
    pub case_patch: String,
    pub case_phase: String,
    pub case_phase_distance: String,
    pub case_string: String,
    pub case_timestamp: String,
    pub version_string: String,
    pub version_timestamp: String,
    pub version_prefix: String,
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
    pub version_phase: String,
    pub version_phase_distance: u32,
    pub version_mode: VersionMode,
}

impl Fields {
    pub fn populate(emit_prefix: &str, case: Case, version: &Version) -> Result<Self, BumpError> {
        Ok(Self {
            emit_prefix: emit_prefix.to_string(),
            case_prefix: case.apply("VERSION_PREFIX"),
            case_major: case.apply("VERSION_MAJOR"),
            case_minor: case.apply("VERSION_MINOR"),
            case_patch: case.apply("VERSION_PATCH"),
            case_phase: case.apply("VERSION_PHASE"),
            case_phase_distance: case.apply("VERSION_PHASE_DISTANCE"),
            case_string: case.apply("VERSION_STRING"),
            case_timestamp: case.apply("VERSION_TIMESTAMP"),
            version_string: print::to_string(version, &PrintOptions::default())?,
            version_timestamp: version.timestamp.last.clone(),
            version_prefix: version.prefix.clone(),
            version_major: version.base.major,
            version_minor: version.base.minor.unwrap_or(0),
            version_patch: version.base.patch.unwrap_or(0),
            version_phase: version.phase.name.clone(),
            version_phase_distance: version.phase.distance,
            version_mode: version.base.mode,
        })
    }
}

pub fn write(format: Format, fields: &Fields, path: &Path) -> Result<(), BumpError> {
    let content = render(format, fields)?;
    fs::write(path, content).map_err(BumpError::IoError)?;
    if let Some(desc) = format.file_description() {
        println!("{desc} written to {}", path.display());
    } else {
        println!("written to {}", path.display());
    }
    Ok(())
}
