//
// process_file.rs
// Copyright (C) 2019 seitz_local <seitz_local@lmeXX>
// Distributed under terms of the GPLv3 license.
//
use crate::beamer::get_frames;
use crate::parsing;

use log::Level::Trace;

use crate::tree_traversal;
use crate::tree_traversal::TraversalOrder;
use clap::ArgMatches;
use latexcompile::{LatexCompiler, LatexError, LatexInput};
use lopdf::Document;
use std::collections::HashMap;
use std::fs::write;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use tectonic;
use tree_sitter::Node;

pub fn process_file(input_file: &str, _app: &ArgMatches) {
    if !Path::new(&input_file).exists() {
        eprintln!("Could not open {}", input_file);
        return;
    }

    let parsed_file = parsing::ParsedFile::new(input_file.to_string());
    debug!("{}", parsed_file.syntax_tree.root_node().to_sexp());

    let frames = get_frames(&parsed_file);

    info!("Found {} frames.", frames.len());

    for (i, f) in frames.iter().enumerate() {
        debug!("Frame {}:", i);
        debug!("{}", parsed_file.get_node_string(&f));
        debug!("");
    }

    if log_enabled!(Trace) {
        let root_node = parsed_file.syntax_tree.root_node();
        let mut stack = vec![root_node];

        while !stack.is_empty() {
            let current_node = stack.pop().unwrap();
            if current_node.kind() != "ERROR" {
                debug!(
                    "\n{}:\n\t {}",
                    current_node.kind(),
                    parsed_file.get_node_string(&current_node),
                );
            }

            for i in (0..current_node.named_child_count()).rev() {
                stack.push(current_node.named_child(i).unwrap());
            }
        }
    }

    let document_envs = tree_traversal::get_children(
        parsed_file.syntax_tree.root_node(),
        &|n| n.kind() == "document_env",
        true,
        TraversalOrder::BreadthFirst,
    );
    //let latex = parsed_file.file_content;
    //let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
    //println!("Output PDF size is {} bytes", pdf_data.len());

    let mut dict = HashMap::new();
    dict.insert("test".into(), "Minimal".into());
    // provide the folder where the file for latex compiler are found
    let input = LatexInput::from("/home/stephan/projects/LMEbeamer4_4");
    // create a new clean compiler enviroment and the compiler wrapper
    let compiler = LatexCompiler::new(dict).unwrap();
    // run the underlying pdflatex or whatever
    let result = compiler.run(&input_file, &input).unwrap();

    let document = Document::load_from(&result[..]).unwrap();
    let pages = document.get_pages();
    println!("{} pages", pages.iter().len());
    // copy the file into the working directory
    let output = ::std::env::current_dir().unwrap().join("out.pdf");
    assert!(write(output, result).is_ok());

    //let root_node = parsed_file.syntax_tree.root_node();
    //let mut stack = vec![root_node];

    //while !stack.is_empty() {
    //let current_node = stack.pop().unwrap();
    //if current_node.kind() != "ERROR" {
    //println!(
    //"\n{}:\n\t {}",
    //current_node.kind(),
    //parsed_file.get_node_string(&current_node)
    //);
    //}

    //for i in (0..current_node.named_child_count()).rev() {
    //stack.push(current_node.named_child(i).unwrap());
    //}
    //}

    //let node_types = vec!["text_env", "group"];
    //for t in node_types {
    //let comments = parsed_file.get_nodes_of_type(t.to_string());
    //println!("");
    //println!("Found {} {}s:", comments.len(), t);
    //for c in comments {
    //let text = &parsed_file.file_content[c.start_byte()..c.end_byte()];
    //println!("{}", text);
    //}
    //}

    //let node_types = vec!["text_env"];
    //let mut frames = Vec::new();
    //for t in node_types {
    //let comments = parsed_file.get_nodes_of_type(t.to_string());
    //println!("");
    //println!("Found {} {}s:", comments.len(), t);
    //for c in comments {
    //let children = tree_traversal::get_children(
    //c,
    //&|n| beamer::is_frame_environment(n, &parsed_file),
    //true,
    //&|n: Node| {
    //n.kind() == "begin"
    //&& parsed_file
    //.get_node_string(&n)
    //.to_string()
    //.contains("{frame}")
    //},
    //true,
    //TraversalOrder::DepthFirst,
    //);
    //if children.len() == 1 {
    //println!("");
    //println!("{}", parsed_file.get_node_string(&c));
    //frames.push(c);
    //}
    //}
    //println!("Found {} frames", frames.len());
    //}
}
