pub(crate) mod search;

pub(crate) mod fs2issue;
pub mod fs2checker;

pub(crate) mod caissue;
pub mod cachecker;

#[cfg(test)]
mod test {
    extern crate serde;
    extern crate serde_json;
    use checkers::cachecker::get_leftmost;
    use checkers::caissue::*;

    #[test]
    fn should_trim_newlines() {
        assert_eq!(get_leftmost("some name \n really long", 10), "some name ");
    }

    #[test]
    fn should_trim_width_no_newline() {
        assert_eq!(get_leftmost("some name really lone", 10), "some name ");
    }

    #[test]
    fn should_trim_shorter_string_newline() {
        assert_eq!(get_leftmost("some\nname", 10), "some");        
    }

    #[test]
    fn should_trim_shorter_string_no_newline() {
        assert_eq!(get_leftmost("some name", 10), "some name");        
    }

    #[test]
    fn should_convert_parsed_fields() {
        let json = r#"{
            "expand" : "",
            "id": "",
            "self": "",
            "key": "",    
            "fields": {
                "summary":"Feature_ID_xxx_yyy",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "customfield_38750":"SW"
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        
        let converted = CAItem::from(&issue.unwrap());
        assert_eq!(converted.summary, "Feature_ID_xxx_yyy");
        assert_eq!(converted.feature_id, "Feature_ID");
        assert_eq!(converted.team, "Team yyy");
        assert_eq!(converted.start_fb, 1808);
        assert_eq!(converted.end_fb, 1809);
    }

    #[test]
    fn should_parse_desc() {
        let json = r#"{
            "expand" : "",
            "id": "",
            "self": "",
            "key": "",    
            "fields": {
                "summary":"Leading - something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "customfield_38750":"SW"
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let converted = CAItem::from(&issue.unwrap());
        
        let (sub, desc) = converted.get_summary();
        assert_eq!(sub, "Leading");
        assert_eq!(desc, "something else");
    }
}