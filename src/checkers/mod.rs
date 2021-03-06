pub mod analyze;
pub mod ca;
pub(crate) mod datatypes;
pub mod fs2;
pub(crate) mod persist;
pub(crate) mod records;
pub(crate) mod search;
pub mod sys;
pub(crate) mod utils;

#[cfg(test)]
mod test {
    use crate::checkers::utils::*;

    #[test]
    fn should_trim_newlines() {
        assert_eq!(get_leftmost("some name \n really long", 10), "some name ");
    }

    #[test]
    fn should_trim_width_no_newline() {
        assert_eq!(get_leftmost("some name really lone", 10), "some name ");
    }

    #[test]
    fn should_trim_shorter_string_newline() {
        assert_eq!(get_leftmost("some\nname", 10), "some");
    }

    #[test]
    fn should_trim_shorter_string_no_newline() {
        assert_eq!(get_leftmost("some name", 10), "some name");
    }
}
