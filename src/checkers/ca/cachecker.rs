extern crate tokio_core;
extern crate serde;
extern crate itertools;

use self::tokio_core::reactor::Core;
use fetch::fetch::{Fetcher};
use query::result::QueryResult;

use std::io::BufWriter;
use std::io::Write;
use std::fmt::format;
use std::fs::File;

use self::itertools::Itertools;

use checkers::search::Searcher;
use checkers::ca::caissue::CAIssue;
use checkers::ca::caitem::{Activity, CAItem};
use checkers::ca::timeline::analyze_timeline;
use checkers::utils::get_leftmost;

type CAResult = QueryResult<CAIssue>;
use checkers::ca::caissue::{CA_FIELDS_FEATUREID, CA_FIELDS_SUMMARY, CA_FIELDS_TYPE, 
        CA_FIELDS_TEAM, CA_FIELDS_STARTFB, CA_FIELDS_ENDFB, CA_FIELDS_ORIG_EFF};

const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const CA_SEARCH : &'static str = "project=FPB AND issuetype = \"\
    Competence Area\" AND \"Competence Area\" = \"MANO MZ\"";
pub const BANNER: &'static str = "================================================================================================\n";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    let mut result: CAResult = CAResult::default(100);
    let fields = vec![CA_FIELDS_FEATUREID, CA_FIELDS_SUMMARY, CA_FIELDS_TEAM, CA_FIELDS_TYPE, 
            CA_FIELDS_STARTFB, CA_FIELDS_ENDFB, CA_FIELDS_ORIG_EFF].iter()
        .map(|x| x.to_string()).collect();
    
    Searcher::new(core, fetcher, SEARCH_URI, vec![]).perform(CA_SEARCH, fields, &mut result);
    analyze_result(&result);
}

pub fn analyze_result(result_list: &CAResult) {
    let items : Vec<CAItem> = result_list.issues.iter().map(|it| CAItem::from(it)).collect();
    let items = items.into_iter().sorted();

    //dumping
    let mut buf_writer = BufWriter::new(File::create("ca-details-report.txt").unwrap());
    dump_all(&mut buf_writer, &items);
    info!("All items' details dumped to report file!");

    //calcualte lead time by features
    let mut buf_writer = BufWriter::new(File::create("ca-lead-time-report.txt").unwrap());
    fn efs_ei(it: &CAItem) -> bool { it.activity != Activity::NA}
    fn efs_sw(it: &CAItem) -> bool { it.activity != Activity::NA && it.activity != Activity::ET }
    analyze_timeline(&mut buf_writer, &items, "EFS-EI", &mut efs_ei);
    analyze_timeline(&mut buf_writer, &items, "EFS-SW", &mut efs_sw);
    info!("All items' lead time analyzed and dump to report file!");
    
    info!("Analysis of CA issues finished!");
}

fn dump_all(buf_writer: &mut BufWriter<File>, items: &Vec<CAItem>){
    let total = items.len();
    let summary = format(format_args!("@@ CA analysis: {} issues in total\n", total));
    info!("{}", summary);
    buf_writer.write(summary.as_bytes()).unwrap();
    
    buf_writer.write(BANNER.as_bytes()).unwrap();
    items.iter().for_each(|it| {
        let line = format(format_args!("{:9}|{:20}|{:3}|{:12}|{:4}|{:4}|{:4}|{:40}\n",
            it.feature_id, get_leftmost(&it.sub_id, 20), it.activity, get_leftmost(&it.team, 12), 
            it.start_fb, it.end_fb, it.efforts, get_leftmost(&it.description, 40)    
        ));
        buf_writer.write(line.as_bytes()).unwrap();
    });
    buf_writer.write(BANNER.as_bytes()).unwrap();

    let total_efforts = items.iter().map(|it| if it.efforts > 0 {it.efforts} else {0}).sum::<i32>();
    let unestimated = items.iter().filter(|it| it.efforts > 0).count();
    buf_writer.write(format(format_args!("Total efforts:{}, unestimated: {}/[{:.1}%]", 
            total_efforts, unestimated, (unestimated as f32)/(total as f32)*100.0
        )).as_bytes()).unwrap();
}