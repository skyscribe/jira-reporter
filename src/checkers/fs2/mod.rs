pub mod fs2checker;
pub(crate) mod fs2issue;
pub(crate) mod fs2item;

#[cfg(test)]
mod test {
    use super::fs2issue::Fs2Issue;
    use crate::query::result::*;

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
                        "status" : {"name": "Done"},
                        "description": "some desc",
                        "customfield_38724" : [{"value":"rel1"}],
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
                        "status" : {"name": "Done"},
                        "description": "some desc",
                        "customfield_38724" : [{"value":"rel1"}],
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
                        "status" : {"name": "Done"},
                        "description": "some desc",
                        "customfield_38724" : [{"value":"rel1"}],
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

        let ees: Vec<Option<u32>> = issues.iter().map(|it| it.get_efforts()).collect();
        assert_eq!(ees, vec![Some(60), None, Some(0)]);
    }
}
