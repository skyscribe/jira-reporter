use super::fs2issue::Fs2Issue;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fs2Item {
    pub summary: String,
    pub efforts: i32,
    pub title: String,
}

impl Fs2Item {
    pub fn from(raw: &Fs2Issue) -> Fs2Item {
        Fs2Item {
            summary: raw.fields.summary.clone(),
            title: raw.get_title_display(),
            efforts: raw.get_efforts().map(|x| x as i32).unwrap_or(-1),
        }
    }

    pub fn has_efforts(&self) -> bool {
        self.efforts != -1
    }
}