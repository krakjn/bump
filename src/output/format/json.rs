use super::{Fields, json_escape, nested_pairs, nested_root_key};

pub(crate) fn render(fields: &Fields) -> String {
    let root = nested_root_key(fields);
    let pairs = nested_pairs(fields);
    let mut out = String::from("{\n  ");
    out.push_str(&json_escape(&root));
    out.push_str(": {\n");
    for (i, (key, value)) in pairs.iter().enumerate() {
        out.push_str("    ");
        out.push_str(&json_escape(key));
        out.push_str(": ");
        out.push_str(&json_escape(value));
        if i + 1 != pairs.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  }\n}\n");
    out
}
