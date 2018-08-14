
extern crate hyper;
extern crate tokio_core;
use std::rc::Rc;
use self::tokio_core::reactor::Core;
extern crate futures;

use fetch::fetch::{Fetcher, RequestInfo};
use query::query::Query;
use query::result::{parse_query_result, QueryResult};
use checkers::fs2issue::Fs2Issue;
use self::futures::future::{Future, Executor};
use std::cell::RefCell;

type Fs2Result = QueryResult<Fs2Issue>;
const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const FS2EE_SEARCH : &'static str = "project=FPB AND issuetype in (\"\
    Effort Estimation\", \"Entity Technical Analysis\") \
    AND \"Competence Area\" = \"MANO MZ\"";

//This has to match with query::issue::Fs2Fields
const FS2EE_FIELDS_SUMMARY  : &'static str = "summary";
const FS2EE_FIELDS_TITLE    : &'static str = "customfield_38703";
const FS2EE_FIELDS_EE       : &'static str = "customfield_38692";

pub fn run(core: &mut Core, fetcher: &mut Fetcher) {
    //construct search
    let fields = vec![FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_TITLE, FS2EE_FIELDS_EE]
                    .iter().map(|x| x.to_string()).collect();
    let search = Query::new(FS2EE_SEARCH.to_string(), 100, fields);

    //seary first page
    let result: Rc<RefCell<Option<Box<Fs2Result>>>> = Rc::new(RefCell::new(None));
    let request_info = RequestInfo::post(SEARCH_URI, &search.to_json().unwrap());
    let first_fetch = fetcher.query_with(request_info, core, Some(|json: &str| {
            *result.borrow_mut() = parse_query_result(&json);}));
    let _ = core.run(first_fetch);

    //search remaining
    for qry in get_remaining_queries(&*result.borrow(), &search){
        //query this page
        let cloned_result = Rc::clone(&result);
        let parser = move |json: &str| {
            let my_issues = parse_query_result(&json).unwrap().issues;
            if let Some(ref mut shared) = *cloned_result.borrow_mut() {
                shared.issues.extend(my_issues);
            }
        };
        let post_info = RequestInfo::post(SEARCH_URI, &qry.to_json().unwrap());
        let sub_fetch = fetcher.query_with(post_info, core, Some(parser)).map(|_| ()).map_err(|_| ());
        let _ = core.execute(sub_fetch); 
    }

    core.turn(None);   
    check_and_dump(&*result.borrow().as_ref().unwrap());
}

fn get_remaining_queries(result: &Option<Box<Fs2Result>>, search: &Query) -> Vec<Query>{
    if let Some(qry_result) = result {
        info!("Got issues = {}, total = {}", qry_result.issues.len(),
            qry_result.total);
        search.create_remaining(qry_result.total)
    } else {
        error!("Unexpected result, query failure???!!!");
        vec![]
    }
}

fn check_and_dump(result_list: &Fs2Result) {
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