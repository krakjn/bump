use super::{Fields, json_escape, nested_pairs, nested_root_key};

pub(crate) fn render(fields: &Fields) -> String {
    let mut out = String::new();
    out.push_str(&nested_root_key(fields));
    out.push_str(":\n");
    for (key, value) in nested_pairs(fields) {
        out.push_str("  ");
        out.push_str(&key);
        out.push_str(": ");
        if value.is_empty() {
            out.push_str("\"\"");
        } else if value.contains(':') || value.contains('#') || value.contains('\n') {
            out.push_str(&json_escape(&value));
        } else {
            out.push_str(&value);
        }
        out.push('\n');
    }
    out
}
