pub(crate) mod search;

pub(crate) mod fs2issue;
pub mod fs2checker;

pub(crate) mod caissue;
pub mod cachecker;

#[cfg(test)]
mod test {
    use checkers::cachecker::get_leftmost;

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