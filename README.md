# faster-beamer

An incremental compiler for LaTeX Beamer slides

## Motivation

Compiling Beamer slides takes to long.
I wanted to have a fast preview of my files even if the output is not 100% correct.

## What it does

It parses your input file and compiles each `frame` enviroment individually and in parallel.
Compiled frames are cached and only recompiled if necessary.  
Of course, frame pages and citation will not be rendered correctly, but it should be sufficient to get an idea
how your frames will look like.

Executing the following line will let `faster-beamer` watch your tex-file for changes, compile all frames on changes and only output
the frame that was changed most recently.

```bash
faster-beamer presentation.tex --server
```

If you want pdfunite to glue all the compiled frames together use:

```bash
faster-beamer presentation.tex --server --unite
```

## Requirements

You need to have `pdflatex` in `PATH`. Addidionally, also `pdfunite` if you want to unite PDFs.

## Installation

```bash
cargo install --path . --force
```

## Thanks

A modified version of `https://github.com/santifa/latexcompile` is used in this project.
