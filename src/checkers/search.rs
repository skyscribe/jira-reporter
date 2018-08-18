
extern crate hyper;
extern crate tokio_core;
use std::clone::Clone;
use std::sync::mpsc::channel;
use self::tokio_core::reactor::Core;
extern crate futures;

extern crate serde;
use self::serde::de::DeserializeOwned;

use fetch::fetch::{Fetcher, RequestInfo};
use query::query::Query;
use query::result::{parse_query_result, QueryResult};
use self::futures::future::join_all;

//generic search with paged response and collect them in a generic/strongly typed manner
pub fn perform_gen<T>(core: &mut Core, fetcher: &mut Fetcher, uri: &str, jql: &str,
        fields: Vec<String>, result: &mut QueryResult<T>) 
        where T: DeserializeOwned + Clone {
    //construct search
    let search = Query::new(jql.to_string(), 100, fields);
    let mut first_job = vec![search.clone()];
    search_and_collect(result, &mut first_job, core, fetcher, &uri);
    if result.issues.len() == 0 {
        error!("First search failed?");
        panic!("Unexpected ending!");
    }

    info!("Got first result now, check remaining by page info!");
    //search remaining and combine the result
    let mut pending_jobs = get_remaining_queries(result, &search);
    search_and_collect(result, &mut pending_jobs, core, fetcher, &uri);
}

//run given queries repeatedly until all results got
fn search_and_collect<T:DeserializeOwned>(result: &mut QueryResult<T>, pending_jobs: &mut Vec<Query>, 
        core: &mut Core, fetcher: &mut Fetcher, uri: &str){
    let (guard_tx, guard_rx) = channel();

    while pending_jobs.len() > 0 {
        let mut sub_queries = Vec::new();

        //Drain all pending jobs 
        for qry in pending_jobs.iter() {
            //query this page
            let my_job = qry.startAt;
            let my_guard = guard_tx.clone();
            let parser = move |json: &str| {
                let my_result = parse_query_result::<T>(&json).ok_or_else(|| my_job);
                let _x = my_guard.send(my_result);
            };

            let post_info = RequestInfo::post(uri, &qry.to_json().unwrap());
            let sub_fetch = fetcher.query_with(post_info, core, Some(parser));
            sub_queries.push(sub_fetch);
        }
        let _x = core.run(join_all(sub_queries));

        //collect paged sub-queries
        let mut unfinished: Vec<usize> = Vec::new();
        let total = pending_jobs.len();
        for  x in 1..(total+1) {
            if let Ok(process_result) = guard_rx.recv() {
                match process_result {
                    Ok(qry) => {
                        result.collect_from(*qry);
                        info!("[{}/{}] Collected a paged response!", x, total);
                    },
                    Err(job) => {
                        unfinished.push(job.clone());
                        info!("[{}/{}] unexpected response, would retry this", x, total);
                    },
                }
            }
        }

        //check failures and retain them for retry
        pending_jobs.retain(|ref qry| {
            unfinished.sort_unstable();
            unfinished.binary_search(&qry.startAt).is_ok()
        });
        info!("Total retry jobs count = {}", pending_jobs.len());
    }
} 

//Create remaining queries list based on first paged search result
fn get_remaining_queries<T>(qry_result: &QueryResult<T>, search: &Query) -> Vec<Query>{
    info!("Got issues = {}, total = {}", qry_result.issues.len(), qry_result.total);
    search.create_remaining(qry_result.total)
}