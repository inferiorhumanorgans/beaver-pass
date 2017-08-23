use regex::Regex;
use std::process::exit;
use serde_json;
use utils::{write_to_browser, write_to_syslog};
use subprocess;
use std::io::{Read};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct GetQuery {
    action: String,
    entry: String
}

fn find_file(filename: &str, searchpath: &Vec<&str>) -> Result<String, &'static str> {
    for path in searchpath {
        let lookhere = format!("{}/{}", path, filename);
        match fs::metadata(lookhere.clone()) {
            Ok(meta) => {
                if meta.is_file() {
                    return Ok(lookhere.clone());
                }
            },
            Err(_) => {}
        }
    }
    Err("Not Found")
}

pub fn execute(payload: &str, root: &String) {
    write_to_syslog("CMD=begin");

    let q: GetQuery = serde_json::from_str(payload).unwrap();
    let search_path = vec!["/usr/bin", "/usr/local/bin", "/opt/bin"];
    let gpg_executable = match find_file("gpg", &search_path) {
        Ok(gpg_path) => {
            write_to_syslog(format!("GPG found at: {}", gpg_path).as_str());
            gpg_path
        },
        Err(_) => {
            write_to_syslog(format!("Could not find GPG, looked here: {:?}", search_path).as_str());
            let json = json!({
                "result": "error",
                "description": "Couldn't find GPG"
            });
            write_to_browser(&json.to_string().as_str());
            panic!("Couldn't find GPG");
        }
    };

    let gpg_file = format!("{}/{}.gpg", root, q.entry);
    let gpg_options = &[
        gpg_executable.as_str(),
        "--decrypt",
        "--yes",
        "--quiet",
        "--use-agent",
        "--batch",
        gpg_file.as_str()
    ];

    let popen_config = subprocess::PopenConfig {
        stdout: subprocess::Redirection::Pipe,
        stderr: subprocess::Redirection::Merge,
        ..Default::default()
    };

    let mut decrypted = subprocess::Popen::create(gpg_options, popen_config).expect("ARGH");
    let retval = decrypted.wait();

    let mut decrypted_string = String::new();
    decrypted.stdout.as_ref().unwrap().read_to_string(&mut decrypted_string).unwrap();

    if retval.unwrap().success() == false {
        write_to_syslog("GPG failed");
        let json = json!({
            "result": "error",
            "description": decrypted_string
        });
        write_to_browser(json.to_string().as_str());
        exit(1);
    }

    let mut decrypted_reader = decrypted_string.split('\n');

    let password: &str = decrypted_reader.next().unwrap();

    let re = Regex::new(r":\s*").unwrap();
    let mut fields: HashMap<String, String> = decrypted_reader
    .filter(|line| re.is_match(line))
    .map(|kv| {
        let split_vec: Vec<&str> = re.splitn(kv, 2).collect();
        (split_vec[0].to_string(), split_vec[1].to_string())
    })
    .collect();
    fields.insert("password".to_string(), password.to_string());

    let json = json!({
        "result": "success",
        "fields": fields,
        "password": password
    });
    write_to_browser(&json.to_string().as_str());
}
