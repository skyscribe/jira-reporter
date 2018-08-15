
extern crate hyper;
extern crate tokio_core;
use std::sync::mpsc::channel;
use self::tokio_core::reactor::Core;
extern crate futures;

use fetch::fetch::{Fetcher, RequestInfo};
use query::query::Query;
use query::result::{parse_query_result, QueryResult};
use checkers::fs2issue::Fs2Issue;
use self::futures::future::{join_all};

type Fs2Result = QueryResult<Fs2Issue>;
const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const FS2EE_SEARCH : &'static str = "project=FPB AND issuetype in (\"\
    Effort Estimation\", \"Entity Technical Analysis\") \
    AND \"Competence Area\" = \"MANO MZ\"";

//This has to match with query::issue::Fs2Fields
const FS2EE_FIELDS_SUMMARY  : &'static str = "summary";
const FS2EE_FIELDS_TITLE    : &'static str = "customfield_38703";
const FS2EE_FIELDS_EE       : &'static str = "customfield_38692";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    //construct search
    let fields = vec![FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_TITLE, FS2EE_FIELDS_EE]
                    .iter().map(|x| x.to_string()).collect();
    let search = Query::new(FS2EE_SEARCH.to_string(), 100, fields);

    //seary first page
    let (tx, rx) = channel();
    let request_info = RequestInfo::post(SEARCH_URI, &search.to_json().unwrap());
    let tx1 = tx.clone();
    let first_fetch = fetcher.query_with(request_info, core, Some(move |json: &str| {
            let qry_result : Option<Box<Fs2Result>> = parse_query_result(&json); 
            let _x = tx1.send(qry_result);
            info!("First respone parsed!");
    }));

    //collect response records
    let _x = core.run(first_fetch);
    let ref mut fs2_result = rx.recv().unwrap().unwrap();

    //search remaining
    let mut jobs = 0;
    let mut sub_queries = Vec::new();
    for qry in get_remaining_queries(fs2_result, &search){
        //query this page
        jobs += 1;
        let my_sender = tx.clone();
        let parser = move |json: &str| {
            let qry_result : Option<Box<Fs2Result>> = parse_query_result(&json); 
            let _x = my_sender.send(qry_result);
            info!("First paged response parsed!");
        };
        let post_info = RequestInfo::post(SEARCH_URI, &qry.to_json().unwrap());
        let sub_fetch = fetcher.query_with(post_info, core, Some(parser));
        sub_queries.push(sub_fetch);
    }
    let _x = core.run(join_all(sub_queries));

    //collect paged sub-queries
    while jobs > 0 {
        if let Ok(qry_result) = rx.recv() {
            fs2_result.issues.extend(qry_result.unwrap().issues);
        }
        jobs -= 1;
        info!("Collected a paged response, remaining jobs = {}", jobs);
    }
    check_and_dump(fs2_result);
}

fn get_remaining_queries(qry_result: &Box<Fs2Result>, search: &Query) -> Vec<Query>{
    info!("Got issues = {}, total = {}", qry_result.issues.len(),
        qry_result.total);
    search.create_remaining(qry_result.total)
}

pub fn check_and_dump(result_list: &Fs2Result) {
    //dumping
    let total = result_list.issues.len();
    info!("Now we get {} issues in total!", total);

    //summarize
    let unsolved:Vec<&Fs2Issue> = result_list.issues.iter().filter(|it| !it.has_efforts()).collect();
    info!("Unsolved entries are: {}", unsolved.len());
    unsolved.iter().for_each(|it| it.log());

    let solved_eff:u32 = result_list.issues.iter()
        .filter(|it| it.has_efforts())
        .map(|it| it.get_efforts().unwrap())
        .fold(0, |acc, x| acc+x);
    info!("Solved efforts are: {} with {} features", solved_eff, total - unsolved.len());
}