use super::{Fields, json_escape, nested_pairs, nested_root_key};

pub fn render(fields: &Fields) -> String {
    let mut out = String::new();
    out.push('[');
    out.push_str(&nested_root_key(fields));
    out.push_str("]\n");
    for (key, value) in nested_pairs(fields) {
        out.push_str(&key);
        out.push_str(" = ");
        out.push_str(&json_escape(&value));
        out.push('\n');
    }
    out
}
