extern crate serde;
extern crate serde_json;

use self::serde_json::Value;
use query::issue::Issue;

pub(crate) const CA_FIELDS_SUMMARY      : &'static str = "summary";
pub(crate) const CA_FIELDS_FEATUREID    : &'static str = "customfield_37381";
pub(crate) const CA_FIELDS_TEAM         : &'static str = "customfield_38727";
pub(crate) const CA_FIELDS_STARTFB      : &'static str = "customfield_38694";
pub(crate) const CA_FIELDS_ENDFB        : &'static str = "customfield_38693";
pub(crate) const CA_FIELDS_TYPE         : &'static str = "customfield_38750";
const NA_STRING : &'static str = "NA";
const DEFAULT_FB : u32 = 9999;

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct CAFields {
    pub summary: String,

    #[serde(rename="customfield_37381")]
    pub feature_id: Value,

    #[serde(rename="customfield_38727")]
    pub team : Value, //string or null

    #[serde(rename="customfield_38694")]
    pub start_fb: Value,

    #[serde(rename="customfield_38693")]
    pub end_fb: Value,

    #[serde(rename="customfield_38750")]
    pub activity_type: Value,
}

#[derive(Ord, Eq, PartialOrd, PartialEq, Debug, Clone)]
pub enum Activity {
    EFS, //This is an EFS CA item
    SW,  //This is a SW team item
    ET,  //This is an ET item
    NA,  //Unrecognized item!
}

use std::fmt::{Display, Formatter};
use std::fmt;
impl Display for Activity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let repr = match *self {
            Activity::EFS => "EFS",
            Activity::SW => "SW",
            Activity::ET => "ET",
            Activity::NA => "NA",
        };
        write!(f, "{}", repr)
    }
}

#[derive(Eq, Clone)]
pub struct CAItem {
    pub summary: String,
    pub feature_id: String,
    pub team: String,
    pub start_fb: u32,
    pub end_fb: u32,
    pub activity: Activity,
}

impl CAItem {
    pub fn from(issue: &CAIssue) -> CAItem {
        let special : &[_] = &['\t', '\n', '\r', ' '];
        CAItem {
            summary: issue.fields.summary.clone(),
            feature_id: issue.get_fid().to_string(),
            team: issue.get_team().trim_right_matches(special).to_string(),
            start_fb: convert_fb(issue.get_start()),
            end_fb: convert_fb(issue.get_end()),
            activity: issue.get_type(),
        }
    }

    pub fn get_summary(&self) -> (&str, &str) {
        //split by first " " only
        match self.summary.find(|x:char| x==' ' || x == '\t') {
            Some(x) => {
                let (first, last) = self.summary.split_at(x);
                let skips: &[_] = &[' ', '-', '\t'];
                (first, last.trim_left_matches(skips))
            },
            None => (&self.summary, " "),
        }
    }
}

use std::cmp::Ordering;
impl Ord for CAItem {
    fn cmp(&self, other: &CAItem) -> Ordering {
        self.feature_id.cmp(&other.feature_id)
            .then(self.start_fb.cmp(&other.start_fb))
            .then(self.end_fb.cmp(&other.end_fb))
            .then(self.activity.cmp(&other.activity))
    }
}

impl PartialOrd for CAItem {
    fn partial_cmp(&self, other:&CAItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CAItem {
    fn eq(&self, other: &CAItem) -> bool {
        if self.cmp(other) == Ordering::Equal {
            self.summary == other.summary && self.team == other.team
        } else {
            false
        }
    }
}

pub type CAIssue = Issue<CAFields>;

fn get_wrapped_string(value:&Value) -> &str {
    match value {
        Value::String(ref some) => some,
        _ => &NA_STRING,
    }
}

fn convert_fb(value: &str) -> u32 {
    if value == NA_STRING {
        DEFAULT_FB.clone()
    } else {
        match u32::from_str_radix(value, 10) {
            Ok(x) => x,
            Err(_) => DEFAULT_FB.clone(),
        }
    }
}

impl CAIssue {
    pub fn log(&self) {
        info!("|{}|{}|{}|{}|{}|{}|", self.fields.summary, self.fields.feature_id, 
            self.fields.activity_type, self.fields.team, self.fields.start_fb, 
            self.fields.start_fb);
    }

    //get summary info - actual id and description
    pub fn get_summary(&self) -> (&str, &str) {
        //split by first " " only
        match self.fields.summary.find(|x:char|x == ' ' || x == '\t') {
            Some(x) => self.fields.summary.split_at(x),
            None => (&self.fields.summary, " "),
        }
    }

    pub fn get_fid(&self) -> &str {
        get_wrapped_string(&self.fields.feature_id)
    }

    pub fn get_type(&self) -> Activity {
        match self.fields.activity_type {
            Value::Object(ref obj) => {
                match obj["value"] {
                    Value::String(ref x) => {
                        if ["Entity Specification", "EFS"].iter().any(|k| x.find(k).is_some()) {
                            Activity::EFS
                        } else if ["Entity Testing", "ET"].iter().any(|k| x.find(k).is_some()) {
                            Activity::ET
                        } else {
                            Activity::SW
                        }
                    }, 
                    _ => Activity::NA, 
                }
            },
            _ => Activity::NA,
        }
    }

    pub fn get_start(&self) -> &str {
        get_wrapped_string(&self.fields.start_fb)
    }

    pub fn get_end(&self) -> &str {
        get_wrapped_string(&self.fields.end_fb)
    }

    pub fn get_team(&self) -> &str {
        get_wrapped_string(&self.fields.team)
    }
}