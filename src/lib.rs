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
use query::issue::Issue;

pub fn run() {
    let mut core = Core::new().unwrap();
    let login = Login::new().to_basic();

    //construct search
    let uri = "https://jiradc.int.net.nokia.com/rest/api/2/search";
    let search = create_initial_search();
    let mut fetcher = Fetcher::new(uri.to_string(), &login);
    fetcher.set_method(FetchMethod::Post);
    fetcher.set_body(search.to_json().unwrap());
    
    //seary first page
    let mut result: Option<Box<QueryResult>> = None;
    {
        let parser = |json: &str| { result = parse_query_result(&json);};
        let future = fetcher.fetch(&mut core, Some(parser));
        let _res = core.run(future);
    };

    //query all the remaining
    let mut issues = Vec::new();
    get_remaining_queries(&mut result, &search).iter().for_each(|qry|{
        let parser = |json: &str| {
            let paged = parse_query_result(&json);
            issues.extend(paged.unwrap().issues); 
        };
        let future = create_sub_fetch(&mut core, uri, &qry, &login, parser);
        let _res = core.run(future);
    });

    //merge query issue list
    let mut result_list = *(result.unwrap());
    result_list.issues.extend(issues);

    //dumping
    let total = result_list.issues.len();
    info!("Now we get {} issues in total!", total);

    //summarize
    let unsolved:Vec<&Issue> = result_list.issues.iter().filter(|it| !it.has_efforts()).collect();
    info!("Unsolved entries are: {}", unsolved.len());
    unsolved.iter().for_each(|it| it.log());

    let solved_eff:u32 = result_list.issues.iter()
        .filter(|it| it.has_efforts())
        .map(|it| it.get_efforts().unwrap())
        .fold(0, |acc, x| acc+x);
    info!("Solved efforts are: {} with {} features", solved_eff, total - unsolved.len());
    
}

fn create_initial_search() -> Query {
    let search_string = r#"project=FPB AND issuetype in ("Effort Estimation", "Entity Technical Analysis") AND "Competence Area" = "MANO MZ""#;
    Query::new(search_string.to_string(), 100, vec![
        String::from("summary"),
        String::from("customfield_38703"), //Title
        String::from("customfield_38692"), //EE
    ])
}

fn get_remaining_queries(result: &mut Option<Box<QueryResult>>, search: &Query) -> Vec<Query>{
    if let Some(qry_result) = result {
        info!("Got issues = {}, total = {}", qry_result.issues.len(),
            qry_result.total);
        search.create_remaining(qry_result.total)
    } else {
        error!("Unexpected result, query failure???!!!");
        vec![]
    }
}

fn create_sub_fetch<P>(core: &mut Core, uri: &str, search: &Query, login: &Basic, parser: P)
    -> impl Future where P: FnOnce(&str) -> (){
    let mut fetcher = Fetcher::new(uri.to_string(), &login);
    fetcher.set_method(FetchMethod::Post);
    fetcher.set_body(search.to_json().unwrap());
    fetcher.fetch(core, Some(parser))
}