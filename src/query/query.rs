extern crate serde;
extern crate serde_json;

use std::rc::Rc;

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct Query{
    pub jql: Rc<String>,
    startAt: usize,
    pub maxResults: usize,
    pub fields: Rc<Vec<String>>,
}

#[derive(Debug)]
pub enum QueryError {
    WrongQuery,
}

impl Query {
    pub fn new(query: String, max: usize, fields: Vec<String>) -> Query {
        Query {
            jql: Rc::new(query),
            startAt: 0,
            maxResults: max,
            fields: Rc::new(fields),
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

    pub fn create_remaining(&self, total: usize) -> Vec<Query> {
        let mut round = total / self.maxResults - 1;
        if total % self.maxResults > 0 {
            round += 1;
        }

        let mut remainings = Vec::new();
        for it in 1..=round {
            remainings.push(Query{
                jql: self.jql.clone(),
                startAt: it * self.maxResults,
                maxResults: self.maxResults,
                fields: self.fields.clone(),
            });
        }

        remainings
    }
}