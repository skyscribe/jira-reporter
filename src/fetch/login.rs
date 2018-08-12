extern crate hyper;
extern crate rpassword;
extern crate hex;

use std::io::{stdin, stdout, Write};
use self::hyper::header::Basic;
use self::rpassword::prompt_password_stdout;
use std::fs;
use self::hex::{encode, decode};

const CRED_FILE : &str = ".cred.bin";

#[derive(PartialEq, Debug)]
pub struct Login {
    username: String,
    password: String,
}

impl Login {
    pub fn new() -> Login {
        match Login::load_credentials() {
            Some(login) => login,
            None => {
                let login = Login::create_from_terminal();
                login.save_credentials();
                login
            }
        }
    }

    fn create_from_terminal() -> Login {
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

    pub fn load_credentials() -> Option<Login> {
        match fs::read(CRED_FILE) {
            Ok(content) => Login::load_from_vec(content),
            Err(_) => {
                warn!("Either file not exist or not readalbe:{}", CRED_FILE);
                None
            }
        }
    }

    pub fn load_from_vec(content: Vec<u8>) -> Option<Login> {
        if content.len() < 5 {
            error!("content is too short!");
            return None;
        }

        if content.len() % 2 != 1 {
            error!("Invalid length, must be odd!");
            return None;
        }

        let sep: usize = content[0] as usize;
        let v8_slice_to_string = |vec : &[u8]| String::from_utf8(vec.to_vec()).unwrap();
        let decode_string = |slice: &[u8] | v8_slice_to_string(&decode(v8_slice_to_string(slice)).unwrap());
        if sep < (content.len() - 1)/2  {
            Some(Login{
                username: decode_string(&content[1..sep]),
                password: decode_string(&content[sep..content.len()]),
            })
        } else {
            warn!("Invalid credential file = {}, is this first time use?", CRED_FILE);
            None
        }
    }

    pub fn save_credentials(&self) -> Option<String> {
        let mut contents : Vec<u8> = Vec::new();
        self.save_to_temp(&mut contents);
        match fs::write(CRED_FILE, contents) {
            Ok(()) => {
                info!("New credentials saved for future use!");
                None
            },
            Err(_) => {
                error!("Writing into file failed!");
                Some("Write error".to_string())
            },
        }
    }

    pub fn save_to_temp(&self, contents: &mut Vec<u8>) {
        contents.push((2 * self.username.len()+1) as u8);
        contents.extend(encode(&self.username).as_bytes());
        contents.extend(encode(&self.password).as_bytes());
    }
}

pub fn new_login(username: String, password: String) -> Login {
    Login{
        username,
        password,
    }
}