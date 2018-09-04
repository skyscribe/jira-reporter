use super::sysissue::SysIssue;
use super::super::datatypes::StoredData;
use std::cmp::Ord;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct SysItem {
    pub summary: String,
    pub title: String,
    pub release: String,
    pub status: String,
    pub area: String,
    pub key: String,
}

impl SysItem {
    pub fn from(raw: &SysIssue) -> SysItem {
        SysItem {
            summary: raw.fields.summary.clone(),
            title: raw.get_title(),
            release: raw.get_release(),
            status: raw.get_status(),
            area: raw.get_area(),
            key: raw.key.clone(),
        }
    }

    pub fn get_fid<'a>(&'a self) -> &'a str {
        match self.summary.find(' ') {
            Some(x) => &self.summary[0..x],
            _ => &(self.summary),
        }
    }

    pub fn is_oam_feature(&self) -> bool {
        self.area.contains("OAM") || self.area.contains("Operability")
    }
}

impl Ord for SysItem {
    fn cmp(&self, other: &SysItem) -> Ordering {
        self.area.cmp(&other.area)
            .then(self.summary.cmp(&other.summary))
            .then(self.status.cmp(&other.status))
            .then(self.key.cmp(&other.key))
    }
}

impl PartialOrd for SysItem {
    fn partial_cmp(&self, other:&SysItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SysItem {
    fn eq(&self, other:&SysItem) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl StoredData for SysItem {
    type Parsed = SysIssue;

    fn parse_from(issue: &Self::Parsed) -> Self {
        Self::from(issue)
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    extern crate serde;
    use super::*;
    use checkers::sys::sysissue::tests::*;

    #[test]
    fn should_parse_fid_from_summary_line() {
        let item = get_test_item();
        assert_eq!(item.get_fid(), "Sample");
    }

    fn get_test_item() -> SysItem {
        let json = get_test_json();
        let issue = serde_json::from_str::<SysIssue>(&json);
        assert!(issue.is_ok(), "{:?}", issue);
        SysItem::from(&issue.unwrap())
    }

    #[test]
    fn should_filter_oam_area_by_first_world_with_given_kws() {
        let mut item = get_test_item();
        item.area = "OAM - XXX".to_string();
        assert!(item.is_oam_feature(), "{:?}", item);

        let mut item1 = item.clone();
        item1.area = "Operability xxx".to_string();
        assert!(item1.is_oam_feature(), "{:?}", item);
    }
}