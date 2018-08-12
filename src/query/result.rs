extern crate serde;
extern crate serde_json;

use query::issue::Issue;

#[derive(Deserialize)]
#[allow(non_snake_case, dead_code)]
pub struct QueryResult{
    expand: String,
    pub startAt: usize,
    maxResults: usize,
    pub total: usize,
    issues: Vec<Issue>,
}

pub fn parse_query_result(json: &str) -> Option<QueryResult> {
    let qry_result = serde_json::from_str::<QueryResult>(&json);
    match qry_result {
        Ok(result) => Some(result),
        Err(err) => {
            error!("Parse json failed, err={}", err);
            None
        }
    } 
}