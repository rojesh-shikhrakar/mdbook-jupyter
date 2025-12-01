# mdbook-jupyter

A [mdbook](https://github.com/rust-lang-nursery/mdBook) preprocessor that converts Jupyter notebooks (`.ipynb`) to markdown chapters.

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
``
