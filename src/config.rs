use std::collections::HashMap;
use std::path::Path;

use cargo_metadata;
use custom_error::custom_error;
use serde::de::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use serde_json;
use std::ops::Deref;

trait OptionDeref<T: Deref> {
    fn as_deref(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionDeref<T> for Option<T> {
    fn as_deref(&self) -> Option<&T::Target> {
        self.as_ref().map(Deref::deref)
    }
}

#[derive(Debug)]
pub(crate) struct MatrixEntry {
    pub(crate) run: bool,
    pub(crate) version: String,

    // TODO: this needs to be shell-escaped!
    pub(crate) install_commandline: Option<String>,

    pub(crate) commandline: String,
}

pub(crate) trait MatrixEntryExt {
    fn the_entry<'a>(&'a self) -> &'a MatrixEntry;

    fn run(&self) -> bool {
        self.the_entry().run
    }

    fn version(&self) -> &str {
        &(self.the_entry().version)
    }

    fn install_commandline(&self) -> Option<&str> {
        self.the_entry().install_commandline.as_deref()
    }

    fn commandline(&self) -> &str {
        &(self.the_entry().commandline)
    }
}

define_matrix_entry!(
    BenchEntry,
    (false, "nightly", None, "cargo bench".to_owned())
);
define_matrix_entry!(
    ClippyEntry,
    (
        true,
        "stable",
        "rustup component add clippy".to_owned(),
        "cargo clippy -- -D warnings".to_owned()
    )
);
define_matrix_entry!(
    RustfmtEntry,
    (
        true,
        "stable",
        "rustup component add rustfmt".to_owned(),
        "cargo fmt -v -- --check".to_owned()
    )
);

define_matrix_entry!(CustomEntry, (false, "stable", None, None));

#[derive(Debug, Deserialize)]
pub(crate) struct TemplateCIConfig {
    #[serde(default)]
    pub(crate) bench: BenchEntry,

    #[serde(default)]
    pub(crate) clippy: ClippyEntry,

    #[serde(default)]
    pub(crate) rustfmt: RustfmtEntry,

    #[serde(default)]
    pub(crate) additional_matrix_entries: HashMap<String, CustomEntry>,

    #[serde(default = "TemplateCIConfig::default_cache")]
    pub(crate) cache: String,

    #[serde(default = "TemplateCIConfig::default_os")]
    pub(crate) os: String,

    #[serde(default = "TemplateCIConfig::default_dist")]
    pub(crate) dist: String,

    #[serde(default = "TemplateCIConfig::default_versions")]
    pub(crate) versions: Vec<String>,

    #[serde(default = "TemplateCIConfig::default_test_commandline")]
    pub(crate) test_commandline: String,
}

impl Default for TemplateCIConfig {
    fn default() -> Self {
        TemplateCIConfig {
            clippy: Default::default(),
            bench: Default::default(),
            rustfmt: Default::default(),
            additional_matrix_entries: Default::default(),
            dist: "xenial".to_string(),
            cache: "cargo".to_string(),
            os: "linux".to_string(),
            versions: vec!["stable", "nightly"]
                .into_iter()
                .map(String::from)
                .collect(),
            test_commandline: "cargo test --verbose --all".to_owned(),
        }
    }
}

impl<'a> TemplateCIConfig {
    pub(crate) fn from_manifest(path: Option<&Path>) -> Result<TemplateCIConfig, Error> {
        #[derive(Debug, Deserialize)]
        struct Metadata {
            #[serde(default)]
            template_ci: Option<TemplateCIConfig>,
        }
        let metadata = cargo_metadata::metadata(path)?;
        match &metadata.packages[0].metadata {
            serde_json::Value::Null => Ok(Default::default()),
            md => {
                let metadata_str = md.to_string();
                let config: Metadata = serde_json::from_str(&metadata_str)?;
                Ok(config.template_ci.unwrap_or_default())
            }
        }
    }

    fn default_cache() -> String {
        Self::default().cache
    }

    fn default_os() -> String {
        Self::default().os
    }

    fn default_dist() -> String {
        Self::default().dist
    }

    fn default_versions() -> Vec<String> {
        Self::default().versions
    }

    fn default_test_commandline() -> String {
        Self::default().test_commandline
    }
}

custom_error! {pub Error
               CargoError{source: cargo_metadata::Error} = "Could not get cargo metadata",
               Deserialization{source: serde_json::Error} = "Could not parse cargo metadata",

}

#[cfg(test)]
mod tests {
    use custom_error::*;
    use std::fs::File;
    use std::io;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile;

    use super::TemplateCIConfig;

    custom_error! {Error
                   ConfigError{source: super::Error} = "configuration error",
                   IO{source: io::Error} = "IO",
                   Tempfile{source: tempfile::PersistError} = "Test setup/teardown",
    }

    fn create_cargo_file(dir: &tempfile::TempDir, extra_content: &str) -> Result<PathBuf, Error> {
        let path = dir.path().join("Cargo.toml");
        let f = File::create(&path)?;
        writeln!(
            &f,
            r#"
[package]
name = "testing"
version = "0.0.1"
[lib]
name = "foo"
path = "/dev/null"
"#
        )?;
        writeln!(&f, "{}", extra_content)?;
        Ok(path)
    }

    #[test]
    fn parses_cargo_without_metadata() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        {
            let f = create_cargo_file(&dir, "")?;
            let _conf = TemplateCIConfig::from_manifest(Some(&f))?;
        }
        Ok(())
    }

    #[test]
    fn parses_cargo_without_matching_metadata() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        {
            let f = create_cargo_file(
                &dir,
                r#"
[package.metadata.foo]
bar = "baz"
"#,
            )?;
            let _conf = TemplateCIConfig::from_manifest(Some(&f))?;
        }
        Ok(())
    }

    #[test]
    fn parses_cargo_with_custom_entry() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        {
            let f = create_cargo_file(
                &dir,
                r#"
[package.metadata.template_ci.additional_matrix_entries.something_custom]
name = "custom_templated_run"
install_commandline='echo "installing for custom tests"'
commandline='echo "running custom tests"'
"#,
            )?;
            let _conf = TemplateCIConfig::from_manifest(Some(&f))?;
        }
        Ok(())
    }

    #[test]
    fn parses_cargo_customizing_stuff() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        {
            let f = create_cargo_file(
                &dir,
                r#"
[package.metadata.template_ci]
os = "foo"
"#,
            )?;
            let conf = TemplateCIConfig::from_manifest(Some(&f))?;
            assert_eq!(conf.os, "foo");
            assert_eq!(conf.dist, TemplateCIConfig::default().dist);
        }
        Ok(())
    }
}