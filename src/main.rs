use jira_reporter::fetch::fetcher::Fetcher;
use jira_reporter::fetch::login::Login;
use std::rc::Rc;
use tokio_core::reactor::Core;

fn main() {
    init_logs();
    run_reports();
}

fn init_logs() {
    use flexi_logger::opt_format;
    flexi_logger::Logger::with_env_or_str("info")
        .format(opt_format)
        .log_to_file()
        .print_message()
        //.duplicate_to_stderr(Duplicate::Info)
        .directory("logs")
        .start()
        .unwrap_or_else(|_e| panic!("Start log failed!"));
}

fn run_reports() {
    use jira_reporter::checkers::{
        analyze::analyze, ca::cachecker, fs2::fs2checker, sys::syschecker,
    };

    let mut core = Core::new().unwrap();
    let login = Rc::new(Login::new().to_basic());
    let mut fetcher = Fetcher::new(login);

    let sys_search = r#"issuetype = "Customer Feature" AND (cf[38700] in (gNB, "Cloud BTS", "AirScale Cloud BTS") OR System in (5G, CloudRAN))"#;
    //let sys_search = "issuetype = \"Customer Feature\" AND System in (5G, CloudRAN)";
    let sys_items = analyze(
        &mut core,
        &mut fetcher,
        sys_search,
        "sys-items.json",
        syschecker::analyze_results,
    );

    let fs2_search = "project=FPB AND issuetype in (\"\
                      Effort Estimation\", \"Entity Technical Analysis\") \
                      AND \"Competence Area\" = \"MANO MZ\"";
    let fs2_items = analyze(&mut core, &mut fetcher, fs2_search, "fs2-items.json", |x| {
        fs2checker::analyze_results(x, &sys_items)
    });

    let ca_search = "project=FPB AND issuetype = \"\
                     Competence Area\" AND \"Competence Area\" = \"MANO MZ\"";
    analyze(&mut core, &mut fetcher, ca_search, "ca-items.json", |x| {
        cachecker::analyze_result(x, &sys_items, &fs2_items)
    });
}
