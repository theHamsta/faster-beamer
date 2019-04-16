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
use cachedir::CacheDirConfig;
use clap::ArgMatches;
use latexcompile::{LatexCompiler, LatexError, LatexInput};
use lopdf::Document;
use md5;
use std::collections::{HashMap, HashSet};
use std::fs::write;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use tectonic;
use tree_sitter::Node;
pub fn process_file(input_file: &str, matches: &ArgMatches) {
    let input_path = Path::new(&input_file);
    if !input_path.is_file() {
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
            if current_node.kind() == "ERROR" {
                eprintln!(
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

    let document_env = tree_traversal::get_children(
        parsed_file.syntax_tree.root_node(),
        &|n| n.kind() == "document_env",
        true,
        TraversalOrder::BreadthFirst,
    );
    let preamble = if document_env.len() == 1 as usize {
        debug!("Document body");
        debug!("{}", parsed_file.get_node_string(&document_env[0]));
        debug!("Preamble");
        parsed_file.file_content[0..document_env[0].start_byte()].to_owned()
    } else {
        eprintln!(
            "Could not find document enviroment with tree_sitter ({})",
            input_file
        );
        let find = parsed_file.file_content.find("\\begin{document}");
        match find {
            Some(x) => Some(parsed_file.file_content[..x].to_owned()),
            None => None,
        }
        .unwrap()
    };
    debug!("{}", &preamble);
    

    //let latex = parsed_file.file_content;
    //let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
    //println!("Output PDF size is {} bytes", pdf_data.len());
    //
    let cachedir: PathBuf = CacheDirConfig::new("faster-beamer")
        .get_cache_dir()
        .unwrap()
        .into();

    //let hash = md5::compute(&preamble);

    let input = LatexInput::from(input_path.parent().unwrap().to_str().unwrap());
    let mut generated_documents = HashMap::new();
    let mut dict = HashMap::new();
    dict.insert("test".into(), "Minimal".into());
    let compiler = LatexCompiler::new(dict).unwrap();
    let mut command = &mut Command::new("pdfunite");
    for f in frames {
        //// provide the folder where the file for latex compiler are found
        //// create a new clean compiler enviroment and the compiler wrapper
        //// run the underlying pdflatex or whatever
        let compile_string = preamble.to_owned()
            + "\n\\begin{document}\n"
            + parsed_file.get_node_string(&f)
            + "\n\\end{document}\n";
        debug!("{}", compile_string);
        //let result: Vec<u8> = tectonic::latex_to_pdf(&compile_string).expect("processing failed");
        //println!("Output PDF size is {} bytes", result.len());
        let hash = md5::compute(&compile_string);

        let output = cachedir.join(format!("{:x}.pdf", hash));
        let pdf_data = if output.is_file() {
            //let mut f = File::open(&output).unwrap();
            //let mut buffer = Vec::new();
            //// read the whole file
            //f.read_to_end(&mut buffer).expect(&format!(
            //"Failed to read file {}",
            //&output.to_str().unwrap_or("")
            //));
            //buffer
        } else {
            let temp_file = cachedir.join(format!("{:x}.tex", hash));
            assert!(write(&temp_file, &compile_string).is_ok());
            let result = compiler.run(&temp_file.to_string_lossy(), &input).unwrap();
            assert!(write(&output, &result).is_ok());
            //result
        };

        //let document = Document::load_from(&pdf_data[..]).unwrap();
        //let pages = document.get_pages();
        //println!("{} pages", pages.iter().len());
        //// copy the file into the working directory
        debug!("{}", &output.to_string_lossy());
        generated_documents.insert(hash, output.to_owned());
        //command_args = command_args + " " + output.to_str().unwrap();
        command = command.arg(output.to_str().unwrap());
    }

    let output = command
        .arg(matches.value_of("OUTPUT").unwrap_or("output.pdf"))
        .output()
        .expect("failed to execute process");

    //println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    eprint!("{}", String::from_utf8_lossy(&output.stdout));

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
