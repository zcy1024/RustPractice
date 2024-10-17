use std::env;
use std::process;

use minigrep::Config;

fn main() {
    // let args = env::args().collect::<Vec<String>>();
    // dbg!(args);

    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    // println!(
    //     "searching for \"{0}\" in file: {1}.",
    //     config.query, config.file_path
    // );

    if let Err(e) = minigrep::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
