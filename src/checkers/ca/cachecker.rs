extern crate itertools;

use checkers::ca::timeline::get_system_split;
use std::io::{BufWriter};
use std::io::Write;
use std::fmt::format;
use std::fs::File;

use super::caitem::{Activity, CAItem};
use super::timeline::analyze_timeline;
use super::pipeline::PipelineInfo;
use super::super::fs2::fs2item::Fs2Item;
use super::super::sys::sysitem::SysItem;
use checkers::utils::get_leftmost;

use self::itertools::Itertools;

pub const BANNER: &'static str = "================================================================================================\n";

pub fn analyze_result(items: &Vec<CAItem>, sys_items: &Vec<SysItem>, _fs2_items: &Vec<Fs2Item>) {
    //dumping
    let mut buf_writer = BufWriter::new(File::create("ca-details-report.txt").unwrap());
    dump_all(&mut buf_writer, &items, sys_items);
    info!("All items' details dumped to report file!");

    //calcualte lead time by features
    let mut buf_writer = BufWriter::new(File::create("ca-lead-time-report.txt").unwrap());
    fn efs_ei(it: &CAItem) -> bool { it.activity != Activity::NA}
    fn efs_sw(it: &CAItem) -> bool { it.activity != Activity::NA && it.activity != Activity::ET }
    analyze_timeline(&mut buf_writer, &items, "EFS-EI", &mut efs_ei);
    analyze_timeline(&mut buf_writer, &items, "EFS-SW", &mut efs_sw);
    info!("All items' lead time analyzed and dump to report file!");

    let mut buf_writer = BufWriter::new(File::create("ca-plan-report.txt").unwrap());    
    analyze_plan(&mut buf_writer, items, sys_items);
    info!("Plan status analyzed!");

    let mut buf_writer = BufWriter::new(File::create("ca-pipeline.txt").unwrap());    
    generate_pipeline(&mut buf_writer, items);

    info!("Analysis of CA issues finished!");
}

fn dump_all(buf_writer: &mut BufWriter<File>, items: &Vec<CAItem>, sys_items: &Vec<SysItem>){
    let total = items.len();
    let summary = format(format_args!("@@ CA analysis: {} issues in total\n", total));
    info!("Got {} issues for this analysis", total);
    buf_writer.write(summary.as_bytes()).unwrap();

    use std::collections::HashMap;
    let mut sys_map = HashMap::with_capacity(sys_items.len());
    for it in sys_items {
        let _x = sys_map.insert(it.get_fid().to_string(), it);
    }
    
    buf_writer.write(BANNER.as_bytes()).unwrap();
    items.iter().for_each(|it| {
        let release = sys_map.get(&it.feature_id).map(|sys_it| sys_it.release.as_ref()).unwrap_or("");
        let line = format(format_args!("{:9}|{:15}|{:4}|{:12}|{:10}|{:3}|{:8}|{:4}|{:4}|{:4}|{:60}\n",
            it.feature_id, get_leftmost(&it.sub_id, 15), it.target, get_leftmost(release, 12),
            it.key, it.activity, get_leftmost(&it.team, 8), it.start_fb, 
            it.end_fb, it.efforts, get_leftmost(&it.description, 60)
        ));
        buf_writer.write(line.as_bytes()).unwrap();
    });
    buf_writer.write(BANNER.as_bytes()).unwrap();

    let total_efforts = items.iter().map(|it| if it.efforts > 0 {it.efforts} else {0}).sum::<i32>();
    let unestimated = items.iter().filter(|it| it.efforts == -1).count();
    buf_writer.write(format(format_args!("Total efforts:{}, unestimated: {}/{}[{:.1}%]", 
            total_efforts, unestimated, total, (unestimated as f32)/(total as f32)*100.0
        )).as_bytes()).unwrap();
}

pub fn analyze_plan(buf_writer: &mut BufWriter<File>, items: &Vec<CAItem>, sys_items: &Vec<SysItem>) {
    //check if everything is planned by entity level!
    let mut om_features:Vec<&str> = sys_items.into_iter()
        .filter(|it| it.is_oam_feature())
        .map(|it| it.get_fid())
        .collect();
    om_features.sort();
    
    let line = format(format_args!("Total {} OM system level features candidate\n", om_features.len()));
    buf_writer.write(line.as_bytes()).unwrap();
    buf_writer.write(BANNER.as_bytes()).unwrap();

    //check planning status
    let mut planned = 0;
    let mut unplanned = 0;
    for (fid, mut sub_items) in &items
        .into_iter()
        .filter(|it| om_features.binary_search_by(|fid| cmp_with_prefix_as_equal(fid, it.sub_id.as_str())).is_ok() )
        .group_by(|item| get_system_split(&item.sub_id).clone()) {
            //check if ET planned
            let test_status = if sub_items.any(|it| it.activity == Activity::ET) {
                planned += 1;
                "planned"
            } else {
                unplanned += 1;
                "not planned!"
            };

        let line = format(format_args!("Fid = {}, ET status ={}\n", fid, test_status));
        buf_writer.write(line.as_bytes()).unwrap();
    }

    let line = format(format_args!("ET unplanned = {}, planned ={}\n", planned, unplanned));
    buf_writer.write(line.as_bytes()).unwrap();
    buf_writer.write(BANNER.as_bytes()).unwrap();
}

use std::cmp::Ordering;
fn cmp_with_prefix_as_equal(prefix: &str, right: &str) -> Ordering {
    if right.contains(prefix) {
        Ordering::Equal
    } else {
        prefix.cmp(right)
    }
}

pub fn generate_pipeline(buf_writer: &mut BufWriter<File>, items: &Vec<CAItem>) {
    items.into_iter()
        .map(|item| PipelineInfo::from_item(item))
        .for_each(|it| {
            //TODO: calculate first and max span?
            let line = it.generate_schedule_row(1801, 20) + "\n";
            buf_writer.write(line.as_bytes()).unwrap();
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    #[test]
    fn should_compare_with_same_prefix_as_equal() {
        assert_eq!(cmp_with_prefix_as_equal("AAA", "AAA-b"), Ordering::Equal);   
        assert_eq!(cmp_with_prefix_as_equal("AAA", "AAA-B-c"), Ordering::Equal);   
        assert_eq!(cmp_with_prefix_as_equal("AAA", "AAA"), Ordering::Equal);   
    }

    #[test]
    fn should_compare_with_different_prefix_by_strcmp() {
        assert_eq!(cmp_with_prefix_as_equal("AAA", "BBB-b"), Ordering::Less);   
        assert_eq!(cmp_with_prefix_as_equal("BBB", "AAA-b"), Ordering::Greater);   
    }
}