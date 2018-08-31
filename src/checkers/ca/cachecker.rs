extern crate itertools;

use std::io::{BufWriter};
use std::io::Write;
use std::fmt::format;
use std::fs::File;

use super::caitem::{Activity, CAItem};
use super::timeline::analyze_timeline;
use checkers::utils::get_leftmost;

pub const BANNER: &'static str = "================================================================================================\n";

pub fn analyze_result(items: &Vec<CAItem>) {
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