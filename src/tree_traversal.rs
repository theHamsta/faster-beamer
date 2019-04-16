//
// tree_traversal.rs
// Copyright (C) 2019 stephan <stephan@stephan-ThinkPad-X300>
// Distributed under terms of the MIT license.
//

use tree_sitter::Node;

pub enum TraversalOrder {
    BreadthFirst,
    DepthFirst,
}

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
    traversal_order: TraversalOrder,
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

        match traversal_order {
            TraversalOrder::DepthFirst => {
                for i in (0..num_children).rev() {
                    stack.push(current_node.named_child(i).unwrap())
                }
            }
            TraversalOrder::BreadthFirst => {
                for i in 0..num_children {
                    stack.push(current_node.named_child(i).unwrap())
                }
            }
        };
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
                results.push(sibling);
                if return_first_only {
                    return results;
                }
            }
            maybe_sibling = sibling.prev_sibling();
        }

        maybe_parent = current_node.parent()
    }
    results
}

#[cfg(test)]
mod tests {
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
        parsed.syntax_tree.root_node();
    }
}
