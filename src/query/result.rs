extern crate serde;
extern crate serde_json;

use self::serde::Deserialize;
use self::serde::de::DeserializeOwned;

#[derive(Deserialize, Clone)]
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

pub fn parse_query_result<T>(json: &str) -> Option<Box<QueryResult<T>>>
        where T: DeserializeOwned {
    let qry_result = serde_json::from_str::<QueryResult<T>>(&json);
    match qry_result {
        Ok(result) => Some(Box::new(result)),
        Err(err) => {
            error!("Parse json failed, err={}", err);
            None
        }
    } 
}

impl<T:DeserializeOwned> QueryResult<T> {
    //move fields from another one
    pub fn collect_from(&mut self, other: QueryResult<T>) {
        self.total = other.total;
        self.expand = other.expand;
        self.startAt = other.startAt;
        self.maxResults = other.maxResults;
        self.issues.extend(other.issues);
    }

    //generate default
    pub fn default(max: usize) -> QueryResult<T> {
        QueryResult {
            total: max,
            expand: "".to_string(),
            startAt: 0,
            maxResults: max,
            issues: Vec::new(),
        }
    }
} 