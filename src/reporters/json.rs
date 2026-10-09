use serde::Serialize;

pub(super) fn pretty_json(value: &impl Serialize) -> String {
    serde_json::to_string_pretty(value).expect("plain structs always serialize") + "\n"
}
