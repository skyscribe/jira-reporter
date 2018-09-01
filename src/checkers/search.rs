
extern crate hyper;
extern crate tokio_core;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::clone::Clone;
use std::sync::mpsc::channel;
use self::tokio_core::reactor::Handle;
extern crate futures;

extern crate serde;
use self::serde::de::DeserializeOwned;
use self::hyper::StatusCode;

use fetch::fetch::{Fetcher, RequestInfo};
use query::query::Query;
use query::result::{parse_query_result, QueryResult};
use self::futures::future::{Future, join_all};

pub struct Searcher<'a> {
    handle: Handle,
    fetcher: &'a mut Fetcher,
    uri: &'a str,
    pending_jobs: Vec<Query>,
    finished: Vec<usize>,
    response_collected: usize,
}

impl <'a> Searcher<'a> {
    pub fn new(handle: Handle, fetcher: &'a mut Fetcher, uri: &'a str, 
            pending: Vec<Query>) -> Searcher<'a> {
        Searcher{
            handle: handle,
            fetcher: fetcher,
            uri: uri,
            pending_jobs: pending,
            finished: Vec::new(),
            response_collected: 0,
        }
    }

    //Search by given jql and issue fields, and collect all results in one single 
    // result, 2-phases based search is used to calculating paging properly.
    pub fn perform<T: DeserializeOwned+'static>(&mut self, jql: &str, fields: Vec<String>, 
            result: &mut QueryResult<T>) {
        let search = Query::new(jql.to_string(), 100, fields);
        //first search
        self.reset_pending(vec![search.clone()])
            .perform_parallel(result);
        if result.issues.len() == 0 {
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

    fn perform_parallel<T: DeserializeOwned+'static>(&mut self, result: &mut QueryResult<T>){
        let (tx, rx) = channel();
        while self.pending_jobs.len() > 0 {
            self.finished.clear();
            self.drain_all_jobs(&tx);
            self.response_collected = 0;
            self.collect_all_responses(result, &rx);
            self.clean_finished_from_pending();
        }
    }

    fn drain_all_jobs<T:DeserializeOwned+'static>(&mut self, sender: 
            &Sender<Result<Box<QueryResult<T>>, usize>>) {
        let mut sub_queries = Vec::new();
        //Drain all pending jobs 
        for qry in self.pending_jobs.iter() {
            //query this page
            let sender1 = sender.clone();
            let start = qry.startAt.clone();
            let parser = move |json: &str, code: StatusCode| {
                let my_result = match code{
                    StatusCode::Ok => parse_query_result::<T>(&json).ok_or_else(|| start),
                    _ => Err(start),
                };
                let _x = sender1.send(my_result);
            };

            let start = qry.startAt.clone();
            let post_info = RequestInfo::post(self.uri, &qry.to_json().unwrap());
            let guard1 = sender.clone();
            let sub_fetch = self.fetcher.query_with(post_info, self.handle.clone(), Some(parser))
                    .map_err(move |err| { 
                        //TODO: handle exceptions in graceful manner?
                        warn!("This job {} has failed by {}", start, err);
                        let _x = guard1.send(Err(start + 10000)); 
                    });
            sub_queries.push(sub_fetch);
        }
        let fut = join_all(sub_queries).map(|_x| {});
        let _x = self.handle.spawn(fut);
    }

    fn collect_all_responses<T:DeserializeOwned>(&mut self, result: &mut QueryResult<T>, 
        receiver: &Receiver<Result<Box<QueryResult<T>>, usize>>) {
        let total = self.pending_jobs.len();
        use std::time;
        while self.response_collected != total {
            if let Ok(process_result) = receiver.recv_timeout(time::Duration::from_millis(200)) {
                self.response_collected = self.response_collected + 1;
                match process_result {
                    Ok(qry) => {
                        self.finished.push(qry.startAt);
                        result.collect_from(*qry);
                        info!("[{}/{}] Collected a paged response!", self.response_collected, total);
                    },
                    Err(job) => {
                        if job > 10000 {
                            //Should also know which was not so successful since upon failure
                            // unsuccessful futures might have been cancelled
                            //job -= 10000;
                            break;
                        } else {
                            info!("[{}/{}] unexpected response, would retry this", job, total);
                        }
                    },
                }
            } else {
                //Nothing received? Retry
            }
        }
    }

    fn clean_finished_from_pending(&mut self) {
        let ref mut finished = self.finished;
        self.pending_jobs.retain(|ref qry| {
            finished.sort_unstable();
            finished.binary_search(&qry.startAt).is_err()
        });
    }
}