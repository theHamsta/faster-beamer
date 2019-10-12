//
// process_file.rs
// Copyright (C) 2019 seitz_local <seitz_local@lmeXX>
// Distributed under terms of the GPLv3 license.
//
use crate::beamer::get_frames;
use crate::parsing;

use log::Level::Trace;

use cachedir::CacheDirConfig;
use clap::ArgMatches;
use indicatif::ProgressBar;
use latexcompile_fasterbeamer::{LatexCompiler, LatexInput, LatexRunOptions};
use md5;
use rayon;
use rayon::prelude::*;
use regex::Regex;
use std::env::current_dir;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::sync::Mutex;
use std::vec::Vec;

#[derive(PartialEq)]
pub enum FasterBeamerError {
    InputFileNotExistent,
    CompileError,
    PdfUniteError,
}

pub type Result<T> = ::std::result::Result<T, FasterBeamerError>;

lazy_static! {
    static ref FRAME_REGEX: Regex =
        Regex::new(r"(?ms)^\\begin\{frame\}.*?^\\end\{frame\}").unwrap();
}
lazy_static! {
    static ref DOCUMENT_REGEX: Regex =
        Regex::new(r"(?ms)^\\begin\{document\}.*^\\end\{document\}").unwrap();
}

lazy_static! {
    static ref PREVIOUS_FRAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn process_file(input_file: &str, args: &ArgMatches) -> Result<()> {
    let cwd = current_dir().unwrap();
    let input_path = Path::new(&input_file);
    let input_dir = input_path.parent().unwrap_or(&cwd);
    if !input_path.is_file() {
        error!("Could not open {}", input_file);
        return Err(FasterBeamerError::InputFileNotExistent);
    }

    let parsed_file = parsing::ParsedFile::new(input_file.to_string());
    trace!("{}", parsed_file.syntax_tree.root_node().to_sexp());

    let frame_nodes = if args.is_present("tree-sitter") {
        get_frames(&parsed_file)
    } else {
        Vec::new()
    };

    let mut frames = Vec::with_capacity(frame_nodes.len());
    if !frame_nodes.is_empty() {
        for f in frame_nodes.iter() {
            info!("Found {} frames with tree-sitter.", frame_nodes.len());
            let node_string = parsed_file.get_node_string(&f);
            frames.push(node_string.to_string());
        }
    } else {
        for cap in FRAME_REGEX.captures_iter(&parsed_file.file_content) {
            let frame_string = cap[0].to_string();
            frames.push(frame_string);
        }
    }
    info!("Found {} frames.", frames.len());

    if log_enabled!(Trace) && args.is_present("tree-sitter") {
        let root_node = parsed_file.syntax_tree.root_node();
        let mut stack = vec![root_node];

        while !stack.is_empty() {
            let current_node = stack.pop().unwrap();
            if current_node.kind() == "ERROR" {
                error!(
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

    //let document_env = tree_traversal::get_children(
    //parsed_file.syntax_tree.root_node(),
    //&|n| n.kind() == "document_env",
    //true,
    //TraversalOrder::BreadthFirst,
    //);
    //let preamble =[> if document_env.len() == 1 as usize {<]
    //parsed_file.file_content[0..document_env[0].start_byte()].to_owned()
    //} else {
    //warn!(
    //"Could not find document environment with tree_sitter ({})",
    //input_file
    /*);*/
    let find = parsed_file.file_content.find("\\begin{document}");
    let preamble = match find {
        Some(x) => Some(parsed_file.file_content[..x].to_owned()),
        None => None,
    }
    .unwrap_or(r"\documentclass[aspectratio=43,c,xcolor=dvipsnames]{beamer}".to_string());

    let cachedir: PathBuf = CacheDirConfig::new("faster-beamer")
        .get_cache_dir()
        .unwrap()
        .into();
    let cache_subdir = cachedir.join(format!(
        "./{}",
        &input_dir.canonicalize().unwrap().to_str().unwrap()
    ));

    let preamble_hash = md5::compute(&preamble);
    let preamble_filename = format!("{:x}_{}", preamble_hash, args.is_present("draft"));
    if input_path
        .parent()
        .unwrap()
        .join(format!("{}.fmt", preamble_filename))
        .is_file()
    {
        info!("Precompiled preamble already exists");
    } else {
        info!(
            "Precompiling preamble {:?}",
            input_path.join(format!("{}.fmt", preamble_filename))
        );
        let output = Command::new("pdflatex")
            .arg("-shell-escape")
            .arg("-ini")
            .arg(format!("-jobname=\"{}\"", preamble_filename))
            .arg("\"&pdflatex\"")
            .arg("mylatexformat.ltx")
            .arg(&input_file)
            .output();
        match output {
            Err(e) => {
                error!("Failed to compile preamble!\n{}", e);
                return Err(FasterBeamerError::CompileError);
            }
            Ok(output) if !output.status.success() => {
                error!(
                    "Failed to compile preamble! {}",
                    str::from_utf8(&output.stderr).unwrap()
                );
                return Err(FasterBeamerError::CompileError);
            }
            _ => {}
        };
    }

    let mut generated_documents = Vec::new();
    let mut command = &mut Command::new("pdfunite");
    for f in &frames {
        let compile_string = format!("%&{}\n", preamble_filename)
            + &preamble
            + "\n\\begin{document}\n"
            + &f
            + "\n\\end{document}\n";

        let hash = md5::compute(&compile_string);
        let output = cache_subdir.join(format!("{:x}.pdf", hash));
        generated_documents.push((hash, compile_string));

        command = command.arg(output.to_str().unwrap());
    }

    trace!("Comparing frames");
    let mut first_changed_frame = 0;
    for frame_pair in frames.iter().zip((*PREVIOUS_FRAMES.lock().unwrap()).iter()) {
        match frame_pair {
            (lhs, rhs) if lhs != rhs => {
                break;
            }
            _ => first_changed_frame += 1,
        }
    }
    debug!(
        "Found first difference in frame {} from {}",
        &first_changed_frame,
        frames.len()
    );

    let progress_bar = ProgressBar::new(generated_documents.len() as u64);

    generated_documents
        .par_iter()
        .enumerate()
        .for_each(|(frame_idx, (hash, tex_content))| {
            let pdf = cache_subdir.join(format!("{:x}.pdf", hash));

            if pdf.is_file() {
                trace!("{} is already compiled!", pdf.to_str().unwrap_or("???"));
            } else {
                let latex_input = LatexInput::from_lazy(
                    input_dir.canonicalize().unwrap().to_str().unwrap(),
                    &cachedir,
                )
                .expect("Failed to create LatexInput");

                let temp_file = cache_subdir.join(format!("{:x}.tex", hash));

                if write(&temp_file, &tex_content).is_ok() {
                    let mut compiler = LatexCompiler::new()
                        .unwrap()
                        .add_arg("-shell-escape")
                        .add_arg("-interaction=nonstopmode");
                    compiler.working_dir = temp_file
                        .parent()
                        .unwrap()
                        .canonicalize()
                        .unwrap()
                        .to_path_buf();

                    let result = compiler.run(
                        &temp_file.canonicalize().unwrap().to_string_lossy(),
                        &latex_input,
                        LatexRunOptions::new(),
                    );
                    if result.is_ok() {
                        trace!("Compiled file {}", &temp_file.to_str().unwrap());
                    } else {
                        error!(
                            "Failed to compile frame {} ({})",
                            frame_idx,
                            &temp_file.to_str().unwrap()
                        );
                        error!("{}", frames[frame_idx]);
                        error!("{:?}", result.err());
                    };
                }
            };
            progress_bar.inc(1);
        });
    progress_bar.finish_and_clear();

    let output_file = args.value_of("OUTPUT").unwrap_or("output.pdf");

    if args.is_present("unite") {
        info!("PDF unite!");
        let output = command.arg(output_file).output();

        match output {
            Err(e) => {
                error!("Failed to run pdf unite!\n{}", e);
                return Err(FasterBeamerError::PdfUniteError);
            }
            Ok(output) if !output.status.success() => {
                error!(
                    "Failed to run pdfunite! {}",
                    str::from_utf8(&output.stderr).unwrap()
                );
                return Err(FasterBeamerError::PdfUniteError);
            }
            _ => {}
        };
    } else {
        if first_changed_frame < generated_documents.len() {
            let (hash, _) = generated_documents[first_changed_frame];
            let compiled_pdf = cache_subdir.join(format!("{:x}.pdf", hash));

            if Path::new(&output_file).is_file() {
                info!("Linking: {:?} -> {:?}", &compiled_pdf, &output_file);
                let _result = ::std::fs::remove_file(&output_file)
                    .expect("Tried to delete previous output file");
            }
            if Path::new(&compiled_pdf).is_file() {
                ::symlink::symlink_file(compiled_pdf, output_file)
                    .expect("Failed to create symlink to output file.");
            } else {
                error!("Compilation failed!");
                return Err(FasterBeamerError::CompileError);
            }
        }
    }

    *PREVIOUS_FRAMES.lock().unwrap() = frames;
    Ok(())
}
