//use std::io::{BufWriter, Write};
use std::fmt::format;
//use std::fs::File;

use super::caitem::{Activity, CAItem};
use crate::checkers::utils::get_leftmost;

pub struct PipelineInfo<'a> {
    sub_id: &'a String,
    description: &'a String,
    team: &'a String,
    start_fb: u32,
    end_fb: u32,
    activity: &'a Activity,
    //efforts: i32,
}

impl<'a> PipelineInfo<'a> {
    pub fn from_item<'c: 'a>(item: &'c CAItem) -> PipelineInfo {
        PipelineInfo {
            sub_id: &item.sub_id,
            description: &item.description,
            team: &item.team,
            start_fb: item.start_fb,
            end_fb: item.end_fb,
            activity: &item.activity,
            //efforts: item.efforts,
        }
    }

    //get its own schedule info
    fn get_sched(&self, first_fb: u32) -> (u32, u32) {
        let mut offset: i32 = self.start_fb as i32 - first_fb as i32;
        if offset > 20 {
            offset = 20;
        }

        if self.end_fb < self.start_fb {
            if offset < 0 {
                (0, 0)
            } else {
                (offset as u32, 1)
            }
        } else {
            let mut lead_time = self.end_fb - self.start_fb + 1;
            if lead_time > 12 {
                lead_time -= 87; //1901 - 1813 = 1
            }

            if offset < 0 {
                let span = if lead_time as i32 + offset < 0 {
                    0
                } else {
                    (lead_time as i32 + offset) as u32
                };
                (0, span)
            } else {
                (offset as u32, lead_time)
            }
        }
    }

    //format pipeline row by start/end
    pub fn generate_schedule_row(&self, first_fb: u32, max_span: u32) -> String {
        let mut output = format(format_args!(
            "{:15}|{:30}|{:3}|{:8}|",
            get_leftmost(self.sub_id, 15),
            get_leftmost(self.description, 30),
            self.activity,
            get_leftmost(self.team, 8)
        ));

        let (offset, span) = self.get_sched(first_fb);
        if self.start_fb == 9999 || offset >= max_span {
            for _i in 0..max_span - 1 {
                output += "    |";
            }
            output += "x   ";
            return output;
        }

        //not started part
        for _i in 0..offset {
            output += "    |";
        }

        //planed
        let middle = if offset + span >= max_span {
            max_span - offset - 1
        } else {
            span
        };
        for _i in 0..middle {
            output += "x   |";
        }

        //beyond end - 1
        for _i in offset + middle..(max_span - 1) {
            output += "    |";
        }

        if offset + span >= max_span {
            output += "x   ";
        } else {
            output += "    "
        }
        output
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    use super::super::caissue::CAIssue;
    use super::super::caitem::tests::get_test_json;
    use super::{CAItem, PipelineInfo};

    #[test]
    fn should_get_schedule_info_without_round() {
        let item = get_test_item();
        let pipeline_info = PipelineInfo::from_item(&item);
        let (start, span) = pipeline_info.get_sched(1807);
        assert_eq!(start, 1);
        assert_eq!(span, 2);
        assert_eq!(pipeline_info.start_fb, 1808);
        assert_eq!(pipeline_info.end_fb, 1809);
        assert_eq!(pipeline_info.team, "X");
    }

    #[test]
    fn should_get_shecule_info_with_rounded_plan() {
        let mut item = get_test_item();
        item.end_fb = 1901; //start = 1807

        let pipeline_info = PipelineInfo::from_item(&item);
        let (start, span) = pipeline_info.get_sched(1807);
        assert_eq!(start, 1);
        assert_eq!(span, 7);
    }

    #[test]
    fn should_get_shecule_info_with_unknown_plan() {
        let mut item = get_test_item();
        item.start_fb = 9999;
        item.end_fb = 9999; //start = 1807

        let pipeline_info = PipelineInfo::from_item(&item);
        let (start, span) = pipeline_info.get_sched(1807);
        assert_eq!(start, 20);
        assert_eq!(span, 1);
    }

    #[test]
    fn should_get_shecule_info_with_started_before_first() {
        let mut item = get_test_item();
        item.start_fb = 1801;
        item.end_fb = 1807;

        let pipeline_info = PipelineInfo::from_item(&item);
        let (start, span) = pipeline_info.get_sched(1806);
        assert_eq!(start, 0);
        assert_eq!(span, 2);
    }

    fn get_test_item() -> CAItem {
        let json = get_test_json("Fid-A-a - description", "SW", "X");
        let issue = serde_json::from_str::<CAIssue>(&json);
        assert!(issue.is_ok());
        CAItem::from(&issue.unwrap())
    }

    #[test]
    fn should_generate_pipeline_item_without_rounding() {
        check_pipeline_for(1808, 1809,
            "Fid-A-a        |description                   |SW |X       |    |    |x   |x   |    |    ");
    }

    #[test]
    fn should_generate_pipeline_item_with_long_leading() {
        check_pipeline_for(1808, 1901,
            "Fid-A-a        |description                   |SW |X       |    |    |x   |x   |x   |x   ");
    }

    #[test]
    fn should_generate_pipeline_item_with_unplanned() {
        check_pipeline_for(9999, 9999,
            "Fid-A-a        |description                   |SW |X       |    |    |    |    |    |x   ");
    }

    #[test]
    fn should_generate_pipeline_item_with_planned_until_last() {
        check_pipeline_for(1808, 1810,
            "Fid-A-a        |description                   |SW |X       |    |    |x   |x   |x   |    ");
    }

    #[test]
    fn should_generate_pipeline_item_with_planned_no_ending() {
        check_pipeline_for(1808, 9999,
            "Fid-A-a        |description                   |SW |X       |    |    |x   |x   |x   |x   ");
    }

    #[test]
    fn should_generate_pipeline_item_with_start_as_first() {
        check_pipeline_for(1806, 1808,
            "Fid-A-a        |description                   |SW |X       |x   |x   |x   |    |    |    ");
    }

    #[test]
    fn should_generate_pipeline_item_with_done_before_first() {
        check_pipeline_for(1801, 1802,
            "Fid-A-a        |description                   |SW |X       |    |    |    |    |    |    ");
    }

    #[test]
    fn should_generate_pipeline_item_with_start_before_first() {
        check_pipeline_for(1801, 1807,
            "Fid-A-a        |description                   |SW |X       |x   |x   |    |    |    |    ");
    }

    fn check_pipeline_for(start: u32, end: u32, expected: &str) {
        let mut item = get_test_item();
        item.start_fb = start;
        item.end_fb = end;
        let p = PipelineInfo::from_item(&item);
        let line = p.generate_schedule_row(1806, 6);
        assert_eq!(line, expected);
    }
}
