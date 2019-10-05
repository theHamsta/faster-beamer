extern crate tree_sitter;
//#[macro_use]
extern crate serde_json;
extern crate tera;
//#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate latexcompile;
extern crate lopdf;
extern crate pretty_env_logger;
//extern crate tectonic;
extern crate cachedir;
extern crate md5;
extern crate rayon;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod beamer;
mod parsing;
mod process_file;
mod tree_traversal;

use clap::{App, Arg};
use std::{process, thread, time};
use std::path::Path;

fn main() {
    pretty_env_logger::init();

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
        .arg(
            Arg::with_name("unite")
                .short("u")
                .long("unite")
                .help("Unites all slides to a PDF (default is only to output newest slide)"),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Filename for output PDF")
                .index(2),
        )
        //.arg(
        //Arg::with_name("draft")
        //.short("d")
        //.help("Compile in draft mode")
        //)
        .get_matches();

    let is_server_mode = matches.is_present("server");
    let input_file = matches.value_of("INPUT").unwrap();
    let input_path = Path::new(input_file).parent().expect("Could not determine parent directory of input file");

    if is_server_mode {
        use hotwatch::{Event, Hotwatch};
        let matches = matches.clone();

        let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
        hotwatch
            .watch(input_path, move |event: Event| match event {
                Event::Write(file) | Event::Create(file) => {
                    info!("{:?} has changed.", file);
                    let input_file = matches.value_of("INPUT").unwrap();
                    if Path::new(&input_file).canonicalize().unwrap() == file.canonicalize().unwrap() {
                        info!("Processing {:?}.", file);
                        process_file::process_file(input_file, &matches);
                    }
                }
                Event::Remove(file) => {
                    info!("Input {:?} deleted!", file);
                }
                _ => {
                    info!("{:?}", event);
                }
            })
            .expect("Failed to watch file!");
        info!("Server mode");
        info!("Watching {}", input_file);

        loop {
            thread::sleep(time::Duration::from_millis(100));
        }
    } else {
        info!("Processing {:?}.", input_file);
        process_file::process_file(input_file, &matches);
    }
}
