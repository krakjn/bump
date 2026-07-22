use super::{Fields, substitute};
use crate::version::VersionMode;

const SEMVER: &str = r#"#define {emit_prefix}{case_prefix} "{version_prefix}"
#define {emit_prefix}{case_major} {version_major}
#define {emit_prefix}{case_minor} {version_minor}
#define {emit_prefix}{case_patch} {version_patch}
#define {emit_prefix}{case_phase} "{version_phase}"
#define {emit_prefix}{case_phase_distance} {version_phase_distance}
#define {emit_prefix}{case_string} "{version_string}"
#define {emit_prefix}{case_timestamp} "{version_timestamp}"
"#;

const CALVER: &str = r#"#define {emit_prefix}{case_string} "{version_string}"
#define {emit_prefix}{case_timestamp} "{version_timestamp}"
"#;

pub(crate) fn render(fields: &Fields) -> String {
    match fields.version_mode {
        VersionMode::Semver => substitute(SEMVER, fields),
        VersionMode::Calver => substitute(CALVER, fields),
    }
}
