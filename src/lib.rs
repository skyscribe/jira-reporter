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
use hyper::header::Basic;
use query::query::Query;
use query::result::{parse_query_result, QueryResult};
use futures::future::Future;

pub fn run() {
    let mut core = Core::new().unwrap();
    let login = Login::new().to_basic();

    //construct search
    let uri = "https://jiradc.int.net.nokia.com/rest/api/2/search";
    let search = create_initial_search();
    let mut fetcher = Fetcher::new(uri.to_string(), &login);
    fetcher.set_method(FetchMethod::Post);
    fetcher.set_body(search.to_json().unwrap());
    
    let mut result: Option<Box<QueryResult>> = None;
    {
        let parser = |json: &str| { result = parse_query_result(&json);};
        let future = fetcher.fetch(&mut core, Some(parser));
        let _res = core.run(future);
    };

    for qry in get_remaining_queries(&result, &search) {
        let future = create_sub_fetch(&mut core, uri, &qry, &login);
        let _res = core.run(future);
    }
}

fn create_initial_search() -> Query {
    let search_string = r#"project=FPB AND issuetype in ("Effort Estimation", "Entity Technical Analysis") AND "Competence Area" = "MANO MZ""#;
    Query::new(search_string.to_string(), 100, vec![
        String::from("Summary"),
        String::from("FP Title"),
        String::from("Status"),
        String::from("FS2 EE (h)"),
    ])
}

fn get_remaining_queries(result: &Option<Box<QueryResult>>, search: &Query) -> Vec<Query>{
    if let Some(qry_result) = result {
        info!("Got issues = {}, total = {}", qry_result.issues.len(),
            qry_result.total);
        search.create_remaining(qry_result.total)
    } else {
        error!("Unexpected result, query failure???!!!");
        vec![]
    }
}

fn create_sub_fetch(core: &mut Core, uri: &str, search: &Query, login: &Basic) -> impl Future {
    let mut fetcher = Fetcher::new(uri.to_string(), &login);
    fetcher.set_method(FetchMethod::Post);
    fetcher.set_body(search.to_json().unwrap());

    let parser = |_x: &str| (); //do nothing now, to be done
    fetcher.fetch(core, Some(parser))
}