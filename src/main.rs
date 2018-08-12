extern crate flexi_logger;
extern crate jira_reporter;
extern crate hyper;

fn main() {
    flexi_logger::Logger::with_env_or_str("info")
        .format(flexi_logger::opt_format)
        .log_to_file()
        .print_message()
        .directory("logs")
        .start()
        .unwrap_or_else(|_e| panic!("Start log failed!"));
    jira_reporter::run();
}
