extern crate serde;
extern crate serde_json;

use self::serde::Deserialize;

#[derive(Deserialize)]
#[allow(non_snake_case, dead_code)]
pub struct QueryResult<T> {
    //not used
    expand: String,
    
    pub startAt: usize,
    
    //not used
    maxResults: usize,

    //total records
    pub total: usize,

    //actual issue structure
    #[serde(bound(deserialize = "T:Deserialize<'de>"))]
    pub issues: Vec<T>,
}

pub fn parse_query_result<'de, T>(json: &'de str) -> Option<Box<QueryResult<T>>>
        where T: Deserialize<'de> {
    let qry_result = serde_json::from_str::<QueryResult<T>>(&json);
    match qry_result {
        Ok(result) => Some(Box::new(result)),
        Err(err) => {
            error!("Parse json failed, err={}", err);
            None
        }
    } 
}