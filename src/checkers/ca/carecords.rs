extern crate serde;
extern crate serde_json;

use checkers::records::*;
use super::caitem::CAItem;
use std::io::{Read, Write};
use std::time::SystemTime;
const REFRESH_THRESHOLD: u64 = 7200;

#[derive(Debug)]
pub(crate) enum ParseError {
    Json(serde_json::Error),
    Outdated,
}

pub(crate) fn parse_from<R>(reader: R) -> Result<Records<CAItem>, ParseError> where R: Read {
    let records:serde_json::Result<Records<CAItem>> = serde_json::from_reader(reader);
    match records {
        Ok(rc) => {
            info!("Loaded records and last updated = <{}>", rc.timestamp);
            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            if current_time - REFRESH_THRESHOLD >= rc.timestamp {
                Err(ParseError::Outdated)
            } else {
                Ok(rc)
            }
        },
        Err(err) => {
            error!("Unable to fetch records from reader by <{}>", err);
            Err(ParseError::Json(err))
        }
    }
}

//write given items into output with current timestamp
pub (crate) fn write_to<W>(writer: W, items: Vec<CAItem>) -> 
        (Result<String, String>, Vec<CAItem>) where W: Write {
    let rec = Records::new(items);
    let result = serde_json::to_writer_pretty(writer, &rec)
        .map(|_| {
            info!("Items updated to local cache now, timestamp={}", rec.timestamp);
            "Ok".to_string()
        })
        .map_err(|err| err.to_string());
    (result, rec.records)
}

// tests
#[cfg(test)]
mod tests {
    use checkers::ca::caissue::CAIssue;
use super::*;
    use super::super::caitem::Activity;
    use std::time::SystemTime;

    #[test]
    fn should_parse_records_from_json() {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let json = String::from("{ \"timestamp\" : ") + &current_time.to_string();
        let json = json + r#",
            "records": [
                {
                    "summary": "SomeFeature-A-a: some desc",
                    "sub_id": "SomeFeature-A-a",
                    "description": "some desc",
                    "feature_id": "SomeFeature",
                    "team": "team",
                    "start_fb": 1809,
                    "end_fb": 1809,
                    "efforts": 100,
                    "activity": "SW"
                }
            ]
        }"#;
        let result = parse_from(json.as_bytes());
        assert!(result.is_ok(), "Parse failed by:{:?}", result.err().unwrap());
        let records = result.unwrap();
        //assert_eq!(records.timestamp, 123456);
        assert_eq!(records.records.len(), 1);
        
        let item = &records.records[0];
        assert_eq!(item.summary, "SomeFeature-A-a: some desc");
        assert_eq!(item.sub_id, "SomeFeature-A-a");
        assert_eq!(item.description, "some desc");
        assert_eq!(item.feature_id, "SomeFeature");
        assert_eq!(item.start_fb, 1809);
        assert_eq!(item.end_fb, 1809);
        assert_eq!(item.activity, Activity::SW);
    }

    #[test]
    fn should_discard_parsed_if_older_than_2_hours() {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let saved = current_time - REFRESH_THRESHOLD;
        let json = String::from("{ \"timestamp\" : ") + &saved.to_string();
        let json = json + r#",
            "records": [
                {
                    "summary": "SomeFeature-A-a: some desc",
                    "sub_id": "SomeFeature-A-a",
                    "description": "some desc",
                    "feature_id": "SomeFeature",
                    "team": "team",
                    "start_fb": 1809,
                    "end_fb": 1809,
                    "efforts": 100,
                    "activity": "SW"
                }
            ]
        }"#;

        let result = parse_from(json.as_bytes());
        assert!(result.is_err());
        let error = result.err().unwrap();
        match error {
            ParseError::Outdated => {},
            ParseError::Json(err) => {assert!(false, "invalid due to {}", err);},
        }
    }

    #[test]
    fn should_forward_json_parsing_error() {
        let json = r#"{
            "timestamp": "invalid type!",
            "records": []
        }"#;
        let result = parse_from(json.as_bytes());
        assert!(result.is_err());
        match result.err().unwrap() {
            ParseError::Json(_) => {},
            _ => {assert!(false, "should give json error while found others!");}
        }
    }

    #[test]
    fn should_save_items_without_err() {
        let json = r#"{"expand" : "", "id": "", "self": "", "key": "", "fields": {
                "summary":"Leading - something else",
                "customfield_37381":"Feature_ID",
                "customfield_38727":"Team yyy",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "timeoriginalestimate":360000,
                "customfield_38750":{ "value": "EFS"}
        }}"#;
        let issue = serde_json::from_str::<CAIssue>(&json);
        let item = CAItem::from(&issue.unwrap());
        let items = vec![item];

        let storage : Vec<u8> = Vec::new();
        let result = write_to(storage, items);
        assert!(result.0.is_ok());
    }
}
