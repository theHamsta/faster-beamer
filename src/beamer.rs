//
// beamer.rs
// Copyright (C) 2019 seitz_local <seitz_local@lmeXX>
// Distributed under terms of the GPLv3 license.
//

use crate::parsing::ParsedFile;
use crate::tree_traversal::get_children;
use tree_sitter::Node;

pub fn get_frames(parsed_file: &ParsedFile) -> Vec<Node> {
    let mut frames = Vec::new();

    let text_envs = parsed_file.get_nodes_of_type("text_env".to_string());

    for t in text_envs {
        let children = get_children(t, &|n| has_begin_frame(n, parsed_file), true);
        if children.len() == 1 {
            frames.push(t)
        }
    }
    frames
}

fn has_begin_frame(node: Node, parsed_file: &ParsedFile) -> bool {
    node.kind() == "begin"
        && parsed_file
            .get_node_string(&node)
            .to_string()
            .contains("{frame}")
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
