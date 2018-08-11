extern crate env_logger;
extern crate jira_reporter;
extern crate hyper;

fn main() {
    let env = env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();
    jira_reporter::run();
}
