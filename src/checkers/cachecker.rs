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

use checkers::search::Searcher;
use checkers::caissue::{CAIssue, CAItem, Activity};
use checkers::utils::get_leftmost;

type CAResult = QueryResult<CAIssue>;
use checkers::caissue::{CA_FIELDS_FEATUREID, CA_FIELDS_SUMMARY, CA_FIELDS_TYPE, 
        CA_FIELDS_TEAM, CA_FIELDS_STARTFB, CA_FIELDS_ENDFB};

const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const CA_SEARCH : &'static str = "project=FPB AND issuetype = \"\
    Competence Area\" AND \"Competence Area\" = \"MANO MZ\"";
pub const BANNER: &'static str = "================================================================================================\n";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    let mut result: CAResult = CAResult::default(100);
    let fields = vec![CA_FIELDS_FEATUREID, CA_FIELDS_SUMMARY, CA_FIELDS_TEAM, 
            CA_FIELDS_TYPE, CA_FIELDS_STARTFB, CA_FIELDS_ENDFB].iter()
        .map(|x| x.to_string()).collect();
    
    Searcher::new(core, fetcher, SEARCH_URI, vec![]).perform(CA_SEARCH, fields, &mut result);
    analyze_result(&result);
}

pub fn analyze_result(result_list: &CAResult) {
    //dumping
    let mut buf_writer = BufWriter::new(File::create("ca-analysis.txt").unwrap());

    let items : Vec<CAItem> = result_list.issues.iter().map(|it| CAItem::from(it)).collect();
    let items = items.into_iter().sorted();
    dump_all(&mut buf_writer, &items);

    //calcualte lead time by features
    fn efs_ei(it: &CAItem) -> bool { it.activity != Activity::NA}
    fn efs_sw(it: &CAItem) -> bool { it.activity != Activity::NA && it.activity != Activity::ET }
    analyze_timeline(&mut buf_writer, &items, "EFS-EI", &mut efs_ei);
    analyze_timeline(&mut buf_writer, &items, "EFS-SW", &mut efs_sw);
    
    info!("Analysis of CA issues finished!");
}

fn dump_all(buf_writer: &mut BufWriter<File>, items: &Vec<CAItem>){
    let total = items.len();
    let summary = format(format_args!("@@ CA analysis: {} issues in total\n", total));
    info!("{}", summary);
    buf_writer.write(summary.as_bytes()).unwrap();
    
    buf_writer.write(BANNER.as_bytes()).unwrap();
    items.iter().for_each(|it| {
        let line = format(format_args!("{:9}|{:20}|{:3}|{:12}|{:4}|{:4}|{:40}\n",
            it.feature_id, get_leftmost(&it.sub_id, 20), it.activity, get_leftmost(&it.team, 12), 
            it.start_fb, it.end_fb, get_leftmost(&it.description, 40)    
        ));
        buf_writer.write(line.as_bytes()).unwrap();
    });
    buf_writer.write(BANNER.as_bytes()).unwrap();
}

struct TimeLineInfo{
    start_first: u32,
    start_last: u32,
    end_first: u32,
    end_last: u32,
    lead_time: u32,
}

impl TimeLineInfo {
    fn new(sf:u32, sl:u32, ef:u32, el:u32) -> TimeLineInfo {
        TimeLineInfo{
            start_first:sf, 
            start_last:sl, 
            end_first:ef, 
            end_last:el,
            lead_time: TimeLineInfo::get_lead_time(sf, el),
        }
    }

    fn get_lead_time(start_first:u32, end_last:u32) -> u32{
        let mut lead_time = end_last - start_first + 1;
        if lead_time > 12 { 
            lead_time -= 87 //1901 - 1813 = 1
        } 
        lead_time
    }
}

fn analyze_timeline<F>(buf_writer:&mut BufWriter<File>, items: &Vec<CAItem>, 
        hint: &str, issue_filter:&mut F)
            where F: FnMut(&CAItem) -> bool {
    let line = format(format_args!("@@ Planned features {} lead time analysis\n", hint));
    buf_writer.write(line.as_bytes()).unwrap();

    let mut planned = 0;
    let mut timelines = Vec::new();
    for (fid, sub_items) in &items.into_iter()
            .filter(|it| it.start_fb < 3000 && it.end_fb < 3000)
            .filter(|it| issue_filter(it))
            .group_by(|item| item.feature_id.clone()) {
        let times:Vec<(u32, u32)> = sub_items.map(|it| (it.start_fb, it.end_fb)).collect();
        let timeline = calculate_timeline(&times);

        let line = format(format_args!("@@@@@@ feature:{:10}, lead_time_{}:{}, start: {} - {}, end: {} - {}, entries:{}\n",
                fid, hint, timeline.lead_time, timeline.start_first, timeline.start_last, 
                timeline.end_first, timeline.end_last, times.len()));
        buf_writer.write(line.as_bytes()).unwrap();
        timelines.push((fid, timeline));

        planned += 1;
    }

    let line = format(format_args!("@@ Totally planned features:{} analyzed\n", planned));
    buf_writer.write(line.as_bytes()).unwrap();

    //TOP 20% dump
    let top_count = (planned as f32 * 0.2) as usize;
    let line = format(format_args!("@@ Top:{}[20%] of them as below\n", top_count));
    buf_writer.write(line.as_bytes()).unwrap();
    timelines.into_iter()
        .sorted_by(|tl_1, tl_2| tl_2.1.lead_time.cmp(&tl_1.1.lead_time))
        .iter()
        .take(top_count)
        .for_each(|tl| {
            let line = format(format_args!("### feature:{:10}, lead_time_{}:{}, start: {} - {}, end: {} - {}\n",
                tl.0, hint, tl.1.lead_time, tl.1.start_first, tl.1.start_last, 
                tl.1.end_first, tl.1.end_last));
            buf_writer.write(line.as_bytes()).unwrap();
        });
}

fn calculate_timeline(times: &Vec<(u32, u32)>) -> TimeLineInfo {
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
    TimeLineInfo::new(start_first, start_last, end_first, end_last)
}