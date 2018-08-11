extern crate hyper;
extern crate rpassword;

use std::io::{stdin, stdout, Write};
use self::hyper::header::Basic;
use self::rpassword::prompt_password_stdout;

pub struct Login {
    username: String,
    password: String,
}

impl Login {
    pub fn new() -> Login {
        let mut buf1 = String::new();
        loop {
            print!("Please input your account:");
            stdout().flush().unwrap();
            //username/password have to be 3~20 characters and newline is not needed
            // also to consider the \r\n extra characters read in by read_line() call
            if let Ok(name_len @ 5...20) = stdin().read_line(&mut buf1) {
                let passwd_string = prompt_password_stdout("Please input your password:");
                if let Ok(passwd_string) = passwd_string {
                    return Login{
                        username: buf1[0..name_len-2].to_string(),
                        password: passwd_string,
                    }
                }
            }
        }
    }

    pub fn to_basic(&self) -> Basic {
        //we just show password as a series of stars
        info!("Now, user=<{}>, pwd=<{}>", self.username, String::from_utf8(
                vec![42; self.password.len()]).unwrap());
        Basic {
            username: self.username.clone().into(),
            password: self.password.clone().into(),
        }
    }
}