extern crate hyper;

use std::io::{stdin, stdout, Write};
use self::hyper::header::Basic;

pub struct Login {
    username: String,
    password: String,
}

impl Login {
    pub fn new() -> Login {
        let mut buf1 = String::new();
        let mut buf2 = String::new();
        loop {
            print!("Please input your account:");
            stdout().flush().unwrap();
            //username/password have to be 3~20 characters and newline is not needed
            // also to consider the \r\n extra characters read in by read_line() call
            if let Ok(name_len @ 5...20) = stdin().read_line(&mut buf1) {
                print!("Please input your password:");
                stdout().flush().unwrap();
                if let Ok(pwd_len @ 5...20) = stdin().read_line(&mut buf2) {
                    return Login{
                        username: buf1[0..name_len-2].to_string(),
                        password: buf2[0..pwd_len-2].to_string(),
                    }
                }
            }
        }
    }

    pub fn to_basic(&self) -> Basic {
        info!("Now, user=<{}>, pwd=<{}>", self.username, self.password);
        Basic {
            username: self.username.clone().into(),
            password: self.password.clone().into(),
        }
    }
}