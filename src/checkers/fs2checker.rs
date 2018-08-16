
extern crate tokio_core;
extern crate serde;

use self::tokio_core::reactor::Core;
use fetch::fetch::{Fetcher};
use query::result::QueryResult;

use checkers::search::perform_gen;
use checkers::fs2issue::Fs2Issue;

type Fs2Result = QueryResult<Fs2Issue>;
use checkers::fs2issue::{FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_EE, FS2EE_FIELDS_TITLE};
const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const FS2EE_SEARCH : &'static str = "project=FPB AND issuetype in (\"\
    Effort Estimation\", \"Entity Technical Analysis\") \
    AND \"Competence Area\" = \"MANO MZ\"";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    let fields = vec![FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_TITLE, FS2EE_FIELDS_EE]
                    .iter().map(|x| x.to_string()).collect();
    let result = perform_gen::<Fs2Issue>(core, fetcher, SEARCH_URI, FS2EE_SEARCH, fields);
    check_and_dump(&result);
}

pub fn check_and_dump(result_list: &Fs2Result) {
    //dumping
    let total = result_list.issues.len();
    info!("Now we get {} issues in total!", total);

    //summarize
    let unsolved:Vec<&Fs2Issue> = result_list.issues.iter().filter(|it| !it.has_efforts()).collect();
    info!("Unsolved entries are: {}", unsolved.len());
    unsolved.iter().for_each(|it| it.log());

    let solved_eff:u32 = result_list.issues.iter()
        .filter(|it| it.has_efforts())
        .map(|it| it.get_efforts().unwrap())
        .fold(0, |acc, x| acc+x);
    info!("Solved efforts are: {} with {} features", solved_eff, total - unsolved.len());
}