extern crate tokio_core;
extern crate serde;

use self::tokio_core::reactor::Core;
use fetch::fetch::{Fetcher};

use super::fs2item::Fs2Item;
use super::super::analyze::analyze;
use checkers::utils::get_leftmost;

use std::io::{BufWriter, Write};
use std::fmt::format;
use std::fs::File;

const FS2EE_SEARCH : &'static str = "project=FPB AND issuetype in (\"\
    Effort Estimation\", \"Entity Technical Analysis\") \
    AND \"Competence Area\" = \"MANO MZ\"";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    analyze(core, fetcher, FS2EE_SEARCH, "fs2-items.json", analyze_results);
}

pub fn analyze_results(result_list: &Vec<Fs2Item>) {
    //dumping
    let total = result_list.len();
    let mut buf_writer = BufWriter::new(File::create("fs-analysis.txt").unwrap());
    let banner = "----------------------------------------------------------------------------\n";

    //summarize
    let unsolved:Vec<&Fs2Item> = result_list.iter().filter(|it| !it.has_efforts()).collect();
    let summary_line = format(format_args!("@@@ Total MZ FS2EE entries: {}, unresolved: {}\n",
            total, unsolved.len()));
    let _x = buf_writer.write(summary_line.as_bytes());
    info!("{}", summary_line);

    buf_writer.write(banner.as_bytes()).unwrap();
    unsolved.iter().for_each(|it| {
        let line = format(format_args!("{:10}|{}\n", get_leftmost(&it.summary, 10), it.title));
        let _x = buf_writer.write(line.as_bytes());
    });
    buf_writer.write(banner.as_bytes()).unwrap();

    let solved_eff:i32 = result_list.iter()
        .filter(|it| it.has_efforts())
        .map(|it| it.efforts)
        .fold(0, |acc, x| acc+x);
    
    let line = format(format_args!("@@@ Solved efforts are: {} with {} features\n", solved_eff, total - unsolved.len()));
    let _x = buf_writer.write(line.as_bytes()).unwrap();
    info!("Analyzed done for this query!");
}