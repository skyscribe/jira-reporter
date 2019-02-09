use std::clone::Clone;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use tokio_core::reactor::Core;

use hyper::StatusCode;
use serde::de::DeserializeOwned;

use crate::fetch::fetcher::{Fetcher, RequestInfo};
use crate::query::batch::Query;
use crate::query::result::{parse_query_result, QueryResult};
use futures::future::{join_all, Future};

pub struct Searcher<'a> {
    core: &'a mut Core,
    fetcher: &'a mut Fetcher,
    uri: &'a str,
    pending_jobs: Vec<Query>,
    finished: Vec<usize>,
}

impl<'a> Searcher<'a> {
    pub fn new(
        core: &'a mut Core,
        fetcher: &'a mut Fetcher,
        uri: &'a str,
        pending: Vec<Query>,
    ) -> Searcher<'a> {
        Searcher {
            core,
            fetcher,
            uri,
            pending_jobs: pending,
            finished: Vec::new(),
        }
    }

    //Search by given jql and issue fields, and collect all results in one single
    // result, 2-phases based search is used to calculating paging properly.
    pub fn perform<T: DeserializeOwned>(
        &mut self,
        jql: &str,
        fields: Vec<String>,
        result: &mut QueryResult<T>,
    ) {
        let search = Query::new(jql.to_string(), 100, fields);
        //first search
        self.reset_pending(vec![search.clone()])
            .perform_parallel(result);
        if result.issues.is_empty() {
            error!("First search failed?");
            panic!("Unexpected ending!");
        }

        //remaining
        info!("Got first result now, check remaining by page info!");
        self.reset_pending(search.create_remaining(result.total))
            .perform_parallel(result);
    }

    fn reset_pending(&mut self, new_pending: Vec<Query>) -> &mut Searcher<'a> {
        self.pending_jobs = new_pending;
        self
    }

    fn perform_parallel<T: DeserializeOwned>(&mut self, result: &mut QueryResult<T>) {
        let (tx, rx) = channel();
        while self.pending_jobs.is_empty() {
            self.finished.clear();
            self.drain_all_jobs(&tx);
            self.collect_all_responses(result, &rx);
            self.clean_finished_from_pending();
        }
    }

    fn drain_all_jobs<T: DeserializeOwned>(
        &mut self,
        sender: &Sender<Result<Box<QueryResult<T>>, usize>>,
    ) {
        let mut sub_queries = Vec::new();
        //Drain all pending jobs
        for qry in self.pending_jobs.iter() {
            //query this page
            let sender1 = sender.clone();
            let parser = move |json: &str, code: StatusCode| {
                let my_result = match code {
                    StatusCode::Ok => parse_query_result::<T>(&json).ok_or_else(|| qry.startAt),
                    _ => Err(qry.startAt),
                };
                let _x = sender1.send(my_result);
            };

            let post_info = RequestInfo::post(self.uri, &qry.to_json().unwrap());
            let guard1 = sender.clone();
            let sub_fetch = self
                .fetcher
                .query_with(post_info, self.core, Some(parser))
                .map_err(move |err| {
                    //TODO: handle exceptions in graceful manner?
                    warn!("This job {} has failed by {}", qry.startAt, err);
                    let _x = guard1.send(Err(qry.startAt + 10000));
                    "failed"
                });
            sub_queries.push(sub_fetch);
        }
        let _x = self.core.run(join_all(sub_queries));
    }

    fn collect_all_responses<T: DeserializeOwned>(
        &mut self,
        result: &mut QueryResult<T>,
        receiver: &Receiver<Result<Box<QueryResult<T>>, usize>>,
    ) {
        let total = self.pending_jobs.len();
        for x in 1..=total {
            if let Ok(process_result) = receiver.recv() {
                match process_result {
                    Ok(qry) => {
                        self.finished.push(qry.startAt);
                        result.collect_from(*qry);
                        info!("[{}/{}] Collected a paged response!", x, total);
                    }
                    Err(job) => {
                        if job > 10000 {
                            //Should also know which was not so successful since upon failure
                            // unsuccessful futures might have been cancelled
                            //job -= 10000;
                            break;
                        } else {
                            info!("[{}/{}] unexpected response, would retry this", x, total);
                        }
                    }
                }
            } else {
                error!("recev error now!");
            }
        }
    }

    fn clean_finished_from_pending(&mut self) {
        let finished = &mut self.finished;
        self.pending_jobs.retain(|ref qry| {
            finished.sort_unstable();
            finished.binary_search(&qry.startAt).is_err()
        });
    }
}
