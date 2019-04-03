//
// build.rs
// Copyright (C) 2019 stephan <stephan@stephan-ThinkPad-X300>
// Distributed under terms of the MIT license.
//
extern crate cc;

fn main() {

    cc::Build::new()
        .file("tree-sitter-latex/src/parser.c")
        .include("tree-sitter-cpp/src")
        .flag_if_supported("-w") // disable warnings
        .compile("tree-sitter-cpp");

    cc::Build::new()
        .file("tree-sitter-latex/src/scanner.cc")
        .cpp(true)
        .flag_if_supported("-w") // disable warnings
        .include("tree-sitter-cpp/src")
        .compile("tree-sitter-cpp-scanner");
}
