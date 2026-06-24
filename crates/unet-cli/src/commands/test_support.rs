use serde_json::Value as JsonValue;

pub(crate) fn parse_json_value(json_str: &str) -> serde_json::Result<JsonValue> {
    serde_json::from_str(json_str)
}

pub(crate) fn expect_json_object(json_str: &str) -> JsonValue {
    let value = parse_json_value(json_str).expect("json should parse");
    assert!(value.is_object());
    value
}

pub(crate) fn expect_json_parse_error(json_str: &str) {
    assert!(parse_json_value(json_str).is_err());
}

pub(crate) fn pagination_values(page: u64, per_page: u64) -> (usize, usize) {
    let offset = usize::try_from((page - 1) * per_page)
        .expect("pagination offset should fit in usize");
    let limit = usize::try_from(per_page).expect("pagination limit should fit in usize");
    (offset, limit)
}
