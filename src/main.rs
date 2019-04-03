extern crate tree_sitter;
#[macro_use]
extern crate tera;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate clap;

mod parsing;
mod process_file;
mod tree_traversal;

use clap::{App, Arg};
use std::{process, thread, time};

fn main() {
    let matches = App::new("codgen")
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
        .arg(
            Arg::with_name("OUTPUT")
                .help("Sets the output file to use")
                .required(true)
                .index(2),
        )
        .get_matches();

    let is_server_mode = matches.is_present("server");
    let input_file = matches.value_of("INPUT").unwrap();
    let output_file = matches.value_of("OUTPUT").unwrap();

    if is_server_mode {
        use hotwatch::{Event, Hotwatch};
        let matches = matches.clone();

        let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
        hotwatch
            .watch(input_file, move |event: Event| {
                if let Event::Write(_) = event {
                    println!("Input file has changed.");
                    let input_file = matches.value_of("INPUT").unwrap();
                    let output_file = matches.value_of("OUTPUT").unwrap();
                    process_file::process_file(input_file, output_file);
                }
                if let Event::Remove(_) = event {
                    println!("Input file deleted!");
                    process::exit(0);
                }
            })
            .expect("Failed to watch file!");
        println!("Server mode");
        println!("Watching {}", input_file);

        loop {
            thread::sleep(time::Duration::from_millis(100));
        }
    } else {
        process_file::process_file(input_file, output_file);
    }
}
