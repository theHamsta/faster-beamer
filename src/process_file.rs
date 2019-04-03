//
// process_file.rs
// Copyright (C) 2019 seitz_local <seitz_local@lmeXX>
// Distributed under terms of the GPLv3 license.
//
use crate::parsing;
use std::fs;
use std::path::Path;

pub fn process_file(input_file: &str, output_file: &str) {
    if !Path::new(&input_file).exists() {
        eprintln!("Could not open {}", input_file);
        return;
    }

    let parsed_file = parsing::ParsedFile::new(input_file.to_string());
    fs::write(output_file, parsed_file.file_content).expect("Unable to write output file");
}
