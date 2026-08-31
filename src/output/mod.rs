mod format;

pub use format::{Case, render};

use crate::cmd::BumpError;
use crate::print::{self, PrintSelection};
use crate::version::Version;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    Raw,
    #[value(name = "c", alias = "C")]
    C,
    #[clap(skip)]
    CHeader,
    Go,
    Java,
    #[value(name = "csharp", alias = "CSharp")]
    CSharp,
    Python,
    Json,
    Toml,
    Yaml,
}

impl Format {
    const fn file_description(self) -> Option<&'static str> {
        match self {
            Self::CHeader => Some("C header file"),
            Self::Go => Some("Go source file"),
            Self::Java => Some("Java source file"),
            Self::CSharp => Some("C# source file"),
            Self::Python => Some("Python source file"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaseComponentField {
    pub key: String,
    pub case_name: String,
    pub value: u32,
}

#[derive(Debug, Clone)]
pub struct Fields {
    pub emit_prefix: String,
    pub base_components: Vec<BaseComponentField>,
    pub case_prefix: String,
    pub case_phase: String,
    pub case_phase_distance: String,
    pub case_string: String,
    pub case_timestamp: String,
    pub version_string: String,
    pub version_timestamp: String,
    pub version_prefix: String,
    pub version_phase: String,
    pub version_phase_distance: u32,
}

impl Fields {
    pub fn populate(emit_prefix: &str, case: Case, version: &Version) -> Result<Self, BumpError> {
        let base_components = version
            .base
            .components
            .iter()
            .map(|(key, value)| BaseComponentField {
                key: key.clone(),
                case_name: case.apply(&format!("VERSION_{}", key.to_uppercase())),
                value: u32::from(*value),
            })
            .collect();

        Ok(Self {
            emit_prefix: emit_prefix.to_string(),
            base_components,
            case_prefix: case.apply("VERSION_PREFIX"),
            case_phase: case.apply("VERSION_PHASE"),
            case_phase_distance: case.apply("VERSION_PHASE_DISTANCE"),
            case_string: case.apply("VERSION_STRING"),
            case_timestamp: case.apply("VERSION_TIMESTAMP"),
            version_string: print::to_string(version, &PrintSelection::default())?,
            version_timestamp: version.timestamp.last.clone(),
            version_prefix: version.prefix.clone(),
            version_phase: version.phase.name.clone(),
            version_phase_distance: version.phase.distance,
        })
    }
}

pub fn write(format: Format, fields: &Fields, path: &Path) -> Result<(), BumpError> {
    let content = render(format, fields)?;
    fs::write(path, content).map_err(BumpError::IoError)?;
    if let Some(desc) = format.file_description() {
        eprintln!("{desc} written to {}", path.display());
    } else {
        eprintln!("written to {}", path.display());
    }
    Ok(())
}
