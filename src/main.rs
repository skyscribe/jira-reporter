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
    use jira_reporter::checkers::*;

    let mut core = Core::new().unwrap();
    let login = Rc::new(Login::new().to_basic());
    let mut fetcher = Fetcher::new(login);
    fschecker::perform(&mut core, &mut fetcher);
}
