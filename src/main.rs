extern crate flexi_logger;
extern crate jira_reporter;
extern crate hyper;
extern crate tokio_core;
extern crate futures;

use tokio_core::reactor::Core;
use jira_reporter::fetch::fetch::Fetcher;
use jira_reporter::fetch::login::Login;
use std::rc::Rc;

fn main() {
    init_logs();
    run_reports();
}

fn init_logs() {
    flexi_logger::Logger::with_env_or_str("info")
        .format(flexi_logger::opt_format)
        .log_to_file()
        .print_message()
        .directory("logs")
        .start()
        .unwrap_or_else(|_e| panic!("Start log failed!"));
}

fn run_reports() {
    use jira_reporter::checkers::{ca::cachecker, fs2::fs2checker, analyze::analyze};

    let mut core = Core::new().unwrap();
    let login = Rc::new(Login::new().to_basic());
    let mut fetcher = Fetcher::new(login);

    let handle = core.handle();
    let analysis = futures::future::lazy(move || {
        const FS2EE_SEARCH : &'static str = "project=FPB AND issuetype in (\"\
            Effort Estimation\", \"Entity Technical Analysis\") \
            AND \"Competence Area\" = \"MANO MZ\"";
        analyze(&handle, &mut fetcher, FS2EE_SEARCH, "fs2-items.json", 
                fs2checker::analyze_results);

        const CA_SEARCH : &'static str = "project=FPB AND issuetype = \"\
            Competence Area\" AND \"Competence Area\" = \"MANO MZ\"";
        analyze(&handle, &mut fetcher, CA_SEARCH, "ca-items.json", 
                cachecker::analyze_result);
        futures::future::ok::<u32, u32>(1)
    });
    core.run(analysis).unwrap();
}
