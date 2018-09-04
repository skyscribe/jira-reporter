extern crate itertools;

use std::io::{BufWriter, Write};
use std::fmt::format;
use std::fs::File;

use super::caitem::CAItem;
use self::itertools::{Itertools, MinMaxResult};

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

pub fn analyze_timeline<F>(buf_writer:&mut BufWriter<File>, items: &Vec<CAItem>, 
        hint: &str, issue_filter:&mut F)
            where F: FnMut(&CAItem) -> bool {
    let line = format(format_args!("@@ Planned features {} lead time analysis\n", hint));
    buf_writer.write(line.as_bytes()).unwrap();

    let mut planned = 0;
    let mut timelines = Vec::new();
    for (fid, sub_items) in &items.into_iter()
            .filter(|it| it.start_fb < 3000 && it.end_fb < 3000)
            .filter(|it| issue_filter(it))
            .group_by(|item| get_system_split(&item.sub_id).clone()) {
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

pub (crate) fn get_system_split<'a>(sub_id: &'a str) -> &'a str {
    let last = sub_id.rfind("-").unwrap_or(sub_id.len());
    let prev = sub_id[0..last].rfind("-").unwrap_or(last);
    if prev == last {
        sub_id
    } else {
        &sub_id[0..last]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_system_level_split() {
        assert_eq!(get_system_split("Feature-A-b"), "Feature-A");
        assert_eq!(get_system_split("Feature-A-b1"), "Feature-A");
        assert_eq!(get_system_split("Feature-A"), "Feature-A");
        assert_eq!(get_system_split("Feature"), "Feature");
        assert_eq!(get_system_split("Feature-A-b1/b2/b3"), "Feature-A");
    }
}