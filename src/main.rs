#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure_derive;

mod beamer;
mod latexcompile;
mod parsing;
mod process_file;
mod tree_traversal;

use clap::{App, Arg};
use std::env;
use std::env::current_dir;
use std::path::Path;
use std::{thread, time};

fn main() {
    if env::var("RUST_LOG").is_err() {
        let mut builder = pretty_env_logger::formatted_builder();
        builder.parse_filters("info");
        builder.init();
    } else {
        pretty_env_logger::init();
    }

    let matches = App::new("faster-beamer")
        .version("0.1.6")
        .author("Stephan Seitz <stephan.seitz@fau.de>")
        .about("Incremental compiler for Beamer LaTeX presentations")
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
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
            Arg::with_name("pdfunite")
                .short("x")
                .long("pdfunite")
                .help("Unites all slides to a PDF using pdfunite"),
        )
        .arg(
            Arg::with_name("frame-numbers")
                .short("f")
                .long("frame-numbers")
                .help("Try to print correct frames numbers. This can harm cache performance when swapping frames."),
        )
        .arg(
            Arg::with_name("tree-sitter")
                .short("t")
                .long("tree-sitter")
                .help("Use tree-sitter to parse LaTeX (instead of regexes)"),
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

    let is_watch_mode = matches.is_present("watch");
    let input_file = matches.value_of("INPUT").unwrap();

    let cwd = current_dir().unwrap();
    let input_dir = Path::new(input_file)
        .parent()
        .unwrap_or(&cwd)
        .canonicalize()
        .unwrap_or(cwd.to_owned());

    info!("Processing {:?}.", input_file);
    let result = process_file::process_file(input_file, &matches);
    if result == Err(process_file::FasterBeamerError::InputFileNotExistent) {
        std::process::exit(-1);
    };

    if is_watch_mode {
        use hotwatch::{Event, Hotwatch};
        let matches = matches.clone();

        let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
        hotwatch
            .watch(input_dir, move |event: Event| match event {
                Event::Write(file) | Event::NoticeRemove(file) => {
                    trace!("{:?} has changed.", file);
                    thread::sleep(time::Duration::from_millis(50));
                    let input_file = matches.value_of("INPUT").unwrap();
                    match (Path::new(&input_file).canonicalize(), file.canonicalize()) {
                        (Ok(file), Ok(changed_file)) if file == changed_file => {
                            let path_str = file.to_str().unwrap();
                            info!("Processing {:?}.", &path_str);
                            let _result = process_file::process_file(path_str, &matches);
                        }
                        _ => {}
                    }
                }
                _ => {
                    trace!("{:?}", event);
                }
            })
            .expect("Failed to watch file!");
        info!("Watch mode");
        info!("Watching {}", input_file);

        loop {
            thread::sleep(time::Duration::from_millis(100));
        }
    }
}
