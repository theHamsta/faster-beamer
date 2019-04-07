//
// parsing.rs
// Copyright (C) 2019 stephan <stephan@stephan-ThinkPad-X300>
// Distributed under terms of the MIT license.
//

use crate::tree_traversal::{get_children, get_nodes_of_type, TraversalOrder};
use std::fs;
use tree_sitter::{InputEdit, Language, Node, Parser};

extern "C" {
    fn tree_sitter_latex() -> Language;
}

pub struct ParsedFile {
    pub filename: String,
    pub file_content: String,
    pub syntax_tree: tree_sitter::Tree,
    parser: Parser,
}

impl ParsedFile {
    pub fn new(filename: String) -> ParsedFile {
        let file_content = fs::read_to_string(&filename).expect("Failed to read file");
        ParsedFile::from_string(filename, file_content)
    }

    pub fn from_string(filename: String, file_content: String) -> ParsedFile {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_latex() };

        parser.set_language(language).unwrap();

        let tree = parser
            .parse(&file_content, None)
            .expect("Failed to parse file");
        ParsedFile {
            filename,
            file_content,
            syntax_tree: tree,
            parser,
        }
    }

    pub fn get_first_node_of_type(&self, node_type: String) -> Option<Node> {
        let root_node = self.syntax_tree.root_node();
        let result = get_nodes_of_type(root_node, node_type, false);
        match result.len() {
            0 => Some(result[0]),
            1 => None,
            _ => panic!("Should not happen!"),
        }
    }

    pub fn get_nodes_of_type(&self, node_type: String) -> Vec<Node> {
        let root_node = self.syntax_tree.root_node();
        get_nodes_of_type(root_node, node_type, false)
    }

    pub fn get_nodes_from_source_text(
        &self,
        source_text: &str,
        return_first_only: bool,
    ) -> Vec<Node> {
        get_children(
            self.syntax_tree.root_node(),
            &|n: Node| self.get_node_string(&n) == source_text,
            return_first_only,
            TraversalOrder::DepthFirst,
        )
    }
    pub fn get_child_nodes_from_source_text<'a>(
        &self,
        root_node: Node<'a>,
        source_text: &str,
        return_first_only: bool,
    ) -> Vec<Node<'a>> {
        get_children(
            root_node,
            &|n: Node| self.get_node_string(&n) == source_text,
            return_first_only,
            TraversalOrder::DepthFirst
        )
    }

    pub fn edit(&mut self, new_source_code: String, edit: &InputEdit) {
        self.syntax_tree.edit(&edit);
        self.syntax_tree = self
            .parser
            .parse(&new_source_code, Some(&self.syntax_tree))
            .unwrap();
        self.file_content = new_source_code;
    }

    //#[cfg(unix)]
    //pub fn write_dot_graph(mut self, file: &str) -> std::io::Result<()> {
    //let file = OpenOptions::new().write(true).create(true).open(file)?;
    //self.parser.print_dot_graphs(&file);
    //Ok(())
    //}

    pub fn get_node_string(&self, node: &Node) -> &str {
        &self.file_content[node.start_byte()..node.end_byte()]
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::path::Path;

    #[test]
    fn get_node_types() {
        let test_file = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test/test_files/simple_tera_template.cpp");
        let parsed_file = ParsedFile::new(test_file.to_string_lossy().into_owned());
        let node_types = vec![
            "comment",
            "declaration",
            "class_specifier",
            "call_expression",
            "function_definition",
            "field_declaration",
            "ERROR",
        ];
        for t in node_types {
            let comments = parsed_file.get_nodes_of_type(t.to_string());
            println!("");
            println!("Found {} {}s:", comments.len(), t);
            for c in comments {
                let text = &parsed_file.file_content[c.start_byte()..c.end_byte()];
                println!("{}", text);
            }
        }
    }

    #[test]
    fn print_all_nodes() {
        let test_file = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test/test_files/simple_tera_template.cpp");
        let parsed_file = ParsedFile::new(test_file.to_string_lossy().into_owned());

        let root_node = parsed_file.syntax_tree.root_node();
        let mut stack = vec![root_node];

        while !stack.is_empty() {
            let current_node = stack.pop().unwrap();
            println!(
                "\n{}:\n\t {}",
                current_node.kind(),
                parsed_file.get_node_string(&current_node)
            );

            for i in (0..current_node.named_child_count()).rev() {
                stack.push(current_node.named_child(i).unwrap());
            }
        }
    }

    #[test]
    fn print_nodes_of_language() {
        let languages = vec![unsafe { tree_sitter_latex() }];

        for l in languages {
            for i in 0..l.node_kind_count() {
                println!("{}", l.node_kind_for_id(i as u16));
            }
        }
    }
}
