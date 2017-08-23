use regex::Regex;
use serde_json;
use utils::{write_to_browser, write_to_syslog};
use glob::glob;

#[derive(Serialize, Deserialize, Debug)]
struct SearchQuery {
    action: String,
    domain: String
}

fn expand_glob(root_path: &String, pattern: &str) -> Vec<String> {
    write_to_syslog(format!("Globbing: {:?}", pattern).as_str());
    let mut strings: Vec<String> = Vec::new();

    let regex_string: String = format!("(^{}/|\\.gpg$)", root_path);
    let re = Regex::new(regex_string.as_str()).unwrap();

    match glob(pattern) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(glob) => {
                        let entry_string = glob.into_os_string().into_string().unwrap();
                        let result = re.replace_all(entry_string.as_str(), "");
                        strings.push(result.clone().to_string())
                    },
                    Err(_) => {
                        write_to_browser("['error #1]");
                    }
                }
            }
            return strings;
        },
        Err(_) => {
            write_to_syslog(format!("glob didn't work: {:?}", pattern).as_str());
            let json = json!({
                "result": "error",
                "description": "Invalid pattern"
            });
            write_to_browser(json.to_string().as_str());
            panic!("Bad globbing.");
        }
    }
}

pub fn execute(payload: &str, root: &String) {
    write_to_syslog("CMD=search");

    let q: SearchQuery = serde_json::from_str(payload).unwrap();
    let message = format!("QUERY={:?}", q);
    write_to_syslog(message.as_str());

    let mut strings: Vec<String> = Vec::new();

    if !q.domain.is_empty() {
        let domain = str::split(q.domain.as_str(), '/').next().unwrap();
        let glob = format!("{}/**/{}/*.gpg", root, domain);
        strings.extend(expand_glob(root, glob.as_str()));

        let components: Vec<&str> = str::split(domain, ".").collect();

        if components.len() > 2 {
            let last: usize = components.len() - 2;
            let root_domain: String = components[last..].join(".");
            let root_glob = format!("{}/**/*{}/*.gpg", root, root_domain);
            strings.extend(expand_glob(root, root_glob.as_str()));
        }

    } else {
        let glob = format!("{}/**/*.gpg", root);
        strings.extend(expand_glob(root, glob.as_str()));
    }

    strings.sort();
    strings.dedup();

    let json = json!({
        "result": "success",
        "data": strings
    });
    write_to_browser(&json.to_string().as_str());
}
