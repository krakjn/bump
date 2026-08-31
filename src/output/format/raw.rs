use super::{Fields, base_int_lines, join_blocks, substitute};

pub(crate) fn render(fields: &Fields) -> String {
    let prefix = substitute("{emit_prefix}{case_prefix}=\"{version_prefix}\"", fields);
    let base = base_int_lines(fields, |prefix, c| {
        format!("{prefix}{}={}", c.case_name, c.value)
    });
    let tail = substitute(
        r#"{emit_prefix}{case_phase}="{version_phase}"
{emit_prefix}{case_phase_distance}={version_phase_distance}
{emit_prefix}{case_string}="{version_string}"
{emit_prefix}{case_timestamp}="{version_timestamp}""#,
        fields,
    );
    join_blocks(&[&prefix, &base, &tail])
}
