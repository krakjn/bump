use crate::bump::BumpError;
use crate::compose::{self, ComposeOptions};
use crate::version::{Version, VersionMode};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    C,
    Go,
    Java,
    CSharp,
    Python,
}

impl Language {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "c" => Some(Self::C),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "csharp" => Some(Self::CSharp),
            "python" => Some(Self::Python),
            _ => None,
        }
    }

    const fn file_description(self) -> &'static str {
        match self {
            Self::C => "C header file",
            Self::Go => "Go source file",
            Self::Java => "Java source file",
            Self::CSharp => "C# source file",
            Self::Python => "Python source file",
        }
    }

    fn template(self, mode: VersionMode) -> &'static str {
        match (self, mode) {
            (Self::C, VersionMode::Semver) => include_str!("templates/c/semver.h"),
            (Self::C, VersionMode::Calver) => include_str!("templates/c/calver.h"),
            (Self::Go, VersionMode::Semver) => include_str!("templates/go/semver.go"),
            (Self::Go, VersionMode::Calver) => include_str!("templates/go/calver.go"),
            (Self::Java, VersionMode::Semver) => include_str!("templates/java/semver.java"),
            (Self::Java, VersionMode::Calver) => include_str!("templates/java/calver.java"),
            (Self::CSharp, VersionMode::Semver) => include_str!("templates/csharp/semver.cs"),
            (Self::CSharp, VersionMode::Calver) => include_str!("templates/csharp/calver.cs"),
            (Self::Python, VersionMode::Semver) => include_str!("templates/python/semver.py"),
            (Self::Python, VersionMode::Calver) => include_str!("templates/python/calver.py"),
        }
    }
}

struct OutputFields {
    version_string: String,
    timestamp: String,
    prefix: String,
    major: u32,
    minor: u32,
    patch: u32,
    phase: String,
}

fn output_fields(version: &Version) -> Result<OutputFields, BumpError> {
    Ok(OutputFields {
        version_string: compose::to_string(version, &ComposeOptions::default())?,
        timestamp: version.timestamp.last.clone(),
        prefix: version.prefix.clone(),
        major: version.base.major,
        minor: version.base.minor.unwrap_or(0),
        patch: version.base.patch.unwrap_or(0),
        phase: version.phase.name.clone(),
    })
}

fn render_calver(tmpl: &str, f: &OutputFields) -> String {
    tmpl.replace("{version_string}", &f.version_string)
        .replace("{timestamp}", &f.timestamp)
}

fn render_semver(tmpl: &str, f: &OutputFields) -> String {
    tmpl.replace("{prefix}", &f.prefix)
        .replace("{major}", &f.major.to_string())
        .replace("{minor}", &f.minor.to_string())
        .replace("{patch}", &f.patch.to_string())
        .replace("{phase}", &f.phase)
        .replace("{version_string}", &f.version_string)
        .replace("{timestamp}", &f.timestamp)
}

fn write_output(lang: Language, path: &Path, content: String) -> Result<(), BumpError> {
    fs::write(path, content).map_err(BumpError::IoError)?;
    println!("{} written to {}", lang.file_description(), path.display());
    Ok(())
}

pub fn render(lang: Language, version: &Version) -> Result<String, BumpError> {
    let fields = output_fields(version)?;
    let mode = version.base.mode;
    let tmpl = lang.template(mode);
    Ok(match mode {
        VersionMode::Calver => render_calver(tmpl, &fields),
        VersionMode::Semver => render_semver(tmpl, &fields),
    })
}

pub fn output_file(lang: Language, version: &Version, path: &Path) -> Result<(), BumpError> {
    write_output(lang, path, render(lang, version)?)
}
