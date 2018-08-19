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

use self::itertools::{Itertools, MinMaxResult};

use checkers::search::perform_gen;
use checkers::caissue::{CAIssue, CAItem};

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
    analyze_result(&result);
}

pub fn analyze_result(result_list: &CAResult) {
    //dumping
    let total = result_list.issues.len();
    let mut buf_writer = BufWriter::new(File::create("ca-analysis.txt").unwrap());
    let banner = "================================================================================================\n"
        .as_bytes();

    let summary = format(format_args!("@@ CA analysis: {} issues in total\n", total));
    info!("{}", summary);
    buf_writer.write(summary.as_bytes()).unwrap();

    //dumy all items
    buf_writer.write(banner).unwrap();
    let items : Vec<CAItem> = result_list.issues.iter().map(|it| CAItem::from(it)).collect();
    //we need to sort by feature ids to group them later
    let items = items.into_iter().sorted_by(|it1, it2| Ord::cmp(&it1.feature_id, &it2.feature_id));
    items.iter().for_each(|it| {
        let (subid, desc) = it.get_summary();
        let line = format(format_args!("{:9}|{:20}|{:3}|{:12}|{:4}|{:4}|{:40}\n",
            it.feature_id, get_leftmost(subid, 20), it.activity, get_leftmost(&it.team, 12), 
            it.start_fb, it.end_fb, get_leftmost(desc, 40)    
        ));
        buf_writer.write(line.as_bytes()).unwrap();
    });
    buf_writer.write(banner).unwrap();

    //calcualte lead time by features
    buf_writer.write("@@ All planned features analysis:\n".as_bytes()).unwrap();
    let mut planned = 0;
    for (fid, sub_items) in &items.into_iter()
            .filter(|it| it.start_fb < 3000 && it.end_fb < 3000)
            .group_by(|item| item.feature_id.clone()) {
        let times:Vec<(u32, u32)> = sub_items.map(|it| (it.start_fb, it.end_fb)).collect();
        let (start_first, start_last) = match times.iter().map(|it| it.0).minmax() {
            MinMaxResult::MinMax(first, last) => (first.clone(), last.clone()),
            MinMaxResult::OneElement(x) => (x.clone(), x.clone()),
            _ => panic!("unexpected!"),
        };
        let (end_first, end_last) = match times.iter().map(|it| it.1).minmax() {
            MinMaxResult::MinMax(first, last) => (first.clone(), last.clone()),
            MinMaxResult::OneElement(x) => (x.clone(), x.clone()),
            _ => panic!("unexpected!"),
        };

        //actual time might be span one year only? 1901-1812+1 = 90 => 2
        let lead_time = end_last - start_first + 1;
        let lead_time = if lead_time > 12 { lead_time - 88} else {lead_time};
        let line = format(format_args!("feature:{:10}, lead_time:{}, start: {} - {}, end: {} - {}, entries:{}\n",
            fid, lead_time, start_first, start_last, end_first, end_last, times.len()));
        buf_writer.write(line.as_bytes()).unwrap();
        planned += 1;
    }
    let line = format(format_args!("@@Totally planned features:{}\n", planned));
    buf_writer.write(line.as_bytes()).unwrap();
    
    info!("Analysis of CA issues finished!");
}

pub fn get_leftmost(raw: &str, total: usize) -> &str {
    let max = raw.find("\n").map_or(raw.len(), |x| x);
    if max > total {
        &raw[0..total]
    } else {
        &raw[0..max]
    }
}