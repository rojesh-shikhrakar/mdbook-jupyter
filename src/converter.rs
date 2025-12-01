use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs::{File, create_dir_all};
use std::path::Path;
use std::fs;

/// Configuration options for notebook conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOptions {
    /// If true, embed images as base64 in the markdown instead of saving to files
    #[serde(default)]
    pub embed_images: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        ConvertOptions {
            embed_images: false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Notebook {
    pub cells: Vec<Cell>,
    // other fields (metadata, nbformat, nbformat_minor) are ignored for now
}
#[derive(Debug, Deserialize)]
#[serde(tag = "cell_type")]
pub enum Cell {
    #[serde(rename = "markdown")]
    Markdown {
        source: MultilineString,
        metadata: Option<Value>,
    },

    #[serde(rename = "code")]
    Code {
        source: MultilineString,
        outputs: Vec<Output>,
        execution_count: Option<u32>,
        metadata: Option<Value>,
    },

    #[serde(rename = "raw")]
    Raw {
        source: MultilineString,
        metadata: Option<Value>,
    },
}

/// MultilineString captures the fact that many fields in nbformat
/// may be a single string or an array of strings.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MultilineString {
    Single(String),
    Multi(Vec<String>),
}

impl MultilineString {
    fn into_string(self) -> String {
        match self {
            MultilineString::Single(s) => s,
            MultilineString::Multi(v) => v.join("")
        }
    }

    // fn as_string(&self) -> String {
    //     match self {
    //         MultilineString::Single(s) => s.clone(),
    //         MultilineString::Multi(v) => v.join("")
    //     }
    // }

    fn len(&self) -> usize {
        match self {
            MultilineString::Single(s) => s.len(),
            MultilineString::Multi(v) => v.iter().map(|s| s.len()).sum()
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "output_type")]
pub enum Output {
    #[serde(rename = "stream")]
    Stream { name: Option<String>, text: MultilineString },

    #[serde(rename = "display_data")]
    DisplayData { data: Map<String, Value>, metadata: Option<Value> },

    #[serde(rename = "execute_result")]
    ExecuteResult { data: Map<String, Value>, metadata: Option<Value>, execution_count: Option<u32> },

    #[serde(rename = "error")]
    Error { ename: String, evalue: String, traceback: MultilineString },
}

/// Converts a Jupyter notebook to Markdown format
pub fn convert_notebook_to_md(path: &Path, assets_out: &Path) -> Result<String> {
    let options = ConvertOptions::default();
    convert_notebook_to_md_with_options(path, assets_out, options)
}

/// Converts a Jupyter notebook to Markdown format with custom options
pub fn convert_notebook_to_md_with_options(path: &Path, assets_out: &Path, options: ConvertOptions) -> Result<String> {
    let file = File::open(path)?;
    let notebook: Notebook = serde_json::from_reader(file)?;

    // Ensure assets directory exists (only needed if not embedding images)
    if !options.embed_images {
        if let Err(e) = create_dir_all(assets_out) {
            // If we cannot create the assets directory, return an error
            return Err(anyhow::anyhow!(e));
        }
    }

    // Pre-reserve reasonable capacity to reduce reallocations
    let est: usize = notebook
        .cells
        .iter()
        .map(|c| estimate_cell_len(c))
        .sum();

    let mut md = String::with_capacity(est);

    // counter for generating unique asset filenames
    let mut asset_counter: u32 = 0;

    for cell in notebook.cells.into_iter() {
        process_cell(&mut md, cell, assets_out, &mut asset_counter, &options)?;
    }

    Ok(md)
}

fn estimate_cell_len(cell: &Cell) -> usize {
    match cell {
        Cell::Markdown { source, .. } => source.len() + 4,
        Cell::Raw { source, .. } => source.len() + 4,
        Cell::Code { source, outputs, .. } => {
            let src_len = source.len() + 12; // fenced code block overhead
            let outputs_len: usize = outputs.iter().map(|o| estimate_output_len(o)).sum();
            src_len + outputs_len
        }
    }
}

fn estimate_output_len(output: &Output) -> usize {
    match output {
        Output::Stream { text, .. } => text.len() + 8,
        Output::DisplayData { data, .. } | Output::ExecuteResult { data, .. } => {
            // Pick the first textual value we might include (handle arrays/objects)
            if let Some(s) = data.get("text/markdown").and_then(|v| value_to_text(v)) {
                s.len() + 4
            } else if let Some(s) = data.get("text/plain").and_then(|v| value_to_text(v)) {
                s.len() + 8
            } else if let Some(s) = data.get("image/png").and_then(|v| value_to_text(v)) {
                s.len() + 32
            } else {
                16
            }
        }
        Output::Error { traceback, .. } => traceback.len() + 16,
    }
}

fn value_to_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Array(arr) => {
            let mut out = String::new();
            for v in arr.iter() {
                if let Some(s) = value_to_text(v) {
                    out.push_str(&s);
                }
            }
            Some(out)
        }
        Value::Number(n) => Some(n.to_string()),
        Value::Object(o) => serde_json::to_string(o).ok(),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
    }
}

