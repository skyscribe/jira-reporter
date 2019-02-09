extern crate itertools;
extern crate serde_json;

use self::itertools::Itertools;
use self::serde_json::Value;
pub const NA_STRING: &str = "NA";

//Get a slice of the leftmost given characters
pub fn get_leftmost(raw: &str, total: usize) -> &str {
    let max = raw.find('\n').map_or(raw.len(), |x| x);
    if max > total {
        let mut end = total;
        //find next char boundary
        while !raw.is_char_boundary(end) {
            end += 1;
        }
        &raw[0..end]
    } else {
        &raw[0..max]
    }
}

/// field extraction utilities

//Get release lists
pub(crate) fn get_releases_from(release: &Value) -> String {
    match release {
        Value::Array(ref releases) => releases
            .iter()
            .map(|it| get_wrapped_object_attr(&it, "value").to_string())
            .filter(|it| it != "")
            .join(","),
        _ => "".to_string(),
    }
}

//Get wrapped object string
pub(crate) fn get_wrapped_object_attr<'a>(value: &'a Value, attr: &str) -> &'a str {
    match value {
        Value::Object(ref obj) => match obj[attr] {
            Value::String(ref x) => x,
            _ => "",
        },
        _ => "",
    }
}

pub(crate) fn get_wrapped_or_na(value: &Value) -> &str {
    get_wrapped_string(value, &NA_STRING)
}

pub(crate) fn get_wrapped_string<'a>(value: &'a Value, na: &'static str) -> &'a str {
    match value {
        Value::String(ref some) => some,
        _ => na,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_get_leftmost_handle_unicode_str() {
        assert_eq!(get_leftmost("Some", 2), "So");
        assert_eq!(get_leftmost("Some", 10), "Some");
        assert_eq!(get_leftmost("Löwe 老虎", 2), "Lö");
        assert_eq!(get_leftmost("Löwe 老虎", 5), "Löwe");
    }
}
