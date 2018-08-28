pub mod cachecker;
pub (crate) mod caissue;
pub (crate) mod caitem;
pub (crate) mod timeline;
pub (crate) mod carecords;

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    extern crate serde;
    extern crate serde_json;
    use super::caissue::*;
    use super::caitem::*;
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
                "timeoriginalestimate":24000,
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
                "timeoriginalestimate":360000,
                "customfield_38750":{ "value": "SW"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let converted = CAItem::from(&issue.unwrap());
        
        assert_eq!(converted.sub_id, "Leading");
        assert_eq!(converted.description, "something else");
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
                "timeoriginalestimate":360000,
                "customfield_38750":{ "value": "SW"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let converted = CAItem::from(&issue.unwrap());
        
        assert_eq!(converted.sub_id, "Leading");
        assert_eq!(converted.description, "something else"); 
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
                "timeoriginalestimate":360000,
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
                "timeoriginalestimate":360000,
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
                "timeoriginalestimate":360000,
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

    #[test]
    fn should_compare_by_subfid_for_same_feature() {
        use std::cmp::Ordering;
        let json = r#"{"expand" : "", "id": "", "self": "", "key": "", "fields": {
                "summary":"Feature-A-a - something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "timeoriginalestimate":360000,
                "customfield_38750":{ "value": "EFS"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let item = CAItem::from(&issue.unwrap());

        let mut item1 = item.clone();
        item1.summary = "Feature-A-b - something else".to_string();
        item1.reparse();

        let mut item2 = item.clone();
        item2.summary = "Feature-A-a - something else".to_string();
        item2.activity = Activity::SW;
        item2.reparse();

        assert_eq!(item.cmp(&item1), Ordering::Less);
        assert_eq!(item.cmp(&item2), Ordering::Less);
        assert_eq!(item1.cmp(&item2), Ordering::Greater);
    }

    #[test]
    fn should_sort_type_first_when_schedule_is_wrong() {
        use std::cmp::Ordering;
        let json = r#"{"expand" : "", "id": "", "self": "", "key": "", "fields": {
                "summary":"Feature-A-a - something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "timeoriginalestimate":360000,
                "customfield_38750":{ "value": "SW"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let item = CAItem::from(&issue.unwrap());

        let mut item1 = item.clone();
        item1.end_fb = 1810;
        item1.activity = Activity::EFS;
        assert_eq!(item.cmp(&item1), Ordering::Greater);
    }

    #[test]
    fn should_normalize_fid_dup_cp3() {
        parse_and_check_against("Feature-A-a-CP3 something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_efs() {
        parse_and_check_against("Feature-A-a-EFS something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_om_cp3() {
        parse_and_check_against("Feature-A-a-OM-CP3 something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_oam() {
        parse_and_check_against("Feature-A-a-OAM something else",
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_om() {
        parse_and_check_against("Feature-A-a-OM something else",
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_cfam() {
        parse_and_check_against("Feature-A-a-CFAM-xx something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_Ei() {
        parse_and_check_against("Feature-A-a-Ei something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_EI() {
        parse_and_check_against("Feature-A-a-EI something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_with_ending_dash() {
        parse_and_check_against("Feature-A-a2- something else", 
            "Feature-A-a2", "something else");
    }

    #[test]
    fn should_normalize_fid_with_ending_colon() {
        parse_and_check_against("Feature-A-a2: something else", 
            "Feature-A-a2", "something else");
    }

    #[test]
    fn should_normalize_fid_with_leading_spaces() {
        parse_and_check_against("      Feature-A-a2 something else", 
            "Feature-A-a2", "something else");
    }

    #[test]
    fn should_normalize_fid_with_no_desc() {
        parse_and_check_against(" Feature-A-OM-CP3",
            "Feature-A", "");
    }

    fn parse_and_check_against(summary: &str, expected: &str, trailing: &str) {
        let summary = String::from(summary);
        let (subid, desc) = CAItem::get_summary(&summary);
        assert_eq!(subid, expected);
        assert_eq!(desc, trailing);
    }
}