extern crate regex;
extern crate byteorder;
extern crate syslog;
extern crate subprocess;
extern crate glob;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::io::{self, Read};
use std::str;
use std::env;
use serde_json::{Value};
use std::path::PathBuf;

use byteorder::{ReadBytesExt, LittleEndian};

pub mod commands;
pub mod utils;
use utils::{write_to_syslog};

fn main() {
    let mut stdin = io::stdin();

    loop {
        let data_len: usize = match stdin.read_u32::<LittleEndian>() {
            Ok(len) => len,
            Err(_) => std::process::exit(0)
        } as usize;

        let mut buffer = vec![0u8; data_len];
        stdin.read_exact(&mut buffer).unwrap();

        let payload: &str = match str::from_utf8(&buffer) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {:?}", e),
        };

        let json: Value = serde_json::from_str(payload).unwrap();
        let action: &str = json["action"].as_str().unwrap();

        let path: PathBuf = env::home_dir().unwrap().join(".password-store");
        let root: String = path.clone().into_os_string().into_string().unwrap();

        match action {
            "search" => commands::search::execute(&payload, &root),
            "get" => commands::get::execute(&payload, &root),
            _ => {
                write_to_syslog("Unknown action, skipping.");
            }
        }
    }
}