fn process_cell(md: &mut String, cell: Cell, assets_out: &Path, counter: &mut u32, options: &ConvertOptions) -> Result<(), anyhow::Error> {
    match cell {
        Cell::Markdown { source, .. } => {
            md.push_str(&source.into_string());
            md.push_str("\n\n");
        }
        Cell::Code { source, outputs, .. } => {
            md.push_str("```python\n");
            md.push_str(&source.into_string());
            md.push_str("\n```\n\n");

            for output in outputs.into_iter() {
                process_output(md, output, assets_out, counter, options)?;
            }
        }
        Cell::Raw { source, .. } => {
            md.push_str(&source.into_string());
            md.push_str("\n\n");
        }
    }

    Ok(())
}

fn process_output(md: &mut String, output: Output, assets_out: &Path, counter: &mut u32, options: &ConvertOptions) -> Result<(), anyhow::Error> {
    match output {
        Output::Stream { text, .. } => {
            md.push_str("```\n");
            md.push_str(&text.into_string());
            md.push_str("\n```\n\n");
        }
        Output::DisplayData { data, .. } | Output::ExecuteResult { data, .. } => {
            // Handle common image types first; values may be strings or arrays of strings
            if let Some(img_b64) = data.get("image/png").and_then(|v| value_to_text(v)) {
                if options.embed_images {
                    // Embed image as base64 data URL
                    md.push_str(&format!("![output image](data:image/png;base64,{})\n\n", img_b64));
                } else {
                    // decode and write to file
                    let decoded = STANDARD.decode(&img_b64)?;
                    let filename = format!("output_{:03}.png", *counter);
                    let out_path = assets_out.join(&filename);
                    fs::write(&out_path, &decoded)?;
                    *counter += 1;

                    if let Some(dirname) = assets_out.file_name().map(|s| s.to_string_lossy()) {
                        md.push_str(&format!("![output image]({}/{})\n\n", dirname, filename));
                    } else {
                        md.push_str(&format!("![output image]({})\n\n", filename));
                    }
                }
            } else if let Some(img_b64) = data.get("image/jpeg").and_then(|v| value_to_text(v)) {
                if options.embed_images {
                    // Embed image as base64 data URL
                    md.push_str(&format!("![output image](data:image/jpeg;base64,{})\n\n", img_b64));
                } else {
                    let decoded = STANDARD.decode(&img_b64)?;
                    let filename = format!("output_{:03}.jpg", *counter);
                    let out_path = assets_out.join(&filename);
                    fs::write(&out_path, &decoded)?;
                    *counter += 1;

                    if let Some(dirname) = assets_out.file_name().map(|s| s.to_string_lossy()) {
                        md.push_str(&format!("![output image]({}/{})\n\n", dirname, filename));
                    } else {
                        md.push_str(&format!("![output image]({})\n\n", filename));
                    }
                }
            } else if let Some(svg) = data.get("image/svg+xml").and_then(|v| value_to_text(v)) {
                if options.embed_images {
                    // Embed SVG as base64 data URL
                    let svg_b64 = STANDARD.encode(&svg);
                    md.push_str(&format!("![output svg](data:image/svg+xml;base64,{})\n\n", svg_b64));
                } else {
                    let filename = format!("output_{:03}.svg", *counter);
                    let out_path = assets_out.join(&filename);
                    fs::write(&out_path, svg.as_bytes())?;
                    *counter += 1;

                    if let Some(dirname) = assets_out.file_name().map(|s| s.to_string_lossy()) {
                        md.push_str(&format!("![output svg]({}/{})\n\n", dirname, filename));
                    } else {
                        md.push_str(&format!("![output svg]({})\n\n", filename));
                    }
                }
            } else if let Some(mdtext) = data.get("text/markdown").and_then(|v| value_to_text(v)) {
                md.push_str(&mdtext);
                md.push_str("\n\n");
            } else if let Some(text) = data.get("text/plain").and_then(|v| value_to_text(v)) {
                md.push_str("```\n");
                md.push_str(&text);
                md.push_str("\n```\n\n");
            } else if let Some(html) = data.get("text/html").and_then(|v| value_to_text(v)) {
                md.push_str("```html\n");
                md.push_str(&html);
                md.push_str("\n```\n\n");
            }
        }
        Output::Error { ename, evalue, traceback } => {
            md.push_str("```error\n");
            md.push_str(&ename);
            md.push_str(": ");
            md.push_str(&evalue);
            md.push_str("\n");
            md.push_str(&traceback.into_string());
            md.push_str("\n```\n\n");
        }
    }

    Ok(())
}
    
