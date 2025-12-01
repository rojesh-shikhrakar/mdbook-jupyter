pub mod converter;
pub mod cli;

use mdbook::book::{Book, BookItem};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use crate::converter::{convert_notebook_to_md_with_options, ConvertOptions};

/// Jupyter preprocessor for mdbook
pub struct JupyterPreprocessor;

impl JupyterPreprocessor {
    pub fn new() -> Self {
        JupyterPreprocessor
    }
}

impl Default for JupyterPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for JupyterPreprocessor {
    fn name(&self) -> &str {
        "jupyter"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        eprintln!("Running Jupyter preprocessor");
        let assets_dir = ctx
            .root
            .join(&ctx.config.build.build_dir)
            .join("html/assets");

        // Extract configuration from the preprocessor config
        let options = ctx.config.get_preprocessor(self.name())
            .and_then(|cfg| {
                // Try to extract embed_images boolean from config table
                cfg.get("embed_images")
                    .and_then(|v| v.as_bool())
                    .map(|embed_images| ConvertOptions { embed_images })
            })
            .unwrap_or_default();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if let Some(path) = &chapter.path {
                    if path.extension().map_or(false, |ext| ext == "ipynb") {
                        let full_path = ctx.root.join(&ctx.config.book.src).join(path);
                        match convert_notebook_to_md_with_options(&full_path, &assets_dir, options.clone()) {
                            Ok(content) => chapter.content = content,
                            Err(e) => {
                                // Log the error to stderr so the mdbook user sees the underlying cause
                                eprintln!("Error converting notebook '{}': {}", path.display(), e);

                                // Inject a visible error message into the generated chapter content
                                // so the book shows an informative placeholder rather than an empty page.
                                chapter.content = format!(
                                    "<!-- mdbook-jupyter: conversion error -->\n\n> **Notebook conversion failed** for `{}`\n\n```
{}\n```
\n\nPlease check the original notebook and converter logs for details.",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html" || renderer == "markdown"
    }
}
