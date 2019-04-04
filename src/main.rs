extern crate tree_sitter;
//#[macro_use]
extern crate serde_json;
extern crate tera;
//#[macro_use]
extern crate clap;
extern crate lazy_static;

mod beamer;
mod parsing;
mod process_file;
mod tree_traversal;

use clap::{App, Arg};
use std::{process, thread, time};
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

fn main() {
    let matches = App::new("faster-beamer")
        .version("1.0")
        .author("Stephan Seitz <stephan.seitz@fau.de>")
        .about("Incremental compiler for Beamer LaTeX presentations")
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .help("Sets a custom config file"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let is_server_mode = matches.is_present("server");
    let input_file = matches.value_of("INPUT").unwrap();

    pretty_env_logger::init();

    let options = process_file::CliOptions {
        server: is_server_mode,
    };

    if is_server_mode {
        use hotwatch::{Event, Hotwatch};
        let matches = matches.clone();

        let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
        hotwatch
            .watch(input_file, move |event: Event| {
                if let Event::Write(_) = event {
                    debug!("Input file has changed.");
                    let input_file = matches.value_of("INPUT").unwrap();
                    process_file::process_file(input_file, options);
                }
                if let Event::Remove(_) = event {
                    debug!("Input file deleted!");
                    process::exit(0);
                }
            })
            .expect("Failed to watch file!");
        info!("Server mode");
        info!("Watching {}", input_file);

        loop {
            thread::sleep(time::Duration::from_millis(100));
        }
    } else {
        process_file::process_file(input_file, options);
    }
}
