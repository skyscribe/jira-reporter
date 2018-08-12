#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

pub mod fetch;
mod demo;
pub mod query;

extern crate hyper;
extern crate tokio_core;
use self::tokio_core::reactor::Core;
extern crate futures;

use fetch::fetch::{Fetcher, FetchMethod};
use fetch::login::Login;
use query::query::Query;
use query::result::{parse_query_result, QueryResult};

pub fn run() {
    let mut core = Core::new().unwrap();
    let login = Login::new().to_basic();

    //fetch a search result
    let uri = "https://jiradc.int.net.nokia.com/rest/api/2/search";
    let search_string = r#"project=FPB AND issuetype in ("Effort Estimation", "Entity Technical Analysis") AND "Competence Area" = "MANO MZ""#;
    let json = Query::new(search_string.to_string(), 100, vec![
        String::from("Summary"),
        String::from("FP Title"),
        String::from("Status"),
        String::from("FS2 EE (h)"),
    ]).to_json().unwrap();

    //construct query now
    let mut fetcher = Fetcher::new(uri.to_string(), &login);
    fetcher.set_method(FetchMethod::Post);
    fetcher.set_body(json);
    
    let mut result: Option<Box<QueryResult>> = None;
    {
        let parser = |json: &str| { result = parse_query_result(&json);};
        let future = fetcher.fetch(&mut core, Some(parser));
        let _res = core.run(future);
    };
    check_result(&result);
}

fn check_result(result: &Option<Box<QueryResult>>) {
    if let Some(qry_result) = result {
        info!("Got issues = {}, total = {}", qry_result.issues.len(),
            qry_result.total);
    } else {
        error!("Unexpected result!");
    }
}