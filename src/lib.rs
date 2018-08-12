#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

pub mod fetch;
mod demo;
pub mod query;

extern crate hyper;
extern crate tokio_core;
use self::tokio_core::reactor::Core;

use fetch::fetch::{Fetcher, FetchMethod};
use fetch::login::Login;
use query::query::Query;

pub fn run() {
    let mut core = Core::new().unwrap();
    let login = Login::new().to_basic();

    //test with filter
    let input = "https://jiradc.int.net.nokia.com/rest/api/2/filter/145359";
    let fut = Fetcher::new(input.to_string(), &login).fetch(&mut core);
    //schedule and run
    if let Err(_err) = core.run(fut) {
        error!("Something wrong here!");
    } else {
        info!("Completed now!");
    }

    //fetch a search result
    let uri = "https://jiradc.int.net.nokia.com/rest/api/2/search";
    let search_string = r#"project=FPB AND issuetype in ("Effort Estimation", "Entity Technical Analysis") AND "Competence Area" = "MANO MZ""#;
    if let Ok(json) = Query::new(search_string.to_string(), 100, vec![
        String::from("Summary"),
        String::from("FP Title"),
        String::from("Status"),
        String::from("FS2 EE (h)"),
    ]).to_json() {
        //construct query now
        let mut fetcher = Fetcher::new(uri.to_string(), &login);
        fetcher.set_method(FetchMethod::Post);
        fetcher.set_body(json);
        let future = fetcher.fetch(&mut core);
        //schedule and run
        if let Err(_) = core.run(future) {
            error!("Query failed to execute");
        }
    }
}