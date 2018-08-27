
extern crate tokio_core;
extern crate serde;

use self::tokio_core::reactor::Core;
use fetch::fetch::{Fetcher};
use query::result::QueryResult;

use checkers::search::Searcher;
use checkers::fs2::fs2issue::Fs2Issue;

use std::io::BufWriter;
use std::io::Write;
use std::fmt::format;
use std::fs::File;

type Fs2Result = QueryResult<Fs2Issue>;
use checkers::fs2::fs2issue::{FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_EE, FS2EE_FIELDS_TITLE};
const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const FS2EE_SEARCH : &'static str = "project=FPB AND issuetype in (\"\
    Effort Estimation\", \"Entity Technical Analysis\") \
    AND \"Competence Area\" = \"MANO MZ\"";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    let fields = vec![FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_TITLE, FS2EE_FIELDS_EE]
                    .iter().map(|x| x.to_string()).collect();
    let mut result = Fs2Result::default(100);
    Searcher::new(core, fetcher, SEARCH_URI, vec![])
        .perform(FS2EE_SEARCH, fields, &mut result);
    check_and_dump(&result);
}

pub fn check_and_dump(result_list: &Fs2Result) {
    //dumping
    let total = result_list.issues.len();
    let mut buf_writer = BufWriter::new(File::create("fs-analysis.txt").unwrap());
    let banner = "----------------------------------------------------------------------------\n";

    //summarize
    let unsolved:Vec<&Fs2Issue> = result_list.issues.iter().filter(|it| !it.has_efforts()).collect();
    let summary_line = format(format_args!("@@@ Total MZ FS2EE entries: {}, unresolved: {}\n",
            total, unsolved.len()));
    let _x = buf_writer.write(summary_line.as_bytes());
    info!("{}", summary_line);

    buf_writer.write(banner.as_bytes()).unwrap();
    unsolved.iter().for_each(|it| {
        let line = format(format_args!("{:10}|{}\n", 
                it.fields.summary.split_at(9).0,  
                it.get_title_display()));
        let _x = buf_writer.write(line.as_bytes());
    });
    buf_writer.write(banner.as_bytes()).unwrap();

    let solved_eff:u32 = result_list.issues.iter()
        .filter(|it| it.has_efforts())
        .map(|it| it.get_efforts().unwrap())
        .fold(0, |acc, x| acc+x);
    
    let line = format(format_args!("@@@ Solved efforts are: {} with {} features\n", solved_eff, total - unsolved.len()));
    let _x = buf_writer.write(line.as_bytes()).unwrap();
    info!("Analyzed done for this query!");
}