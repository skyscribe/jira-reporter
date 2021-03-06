use super::sysitem::SysItem;
use crate::checkers::utils::get_leftmost;

use std::fmt::format;
use std::fs::File;
use std::io::{BufWriter, Write};
const BANNER: &str =
    "----------------------------------------------------------------------------\n";

pub fn analyze_results(items: &[SysItem]) {
    //dumping
    let mut buf_writer = BufWriter::new(File::create("sys-feature-details.txt").unwrap());
    dump_all(&mut buf_writer, &items);

    info!("Analyzed done for this query!\n");
}

fn dump_all(buf_writer: &mut BufWriter<File>, items: &[SysItem]) {
    let total = items.len();
    let summary = format(format_args!(
        "@@ Feature analysis: {} issues in total\n",
        total
    ));
    info!("Got {} system level features", total);
    buf_writer.write_all(summary.as_bytes()).unwrap();
    buf_writer.write_all(BANNER.as_bytes()).unwrap();

    items.iter().for_each(|it| {
        let line = format(format_args!(
            "{:20}|{:12}|{:40}|{:10}|{:8}|{:10}\n",
            get_leftmost(&it.summary, 20),
            get_leftmost(&it.area, 12),
            get_leftmost(&it.title, 40),
            get_leftmost(&it.key, 10),
            get_leftmost(&it.status, 8),
            get_leftmost(&it.release, 10)
        ));
        buf_writer.write_all(line.as_bytes()).unwrap();
    });
}
