# mdbook-jupyter

A [mdbook](https://github.com/rust-lang-nursery/mdBook) preprocessor that converts Jupyter notebooks (`.ipynb`) to markdown chapters.

[![Latest version](https://img.shields.io/crates/v/mdbook-jupyter)](https://crates.io/crates/mdbook-jupyter)
![Deps.rs Crate Dependencies (latest)](https://img.shields.io/deps-rs/mdbook-jupyter/latest)
[![Documentation](https://docs.rs/mdbook-jupyter/badge.svg)](https://docs.rs/mdbook-jupyter)
![GitHub License](https://img.shields.io/github/license/rojesh-shikhrakar/mdbook-jupyter)

## Installation

```bash
cargo install mdbook-jupyter
mdbook-jupyter install
```

The `install` command adds `[preprocessor.jupyter]` to your `book.toml`.

## Usage

Add `.ipynb` files to your book and reference them in `SUMMARY.md`:

```Markdown
- [My Notebook](path/to/notebook.ipynb)
```

Configure in `book.toml` under `[preprocessor.jupyter]`:
```toml
[preprocessor.jupyter]
embed_images = true
```
