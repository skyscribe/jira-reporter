const DEFAULT_FB : u32 = 9999;

extern crate serde;
extern crate serde_json;

use super::caissue::CAIssue;
use super::super::utils::NA_STRING;
use super::super::datatypes::StoredData;

use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Ord, Eq, PartialOrd, PartialEq, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Eq, Clone, Serialize, Deserialize)]
pub struct CAItem {
    pub summary: String,
    pub key: String,
    pub sub_id : String, 
    pub description: String, 
    pub feature_id: String,
    pub team: String,
    pub start_fb: u32,
    pub end_fb: u32,
    pub efforts: i32,
    pub target: String,
    pub activity: Activity,
}

impl CAItem {
    pub fn from(issue: &CAIssue) -> CAItem {
        let special : &[_] = &['\t', '\n', '\r', ' '];
        let (subid, desc) = CAItem::get_summary(&issue.fields.summary);
        let activity = CAItem::get_type(issue.get_type());
        CAItem {
            summary: issue.fields.summary.clone(),
            key: issue.key.clone(),
            feature_id: issue.get_fid().to_string(),
            team: issue.get_team().trim_right_matches(special).to_string(),
            start_fb: convert_fb(issue.get_start()),
            end_fb: convert_fb(issue.get_end()),
            activity,
            sub_id: subid.to_string(),
            description: desc.to_string(),
            target: issue.get_target().to_string(),
            efforts: issue.get_efforts(),
        }
    }

    pub fn get_summary(summary: &str) -> (&str, &str) {
        //split by first " " only after trimming left spaces
        let summary = trim_leading_puncs(summary);
        match summary.find(|x:char| x==' ' || x == '\t') {
            Some(x) => {
                let (first, last) = summary.split_at(x);
                (trim_as_sub_fid(first), trim_leading_puncs(last))
            },
            None => {
                (trim_as_sub_fid(&summary), "")
            },
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
        } else if match_kwds(efs_kwds, value) {
    Activity::EFS
} else if match_kwds(et_kwds, value) { 
    Activity::ET
} else {
    Activity::SW
}
    }
}

fn convert_fb(value: &str) -> u32 {
    if value == NA_STRING {
        DEFAULT_FB
    } else {
        match u32::from_str_radix(value, 10) {
            Ok(x) => x,
            Err(_) => DEFAULT_FB,
        }
    }
}

//Trim leftmost spaces until meaningful characters
fn trim_leading_puncs(current: &str) -> &str {
    let skips: &[_] = &[' ', '\t', '\n', '-'];
    current.trim_left_matches(skips)
}

