const DEFAULT_FB : u32 = 9999;
use checkers::caissue::CAIssue;
use checkers::caissue::NA_STRING;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Ord, Eq, PartialOrd, PartialEq, Debug, Clone)]
pub enum Activity {
    EFS, //This is an EFS CA item
    SW,  //This is a SW team item
    ET,  //This is an ET item
    NA,  //Unrecognized item!
}

impl Display for Activity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let repr = match *self {
            Activity::EFS => "EFS",
            Activity::SW => "SW",
            Activity::ET => "ET",
            Activity::NA => "NA",
        };
        if let Some(wid) = f.width() {
            write!(f, "{:width$}", repr, width=wid)
        } else {
            write!(f, "{}", repr)
        }
    }
}

#[derive(Eq, Clone)]
pub struct CAItem {
    pub summary: String,
    pub sub_id : String, 
    pub description: String, 
    pub feature_id: String,
    pub team: String,
    pub start_fb: u32,
    pub end_fb: u32,
    pub activity: Activity,
}

impl CAItem {
    pub fn from(issue: &CAIssue) -> CAItem {
        let special : &[_] = &['\t', '\n', '\r', ' '];
        let (subid, desc) = CAItem::get_summary(&issue.fields.summary);
        let activity = CAItem::get_type(issue.get_type());
        CAItem {
            summary: issue.fields.summary.clone(),
            feature_id: issue.get_fid().to_string(),
            team: issue.get_team().trim_right_matches(special).to_string(),
            start_fb: convert_fb(issue.get_start()),
            end_fb: convert_fb(issue.get_end()),
            activity: activity,
            sub_id: subid.to_string(),
            description: desc.to_string(),
        }
    }

    pub fn get_summary(summary: &str) -> (&str, &str) {
        //split by first " " only after trimming left spaces
        let skips: &[_] = &[' ', '\t', '\n'];
        let summary = summary.trim_left_matches(skips);
        match summary.find(|x:char| x==' ' || x == '\t') {
            Some(x) => {
                let (first, last) = summary.split_at(x);
                let skips: &[_] = &[' ', '-', '\t'];
                let last = last.trim_left_matches(skips);

                let skips: &[_] = &['-', ':'];
                let first = get_substr_until_one_of(&first, &[
                    "-CP3", "-EFS", "-OM", "-OAM", "-CFAM", "-EI", "-Ei"
                ]).trim_right_matches(skips);
                (first, last)
            },
            None => (&summary, " "),
        }
    }

    #[allow(dead_code)]
    pub fn reparse(&mut self) {
        let (subid, desc) = CAItem::get_summary(&self.summary);
        self.sub_id = subid.to_string();
        self.description = desc.to_string();
    }

    pub fn get_type(value: &str) -> Activity {
        let efs_kwds = vec!["Entity Specification", "EFS"];
        let et_kwds = vec!["Entity Testing", "ET"];
        let match_kwds = |kwds:Vec<&str>, x:&str| kwds.iter().any(|k| x.find(k).is_some());

        if value == NA_STRING {
            Activity::NA
        } else {
            if match_kwds(efs_kwds, value) {
                Activity::EFS
            } else if match_kwds(et_kwds, value) { 
                Activity::ET
            } else {
                Activity::SW
            }
        }
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

fn get_substr_until_one_of<'a>(current: &'a str, ignored: &[&str]) -> &'a str {
    let mut substr = current;
    for ignore in ignored {
        substr = get_substr_until(&substr, ignore);
    }
    substr
}

fn get_substr_until<'a>(current: &'a str, trailing: &str) -> &'a str {
    match current.rfind(trailing) {
        Some(ends) => &current[0..ends],
        _ => &current,
    }
}

use std::cmp::Ordering;
impl Ord for CAItem {
    fn cmp(&self, other: &CAItem) -> Ordering {
        self.feature_id.cmp(&other.feature_id)
            .then(self.sub_id.cmp(&other.sub_id))
            .then(self.activity.cmp(&other.activity))            
            .then(self.start_fb.cmp(&other.start_fb))
            .then(self.end_fb.cmp(&other.end_fb))
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