//
// build.rs
// Copyright (C) 2019 stephan <stephan@stephan-ThinkPad-X300>
// Distributed under terms of the MIT license.
//
extern crate cc;

fn main() {
    #[cfg(not(feature = "tree-sitter-parsing"))]
    cc::Build::new()
        .include("tree-sitter-latex/src")
        .file("tree-sitter-latex/src/parser.c")
        .flag_if_supported("-w") // disable warnings
        .compile("tree-sitter-latex");

    #[cfg(not(feature = "tree-sitter-parsing"))]
    cc::Build::new()
        .file("tree-sitter-latex/src/scanner.cc")
        .file("tree-sitter-latex/src/catcode.cc")
        .file("tree-sitter-latex/src/scanner_control_sequences.cc")
        .file("tree-sitter-latex/src/scanner_environments.cc")
        .file("tree-sitter-latex/src/scanner_keywords.cc")
        .file("tree-sitter-latex/src/scanner_names.cc")
        .flag_if_supported("-w") // disable warnings
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .include("tree-sitter-latex/src")
        .include("tree-sitter-latex/src/tree_sitter")
        .compile("tree-sitter-latex-scanner");
}