//Trim duplicate information from sub-fid str
fn trim_as_sub_fid(first: &str) -> &str {
    let skips: &[_] = &['-', ':'];
    get_substr_until_one_of(first, &[
        "-CP3", "-EFS", "-OM", "-OAM", "-CFAM", "-EI", "-Ei"
    ]).trim_right_matches(skips)
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

impl StoredData for CAItem {
    type Parsed = CAIssue;

    fn parse_from(issue: &Self::Parsed) -> Self {
        Self::from(issue)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) mod tests {
    extern crate serde;
    extern crate serde_json;
    use super::super::caissue::*;
    use super::*;
    #[test]
    fn should_convert_parsed_fields() {
        let json = get_test_json("Feature_ID_xxx_yyy", "SW", "Team yyy");
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        
        let converted = CAItem::from(&issue.unwrap());
        assert_eq!(converted.summary, "Feature_ID_xxx_yyy");
        assert_eq!(converted.feature_id, "Feature_ID");
        assert_eq!(converted.team, "Team yyy");
        assert_eq!(converted.start_fb, 1808);
        assert_eq!(converted.end_fb, 1809);
        assert_eq!(converted.activity, Activity::SW);
    }

    pub fn get_test_json(summary:&str, activity: &str, team: &str) -> String {
        let hdr = r#"{
            "expand" : "",
            "id": "",
            "self": "",
            "key": "",    
            "fields": {
                "summary": ""#;
        String::from(hdr) + summary + r#"",
                "customfield_37381":"Feature_ID",
                "customfield_38727":""# + team + r#"",
                "customfield_38723":"PT4",
                "customfield_38694":"1808",
                "customfield_38693":"1809",
                "timeoriginalestimate":24000,
                "customfield_38750":{ "value": ""#
        + activity + r#""}
        }}"#
    }

    #[test]
    fn should_parse_desc_by_space() {
        let json = get_test_json("Leading - something else", "SW", "X");
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let converted = CAItem::from(&issue.unwrap());
        
        assert_eq!(converted.sub_id, "Leading");
        assert_eq!(converted.description, "something else");
    }

    #[test]
    fn parse_desc_by_special_chars() {
        //Note raw string is needed to pass in "special characters"
        let json = get_test_json(r#"Leading \t \t something else"#, "SW", r#"Team yyy\t\t"#);
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok(), "Parsing result:{:?}", issue);
        let converted = CAItem::from(&issue.unwrap());
        
        assert_eq!(converted.sub_id, "Leading");
        assert_eq!(converted.description, "something else"); 
        assert_eq!(converted.team, "Team yyy");
    }

    #[test]
    fn should_translate_efs_type() {
        let json = get_test_json("Leading - something else", "Entity Specification", "X");
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        assert_eq!(CAItem::from(&issue.unwrap()).activity, Activity::EFS);   
    }

    #[test]
    fn should_translate_et_type() {
        let json = get_test_json("Leading - something else", "Entity Testing", "X");
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        assert_eq!(CAItem::from(&issue.unwrap()).activity, Activity::ET);   
    }

    #[test]
    fn should_compare_caitem_by_given_order() {
        let json = get_test_json("Leading - something else", "EFS", "X");
        use std::cmp::Ordering;
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let item = CAItem::from(&issue.unwrap());
        
        let mut item1 = item.clone();
        item1.activity = Activity::SW;
        assert_eq!(item.cmp(&item1), Ordering::Less);

        let mut item2 = item1.clone();
        item2.start_fb = 1809;
        assert!(item1 < item2);

        let mut item3 = item2.clone();
        item3.end_fb = 1810;
        assert!(item2 < item3);

        let mut item4 = item3.clone();
        item4.activity = Activity::ET;
        assert!(item3 < item4);
    }

    #[test]
    fn should_compare_by_subfid_for_same_feature() {
        use std::cmp::Ordering;
        let json = get_test_json("Feature-A-a - something else", "EFS", "X");
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let item = CAItem::from(&issue.unwrap());

        let mut item1 = item.clone();
        item1.summary = "Feature-A-b - something else".to_string();
        item1.reparse();

        let mut item2 = item.clone();
        item2.summary = "Feature-A-a - something else".to_string();
        item2.activity = Activity::SW;
        item2.reparse();

        assert_eq!(item.cmp(&item1), Ordering::Less);
        assert_eq!(item.cmp(&item2), Ordering::Less);
        assert_eq!(item1.cmp(&item2), Ordering::Greater);
    }

    #[test]
    fn should_sort_type_first_when_schedule_is_wrong() {
        use std::cmp::Ordering;
        let json = get_test_json("Leading - something else", "SW", "X");
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        let item = CAItem::from(&issue.unwrap());

        let mut item1 = item.clone();
        item1.end_fb = 1810;
        item1.activity = Activity::EFS;
        assert_eq!(item.cmp(&item1), Ordering::Greater);
    }

    #[test]
    fn should_normalize_fid_dup_cp3() {
        parse_and_check_against("Feature-A-a-CP3 something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_efs() {
        parse_and_check_against("Feature-A-a-EFS something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_om_cp3() {
        parse_and_check_against("Feature-A-a-OM-CP3 something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_oam() {
        parse_and_check_against("Feature-A-a-OAM something else",
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_om() {
        parse_and_check_against("Feature-A-a-OM something else",
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_cfam() {
        parse_and_check_against("Feature-A-a-CFAM-xx something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_Ei() {
        parse_and_check_against("Feature-A-a-Ei something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_dup_EI() {
        parse_and_check_against("Feature-A-a-EI something else", 
            "Feature-A-a", "something else");
    }

    #[test]
    fn should_normalize_fid_with_ending_dash() {
        parse_and_check_against("Feature-A-a2- something else", 
            "Feature-A-a2", "something else");
    }

    #[test]
    fn should_normalize_fid_with_ending_colon() {
        parse_and_check_against("Feature-A-a2: something else", 
            "Feature-A-a2", "something else");
    }

    #[test]
    fn should_normalize_fid_with_leading_spaces() {
        parse_and_check_against("      Feature-A-a2 something else", 
            "Feature-A-a2", "something else");
    }

    #[test]
    fn should_normalize_fid_with_no_desc() {
        parse_and_check_against(" Feature-A-OM-CP3",
            "Feature-A", "");
    }

    fn parse_and_check_against(summary: &str, expected: &str, trailing: &str) {
        let summary = String::from(summary);
        let (subid, desc) = CAItem::get_summary(&summary);
        assert_eq!(subid, expected);
        assert_eq!(desc, trailing);
    }
}