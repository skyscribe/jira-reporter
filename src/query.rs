extern crate serde;
extern crate serde_json;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Query{
    pub jql: String,
    startAt: usize,
    pub maxResults: usize,
    pub fields: Vec<String>,
}

#[derive(Debug)]
pub enum QueryError {
    WrongQuery,
}

impl Query {
    pub fn new(query: String, max: usize, fields: Vec<String>) -> Query {
        Query {
            jql: query,
            startAt: 0,
            maxResults: max,
            fields: fields,
        }
    }

    pub fn to_json(&self) -> Result<String, QueryError> {
        if let Ok(json) = serde_json::to_string(&self) {
            debug!("Query string as {}", json);
            Ok(json)
        } else {
            error!("Invalid query string specified!");
            Err(QueryError::WrongQuery)
        }
    }
}