//
// tree_traversal.rs
// Copyright (C) 2019 stephan <stephan@stephan-ThinkPad-X300>
// Distributed under terms of the MIT license.
//

use crate::parsing::ParsedFile;
use std::boxed::Box;
use tree_sitter::Node;

pub fn get_nodes_of_type(root_node: Node, node_type: String, return_first_only: bool) -> Vec<Node> {
    let mut stack = vec![root_node];
    let mut results = Vec::new();

    while !stack.is_empty() {
        let current_node = stack.pop().unwrap();

        if current_node.kind() == &node_type[..] {
            results.push(current_node);
            if return_first_only {
                return results;
            }
        }

        let num_children = current_node.named_child_count();

        for i in (0..num_children).rev() {
            stack.push(current_node.named_child(i).unwrap());
        }
    }
    results
}

pub fn get_children<'a, 'b>(
    root_node: Node<'a>,
    predicate: &Fn(Node<'a>) -> bool,
    return_first_only: bool,
) -> Vec<Node<'a>> {
    let mut stack = vec![root_node];
    let mut results = Vec::new();

    while !stack.is_empty() {
        let current_node = stack.pop().unwrap();

        if predicate(current_node) {
            results.push(current_node);
            if return_first_only {
                return results;
            }
        }

        let num_children = current_node.named_child_count();

        for i in (0..num_children).rev() {
            stack.push(current_node.named_child(i).unwrap());
        }
    }
    results
}

pub fn get_scope_nodes<'a, 'b>(
    current_node: Node<'a>,
    predicate: &Fn(Node<'a>) -> bool,
    return_first_only: bool,
) -> Vec<Node<'a>> {
    let mut results = Vec::new();
    let mut maybe_parent = Some(current_node);

    while maybe_parent.is_some() {
        let current_node = maybe_parent.unwrap();
        let mut maybe_sibling = Some(current_node);
        while maybe_sibling.is_some() {
            let sibling = maybe_sibling.unwrap();
            if predicate(sibling) {
                results.push(sibling)
            }
            maybe_sibling = sibling.prev_sibling();
        }

        maybe_parent = current_node.parent()
    }
    results
}

//pub fn get_node_from_text<'a, 'b>(
//file: &'a ParsedFile,
//parent_node: Node<'a>,
//node_source_text: String,
//return_first_only: bool,
//) -> Vec<Node<'a>> {
//get_children(
//parent_node,
//&|node: Node| file.get_node_string(node).to_string() == node_source_text,
//return_first_only,
//)
//}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::ParsedFile;

    #[test]
    fn test_get_scope_nodes() {
        let source_code = r#" 
#include <iostream>

int main() 
{
std::cout << "Hello, World!";
return 0;
}
    "#;
        let parsed = ParsedFile::from_string("main.c".to_string(), source_code.to_string());
        let root_node = parsed.syntax_tree.root_node();
    }
    #[test]
    fn test_get_node() {}
}
