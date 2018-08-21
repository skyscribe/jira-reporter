pub(crate) mod search;

pub(crate) mod fs2issue;
pub mod fs2checker;

pub(crate) mod caissue;
pub mod cachecker;
pub(crate) mod utils;

#[cfg(test)]
mod test {
    extern crate serde;
    extern crate serde_json;
    use checkers::caissue::*;
    use checkers::utils::*;

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
                "customfield_38750":{ "value": "SW"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        
        let converted = CAItem::from(&issue.unwrap());
        assert_eq!(converted.summary, "Feature_ID_xxx_yyy");
        assert_eq!(converted.feature_id, "Feature_ID");
        assert_eq!(converted.team, "Team yyy");
        assert_eq!(converted.start_fb, 1808);
        assert_eq!(converted.end_fb, 1809);
        assert_eq!(converted.activity, Activity::SW);
    }

    #[test]
    fn should_parse_desc_by_space() {
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
                "customfield_38750":{ "value": "SW"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let converted = CAItem::from(&issue.unwrap());
        
        let (sub, desc) = converted.get_summary();
        assert_eq!(sub, "Leading");
        assert_eq!(desc, "something else");
    }

    #[test]
    fn parse_desc_by_special_chars() {
        let json = r#"{
            "expand" : "",
            "id": "",
            "self": "",
            "key": "",    
            "fields": {
                "summary":"Leading\t \t something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy\t\n",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "customfield_38750":{ "value": "SW"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let converted = CAItem::from(&issue.unwrap());
        
        let (sub, desc) = converted.get_summary();
        assert_eq!(sub, "Leading");
        assert_eq!(desc, "something else"); 
        assert_eq!(converted.team, "Team yyy");
    }

    #[test]
    fn should_translate_efs_type() {
        let json = r#"{"expand" : "", "id": "", "self": "", "key": "", "fields": {
                "summary":"Leading - something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "customfield_38750":{"value": "Entity Specification"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        assert_eq!(CAItem::from(&issue.unwrap()).activity, Activity::EFS);   
    }

    #[test]
    fn should_translate_et_type() {
        let json = r#"{"expand" : "", "id": "", "self": "", "key": "", "fields": {
                "summary":"Leading - something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "customfield_38750":{ "value": "Entity Testing"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        assert_eq!(CAItem::from(&issue.unwrap()).activity, Activity::ET);   
    }

    #[test]
    fn should_compare_caitem_by_given_order() {
        use std::cmp::Ordering;
        let json = r#"{"expand" : "", "id": "", "self": "", "key": "", "fields": {
                "summary":"Leading - something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "customfield_38750":{ "value": "EFS"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let item = CAItem::from(&issue.unwrap());
        
        let mut item1 = item.clone();
        item1.activity = Activity::SW;
        assert_eq!(item.cmp(&item1), Ordering::Less);

        let mut item2 = item1.clone();
        item2.start_fb = 1809;
        assert!(item1 < item2);

        let mut item3 = item2.clone();
        item3.end_fb = 1810;
        assert!(item2 < item3);

        let mut item4 = item3.clone();
        item4.activity = Activity::ET;
        assert!(item3 < item4);
    }
}