use super::fs2item::Fs2Item;
use super::super::sys::sysitem::SysItem;
use checkers::utils::get_leftmost;

use std::io::{BufWriter, Write};
use std::fmt::format;
use std::fs::File;

pub fn analyze_results(result_list: &Vec<Fs2Item>, _sys_items:&Vec<SysItem>) {
    //dumping
    let total = result_list.len();
    let mut buf_writer = BufWriter::new(File::create("fs-analysis.txt").unwrap());
    let banner = "----------------------------------------------------------------------------\n";

    //summarize
    let unsolved:Vec<&Fs2Item> = result_list.iter()
        .filter(|it| !it.has_efforts() || it.status != "Done")
        .collect();
    let summary_line = format(format_args!("@@@ Total MZ FS2EE entries: {}, unresolved: {}\n",
            total, unsolved.len()));
    let _x = buf_writer.write(summary_line.as_bytes());
    info!("Got {} items for this analysis", total);

    buf_writer.write(banner.as_bytes()).unwrap();
    unsolved.iter().for_each(|it| {
        let line = format(format_args!("{:9}|{:31}|{:12}|{:6}|{:40}\n", 
            get_leftmost(&it.summary, 9), get_leftmost(&it.title, 31), 
            get_leftmost(&it.release, 12), it.efforts, get_leftmost(&it.description, 40)));
        let _x = buf_writer.write(line.as_bytes());
    });
    buf_writer.write(banner.as_bytes()).unwrap();

    let solved_eff:i32 = result_list.iter()
        .filter(|it| it.has_efforts())
        .map(|it| it.efforts)
        .fold(0, |acc, x| acc+x);
    
    let line = format(format_args!("@@@ Solved efforts are: {} with {} features\n", 
        solved_eff, total - unsolved.len()));
    let _x = buf_writer.write(line.as_bytes()).unwrap();
    info!("Analyzed done for this analysis!\n");
}