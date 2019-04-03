//
// process_file.rs
// Copyright (C) 2019 seitz_local <seitz_local@lmeXX>
// Distributed under terms of the GPLv3 license.
//
use crate::parsing;
use crate::tree_traversal;
use std::fs;
use std::path::Path;
use tree_sitter::Node;

pub fn process_file(input_file: &str, output_file: &str) {
    if !Path::new(&input_file).exists() {
        eprintln!("Could not open {}", input_file);
        return;
    }

    let parsed_file = parsing::ParsedFile::new(input_file.to_string());
    println!("{}", parsed_file.syntax_tree.root_node().to_sexp());

    let root_node = parsed_file.syntax_tree.root_node();
    let mut stack = vec![root_node];

    while !stack.is_empty() {
        let current_node = stack.pop().unwrap();
        if current_node.kind() != "ERROR" {
            println!(
                "\n{}:\n\t {}",
                current_node.kind(),
                parsed_file.get_node_string(&current_node)
            );
        }

        for i in (0..current_node.named_child_count()).rev() {
            stack.push(current_node.named_child(i).unwrap());
        }
    }

    let node_types = vec!["text_env", "group"];
    for t in node_types {
        let comments = parsed_file.get_nodes_of_type(t.to_string());
        println!("");
        println!("Found {} {}s:", comments.len(), t);
        for c in comments {
            let text = &parsed_file.file_content[c.start_byte()..c.end_byte()];
            println!("{}", text);
        }
    }

    let node_types = vec!["text_env"];
    let mut frames = Vec::new();
    for t in node_types {
        let comments = parsed_file.get_nodes_of_type(t.to_string());
        println!("");
        println!("Found {} {}s:", comments.len(), t);
        for c in comments {
            let children = tree_traversal::get_children(
                c,
                &|n: Node| {
                    n.kind() == "begin"
                        && parsed_file
                            .get_node_string(&n)
                            .to_string()
                            .contains("{frame}")
                },
                true,
            );
            if children.len() == 1 {
                println!("");
                println!("{}", parsed_file.get_node_string(&c));
                frames.push(c);
            }
        }
        println!("Found {} frames", frames.len());
    }
    fs::write(output_file, parsed_file.file_content).expect("Unable to write output file");
}
