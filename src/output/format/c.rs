use super::{Fields, base_int_lines, join_blocks, substitute};

pub(crate) fn render(fields: &Fields) -> String {
    let prefix = substitute("#define {emit_prefix}{case_prefix} \"{version_prefix}\"", fields);
    let base = base_int_lines(fields, |prefix, c| {
        format!("#define {prefix}{} {}", c.case_name, c.value)
    });
    let tail = substitute(
        r#"#define {emit_prefix}{case_phase} "{version_phase}"
#define {emit_prefix}{case_phase_distance} {version_phase_distance}
#define {emit_prefix}{case_string} "{version_string}"
#define {emit_prefix}{case_timestamp} "{version_timestamp}""#,
        fields,
    );
    join_blocks(&[&prefix, &base, &tail])
}
