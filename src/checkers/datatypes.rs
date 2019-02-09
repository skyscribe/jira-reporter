pub trait ParsedData {
    //get field lists
    fn get_field_list() -> Vec<String>;
}

pub trait StoredData {
    type Parsed;
    //translate from parsed data type
    fn parse_from(parsed: &Self::Parsed) -> Self;
}
