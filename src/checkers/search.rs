
extern crate hyper;
extern crate tokio_core;
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
        fields: Vec<String>) -> Box<QueryResult<T>> where T: DeserializeOwned {
    //construct search
    let search = Query::new(jql.to_string(), 100, fields);

    //seary first page
    let (tx, rx) = channel();
    let request_info = RequestInfo::post(uri, &search.to_json().unwrap());
    let tx1 = tx.clone();
    let first_fetch = fetcher.query_with(request_info, core, Some(move |json: &str| {
            let qry_result = parse_query_result::<T>(&json); 
            let _x = tx1.send(qry_result);
            info!("First respone parsed!");
    }));

    //collect response records
    let _x = core.run(first_fetch);
    let mut search_result = rx.recv().unwrap().unwrap();

    //search remaining
    let (guard_tx, guard_rx) = channel();
    let mut pending_jobs = get_remaining_queries(&search_result, &search);

    while pending_jobs.len() > 0 {
        let mut sub_queries = Vec::new();

        //Drain all pending jobs 
        for qry in pending_jobs.iter() {
            //query this page
            let my_job = qry.startAt;
            let my_guard = guard_tx.clone();
            let parser = move |json: &str| {
                let my_result = parse_query_result::<T>(&json).ok_or_else(||{
                    warn!("Job {} failed!", my_job);
                    my_job                  
                });
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
                        search_result.issues.extend(qry.issues);
                        info!("[{}/{}] Collected a paged response and collected", x, total);
                    },
                    Err(job) => {
                        unfinished.push(job.clone());
                        info!("[{}/{}] unexpected response, would retry this", x, total);
                    },
                }
            }

        }

        //check failures and retry them
        pending_jobs.retain(|ref qry| unfinished.binary_search(&qry.startAt).is_ok());
    }

    return search_result;
}

//Create remaining queries list based on first paged search result
fn get_remaining_queries<T>(qry_result: &Box<QueryResult<T>>, search: &Query) -> Vec<Query>{
    info!("Got issues = {}, total = {}", qry_result.issues.len(),
        qry_result.total);
    search.create_remaining(qry_result.total)
}