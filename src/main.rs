use std::{fs::File, io::Read, path::Path, str::from_utf8};

use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use curl::easy::Easy;

fn main() {
    let args = parse_args();
    let config_path = Path::new(args.value_of("CONFIG").unwrap());
    let target_user = args.value_of("TARGET").unwrap().to_string();
    let config_data = read_config(config_path);
    let mut response_buf = Vec::new();
    get_response_from_github(&config_data, &target_user, &mut response_buf);
    get_presentable_output(&target_user, &response_buf.join(""));
}

fn read_config(config_path: &Path) -> Vec<String> {
    let mut config_file = File::open(config_path).expect(
        format!(
            "Error opening config file {}",
            config_path.to_str().unwrap()
        )
        .as_str(),
    );
    let mut read_buf = Vec::new();
    config_file.read_to_end(&mut read_buf).unwrap();
    String::from_utf8(read_buf)
        .unwrap()
        .lines()
        .map(|str| str.to_string())
        .collect::<Vec<String>>()
}

fn get_response_from_github(
    config_data: &Vec<String>,
    target_user: &String,
    resp_buf: &mut Vec<String>,
) {
    let mut curl_handle = Easy::new();

    curl_handle.username(config_data[0].as_str()).unwrap();
    curl_handle.password(config_data[1].as_str()).unwrap();
    curl_handle
        .url(format!("https://api.github.com/users/{}/repos", target_user).as_str())
        .unwrap();
    curl_handle.useragent("request").unwrap();
    let mut curl_transfer = curl_handle.transfer();
    curl_transfer
        .write_function(|response| {
            let decoded = from_utf8(response).unwrap();
            resp_buf.push(decoded.to_string());
            Ok(response.len())
        })
        .unwrap();
    curl_transfer.perform().unwrap();
}

fn get_presentable_output(username: &String, json_string: &String) {
    let json = json::parse(json_string).unwrap();
    println!("Public repo statistics for user {}:", username);
    println!(
        "{0: <30} | {1: ^12} | {2: >12} | {3: >12} | {4: >12} | {5:^6} | {6:^6} |",
        "Repo Name", "Language", "Created at", "Last push", "Last update", "Forks", "Stars"
    );
    println!(
        "{0:-<30} | {0:-^12} | {0:->12} | {0:->12} | {0:->12} | {0:-^6} | {0:-^6} |",
        ""
    );
    for entry in json.members() {
        println!(
            "{0: <30} | {1: ^12} | {2: >12} | {3: >12} | {4: >12} | {5:^6} | {6:^6} |",
            entry["name"],
            entry["language"].as_str().unwrap_or_else(|| "Unknown"),
            &entry["created_at"].as_str().unwrap()[..10],
            &entry["pushed_at"].as_str().unwrap()[..10],
            &entry["updated_at"].as_str().unwrap()[..10],
            entry["forks_count"].as_i32().unwrap(),
            entry["stargazers_count"].as_i32().unwrap()
        );
    }
}

fn parse_args() -> ArgMatches<'static> {
    App::new("Github API Repo Statistics")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Shows overall statistics of public repos of provided user")
        .arg(
            Arg::with_name("TARGET")
                .help("Target github account to get statistics from")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("CONFIG")
                .help("Config file containing your github username and token (in new lines)")
                .required(true)
                .index(2),
        )
        .get_matches()
}
