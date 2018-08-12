extern crate serde;
extern crate serde_json;

use query::issue::Issue;

#[derive(Deserialize)]
#[allow(non_snake_case, dead_code)]
pub struct QueryResult{
    //not used
    expand: String,
    
    pub startAt: usize,
    
    //not used
    maxResults: usize,

    pub total: usize,
    pub issues: Vec<Issue>,
}

pub fn parse_query_result(json: &str) -> Option<Box<QueryResult>> {
    let qry_result = serde_json::from_str::<QueryResult>(&json);
    match qry_result {
        Ok(result) => Some(Box::new(result)),
        Err(err) => {
            error!("Parse json failed, err={}", err);
            None
        }
    } 
}