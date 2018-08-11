#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;

pub mod fetch;
pub mod login;
mod demo;

#[cfg(test)]
mod test {
    use ::demo::*;
    use super::login::*;
    #[test]
    fn typed_example_test() {
        test_typed();
    }

    #[test]
    fn shall_load_cred_from_contents() {
        let contents = vec![9, 53, 50, 55, 53, 55, 51, 55, 52, 
            53, 50, 54, 102, 54, 51, 54, 98, 55, 51, 50, 49];
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
        let contents = vec![9, 53, 50, 55, 53, 55, 51, 55, 52, 
            53, 50, 54, 102, 54, 51, 54, 98, 55, 51, 50, 49];
        let login = new_login("Rust".to_string(), "Rocks!".to_string());
        let mut saved = Vec::new();
        login.save_to_temp(&mut saved);
        assert_eq!(contents, saved);
    }
}

extern crate hyper;
extern crate tokio_core;
use self::tokio_core::reactor::Core;

pub fn run() {
    let mut core = Core::new().unwrap();
    let input = "https://jiradc.int.net.nokia.com/rest/api/2/filter/145359";
    let login = login::Login::new().to_basic();
    let fut = fetch::fetch(&mut core, input, login);

    //schedule and run
    if let Err(_err) = core.run(fut) {
        error!("Something wrong here!");
    } else {
        info!("Completed now!");
    }
}
