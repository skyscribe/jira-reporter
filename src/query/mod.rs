pub mod batch;
pub mod issue;
pub mod result;

// Tests for this module
#[cfg(test)]
mod test {
    use super::batch::Query;

    fn create_query() -> (Query, String) {
        (
            Query::new("Project = FPB".to_string(), 100, vec![
            String::from("summary"), 
            String::from("status"), 
            String::from("assignee")
            ]),
            r#"{"jql":"Project = FPB","startAt":0,"maxResults":100,"fields":["summary","status","assignee"]}"#
            .to_string()
        )
    }

    #[test]
    fn shall_create_correct_query() {
        let (qry, expected_json) = create_query();
        if let Ok(json) = qry.to_json() {
            assert_eq!(expected_json, json);
        } else {
            assert!(false, "query failed!");
        }
    }

    #[test]
    fn shall_create_paged_remaining() {
        let (qry, base_json) = create_query();
        let remainings = qry.create_remaining(300);
        assert_eq!(remainings.len(), 2);

        assert_eq!(
            base_json.replace(r#"startAt":0"#, r#"startAt":100"#),
            remainings[0].to_json().unwrap()
        );
        assert_eq!(
            base_json.replace(r#"startAt":0"#, r#"startAt":200"#),
            remainings[1].to_json().unwrap()
        );
    }
}
