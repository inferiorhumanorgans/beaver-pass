extern crate syslog;
extern crate byteorder;

use std::io::{self, Write};
use byteorder::{WriteBytesExt, LittleEndian};
use syslog::{Facility,Severity};

pub fn write_to_browser(output: &str) {
    let mut stdout = io::stdout();
    let output_len = output.len();
    stdout.write_u32::<LittleEndian>(output_len as u32).unwrap();
    stdout.write(&output.as_bytes()).unwrap();
    stdout.flush().unwrap();
}

pub fn write_to_syslog(message: &str) -> usize {
    let writer = match syslog::unix(Facility::LOG_USER) {
        Err(e)     => panic!("Impossible to connect to syslog: {:?}", e),
        Ok(writer) => writer
    };

    let qualified_string = format!("[beaver-pass] {}", message);
    match writer.send_3164(Severity::LOG_ALERT, qualified_string) {
        Ok(length) => length,
        Err(error) => panic!("Couldn't write to syslog: {}", error)
    }
}
