
extern crate tokio_core;
extern crate serde;

use self::tokio_core::reactor::Core;
use fetch::fetch::{Fetcher};
use query::result::QueryResult;

use std::io::BufWriter;
use std::io::Write;
use std::fmt::format;
use std::fs::File;

use checkers::search::perform_gen;
use checkers::caissue::CAIssue;

type CAResult = QueryResult<CAIssue>;
use checkers::caissue::{CA_FIELDS_FEATUREID, CA_FIELDS_SUMMARY, CA_FIELDS_TYPE, 
        CA_FIELDS_TEAM, CA_FIELDS_STARTFB, CA_FIELDS_ENDFB};

const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const CA_SEARCH : &'static str = "project=FPB AND issuetype = \"\
    Competence Area\" AND \"Competence Area\" = \"MANO MZ\"";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    let fields = vec![CA_FIELDS_FEATUREID, CA_FIELDS_SUMMARY, CA_FIELDS_TEAM, 
            CA_FIELDS_TYPE, CA_FIELDS_STARTFB, CA_FIELDS_ENDFB].iter()
        .map(|x| x.to_string()).collect();
    
    let mut result: CAResult = CAResult::default(100);
    perform_gen::<CAIssue>(core, fetcher, SEARCH_URI, CA_SEARCH, fields, &mut result);
    check_and_dump(&result);
}

pub fn check_and_dump(result_list: &CAResult) {
    //dumping
    let total = result_list.issues.len();
    let mut buf_writer = BufWriter::new(File::create("ca-analysis.txt").unwrap());
    let summary = format(format_args!("@@ CA analysis: {} issues in total\n", total));
    info!("{}", summary);
    buf_writer.write(summary.as_bytes()).unwrap();

    result_list.issues.iter().for_each(|it| {
        let line = format(format_args!("{}|{}|{}|{}|{}\n",
            it.get_summary(), it.get_fid(), it.get_type(), it.get_start(), it.get_end()    
        ));
        buf_writer.write(line.as_bytes()).unwrap();
    });

    info!("Analysis of CA issues finished!");
}