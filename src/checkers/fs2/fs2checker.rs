
extern crate tokio_core;
extern crate serde;

use self::tokio_core::reactor::Core;
use fetch::fetch::{Fetcher};
use query::result::QueryResult;
use super::super::persist::{parse_from, write_to};

use checkers::search::Searcher;
use super::fs2issue::Fs2Issue;
use super::fs2item::Fs2Item;
use checkers::utils::get_leftmost;

use std::io::{BufWriter, Write};
use std::fmt::format;
use std::fs::File;

type Fs2Result = QueryResult<Fs2Issue>;
use super::fs2issue::{FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_EE, FS2EE_FIELDS_TITLE};
const SEARCH_URI : &'static str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
const FS2EE_SEARCH : &'static str = "project=FPB AND issuetype in (\"\
    Effort Estimation\", \"Entity Technical Analysis\") \
    AND \"Competence Area\" = \"MANO MZ\"";

pub fn perform(core: &mut Core, fetcher: &mut Fetcher) {
    let fields = vec![FS2EE_FIELDS_SUMMARY, FS2EE_FIELDS_TITLE, FS2EE_FIELDS_EE]
                    .iter().map(|x| x.to_string()).collect();
    let mut result = Fs2Result::default(100);

    use std::io::{Error, ErrorKind};
    let items = File::open("fs2-items.json")
        .and_then(|f| parse_from(f)
                        .map(|rcs| rcs.records)
                        .map_err(|_x| Error::new(ErrorKind::Other, "not interested")))
        .or_else(|_x| -> Result<Vec<Fs2Item>, Error> {
            Searcher::new(core, fetcher, SEARCH_URI, vec![])
                .perform(FS2EE_SEARCH, fields, &mut result);
            let items : Vec<Fs2Item> = result.issues.iter().map(|it| Fs2Item::from(it)).collect();
            Ok(write_to(File::create("fs2-items.json").unwrap(), items).1)
        }).unwrap();
    analyze_results(&items);
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