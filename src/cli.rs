use anyhow::anyhow;
use mdbook::preprocess::Preprocessor;
use semver::{Version, VersionReq};
use std::fs;

/// Handle the install command to add preprocessor config to book.toml
pub fn handle_install() -> anyhow::Result<()> {
    let mut config_path = std::env::current_dir()?;
    config_path.push("book.toml");

    if !config_path.exists() {
        return Err(anyhow!(
            "book.toml not found in the current directory. \
             Make sure you are in the root of your mdbook project."
        ));
    }

    let mut config_str = fs::read_to_string(&config_path)?;
    if !config_str.contains("[preprocessor.jupyter]") {
        config_str.push_str("\n[preprocessor.jupyter]\n");
        fs::write(&config_path, config_str)?;
        println!("Added [preprocessor.jupyter] to book.toml");
    } else {
        println!("[preprocessor.jupyter] already exists in book.toml");
    }

    Ok(())
}

/// Check version compatibility with mdbook
pub fn check_version_compatibility(mdbook_version: &str) -> Result<(), String> {
    let version_req = VersionReq::parse(&format!("^{}", mdbook::MDBOOK_VERSION))
        .expect("MDBOOK_VERSION is a valid version requirement");
    let version = Version::parse(mdbook_version)
        .expect("mdbook_version is a valid version string");

    if !version_req.matches(&version) {
        eprintln!(
            "Warning: The jupyter preprocessor was built against mdbook version {}, \
             but we're being called from version {}",
            mdbook::MDBOOK_VERSION,
            mdbook_version
        );
    }

    Ok(())
}

/// Handle the supports command
pub fn handle_supports<P: Preprocessor>(preprocessor: &P, renderer: &str) -> bool {
    preprocessor.supports_renderer(renderer)
}
