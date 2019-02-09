pub mod fetcher;
pub mod login;

#[cfg(test)]
mod test {
    use super::login::{new_login, Login};

    #[test]
    fn shall_load_cred_from_contents() {
        let contents = vec![
            9, 53, 50, 55, 53, 55, 51, 55, 52, 53, 50, 54, 102, 54, 51, 54, 98, 55, 51, 50, 49,
        ];
        let login = new_login("Rust".to_string(), "Rocks!".to_string());
        if let Some(result) = Login::load_from_vec(contents) {
            assert_eq!(login, result);
        } else {
            assert!(false, "not parsed!");
        }
    }

    #[test]
    fn shall_write_cred_properly() {
        //9 '5' '2' '7' '5' '7' '3' '7' '4' => 52 75 73 74 => Rust
        //  '5' '2' '6' 'f' '6' '3' '6' 'b' '7' '3' '2' '1' => Rocks!
        let contents = vec![
            9, 53, 50, 55, 53, 55, 51, 55, 52, 53, 50, 54, 102, 54, 51, 54, 98, 55, 51, 50, 49,
        ];
        let login = new_login("Rust".to_string(), "Rocks!".to_string());
        let mut saved = Vec::new();
        login.save_to_temp(&mut saved);
        assert_eq!(contents, saved);
    }
}
