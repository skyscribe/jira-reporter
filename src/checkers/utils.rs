//Get a slice of the leftmost given characters
pub fn get_leftmost(raw: &str, total: usize) -> &str {
    let max = raw.find("\n").map_or(raw.len(), |x| x);
    if max > total {
        &raw[0..total]
    } else {
        &raw[0..max]
    }
}

