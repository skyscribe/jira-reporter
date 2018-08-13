pub mod query;
pub mod result;

// Tests for this module
#[cfg(test)]
mod test {
    use super::query::*;
    use super::result::*;
    use checkers::issue::Fs2Issue;

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

        assert_eq!(base_json.replace(r#"startAt":0"#, r#"startAt":100"#), 
            remainings[0].to_json().unwrap());
        assert_eq!(base_json.replace(r#"startAt":0"#, r#"startAt":200"#), 
            remainings[1].to_json().unwrap());
    }

    #[test]
    fn shall_parse_search_result() {
        let json = r#"{
            "expand" : "schema, names",
            "startAt" : 0,
            "maxResults" : 3,
            "total" : 179,
            "issues" : [
                {
                    "expand": "operations, versionedRepresentations, editmeta, changelog, renderedFields",
                    "id" : "2969769",
                    "self" : "https://jiradc.int.net.nokia.com/rest/api/2/issue/2969769",
                    "key" : "FPB-12512",
                    "fields" : {
                        "summary":"5GC001000-EE-MANO MZ",
                        "customfield_38692":60.0,
                        "customfield_38703":"20 MHz cell bandwidth for cmWave"
                    }
                },
                {
                    "expand": "operations, versionedRepresentations, editmeta, changelog, renderedFields",
                    "id" : "2969747",
                    "self" : "https://jiradc.int.net.nokia.com/rest/api/2/issue/2969747",
                    "key" : "FPB-12490",
                    "fields" : {
                        "summary":"5GC0010xxx-EE-MANO MZ",
                        "customfield_38692":null,
                        "customfield_38703":"What ever in fp"
                    }
                },
                {
                    "expand": "operations, versionedRepresentations, editmeta, changelog, renderedFields",
                    "id" : "2969704",
                    "self" : "https://jiradc.int.net.nokia.com/rest/api/2/issue/2969704",
                    "key" : "FPB-12447",
                    "fields" : {
                        "summary":"5GC0010xxx-EE-MANO MZ",
                        "customfield_38692":0.0,
                        "customfield_38703":"What ever in fp"
                    }
                }
            ]
        }"#;

        let qry_result: Box<QueryResult<Fs2Issue>> = parse_query_result(json).unwrap();
        assert_eq!(qry_result.total, 179);
        assert_eq!(qry_result.startAt, 0);
        let issues = (*qry_result).issues;
        assert_eq!(issues.len(), 3);

        let ees : Vec<Option<u32>> = issues.iter().map(|it| it.get_efforts()).collect();
        assert_eq!(ees, vec![Some(60), None, Some(0)]);
    }
}