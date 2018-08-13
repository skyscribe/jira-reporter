
extern crate hyper;
extern crate tokio_core;
use self::tokio_core::reactor::Core;
extern crate futures;

use fetch::fetch::{Fetcher, RequestInfo};
use query::query::Query;
use query::result::{parse_query_result, QueryResult};
use checkers::fs2issue::Fs2Issue;

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
    let mut result: Option<Box<Fs2Result>> = None;
    {
        let request_info = RequestInfo::post(SEARCH_URI, &search.to_json().unwrap());
        let parser = |json: &str| { result = parse_query_result(&json);};
        let future = fetcher.query_with(request_info, core, Some(parser));
        let _res = core.run(future);
    };

    //query all the remaining
    let mut issues = Vec::new();
    get_remaining_queries(&mut result, &search).iter().for_each(|qry|{
        let parser = |json: &str| {
            let paged = parse_query_result(&json);
            issues.extend(paged.unwrap().issues); 
        };
        let post_info = RequestInfo::post(SEARCH_URI, &qry.to_json().unwrap());
        let future = fetcher.query_with(post_info, core, Some(parser));
        let _res = core.run(future);
    });

    //merge query issue list
    let mut result_list = *(result.unwrap());
    result_list.issues.extend(issues);

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

fn get_remaining_queries(result: &mut Option<Box<Fs2Result>>, search: &Query) -> Vec<Query>{
    if let Some(qry_result) = result {
        info!("Got issues = {}, total = {}", qry_result.issues.len(),
            qry_result.total);
        search.create_remaining(qry_result.total)
    } else {
        error!("Unexpected result, query failure???!!!");
        vec![]
    }
}