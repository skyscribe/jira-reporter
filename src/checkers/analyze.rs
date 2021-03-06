use super::persist::{parse_from, write_to};
use crate::fetch::fetcher::Fetcher;
use crate::query::result::QueryResult;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio_core::reactor::Core;

use std::cmp::Ord;
use std::fs::File;
use std::io::BufReader;

use super::datatypes::{ParsedData, StoredData};
use super::search::Searcher;
use itertools::Itertools;

const SEARCH_URI: &str = "https://jiradc.int.net.nokia.com/rest/api/2/search";
//skeleton function for fetch data and do analysis
pub fn analyze<T, R, F>(
    core: &mut Core,
    fetcher: &mut Fetcher,
    search: &'static str,
    cache_fname: &str,
    analyzer: F,
) -> Vec<T>
where
    T: DeserializeOwned + Serialize + StoredData<Parsed = R> + Ord,
    R: DeserializeOwned + ParsedData,
    F: Fn(&[T]) -> (),
{
    let mut result = QueryResult::<R>::default(100);
    let fields = R::get_field_list();

    use std::io::{Error, ErrorKind};
    let items = File::open(cache_fname)
        .and_then(|f| {
            parse_from(BufReader::new(f))
                .map(|rcs| rcs.records)
                .map_err(|_x| Error::new(ErrorKind::Other, "not interested"))
        })
        .or_else(|_x| -> Result<Vec<T>, Error> {
            Searcher::new(core, fetcher, SEARCH_URI, vec![]).perform(search, fields, &mut result);
            let items: Vec<T> = result.issues.iter().map(|it| T::parse_from(it)).collect();
            let items = items.into_iter().sorted();
            Ok(write_to(File::create(cache_fname).unwrap(), items).1)
        })
        .unwrap();
    analyzer(&items);
    items
}
