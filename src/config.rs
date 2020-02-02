use std::collections::HashMap;
use std::env::current_dir;
use std::fs::read_to_string;
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::Duration;

use cargo_metadata;
use custom_error::custom_error;
use serde::de::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use serde_json;
use toml;

trait OptionDeref<T: Deref> {
    fn as_deref_option(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionDeref<T> for Option<T> {
    fn as_deref_option(&self) -> Option<&T::Target> {
        self.as_ref().map(Deref::deref)
    }
}

#[derive(Debug)]
pub(crate) struct MatrixEntry {
    pub(crate) run: bool,
    pub(crate) run_cron: bool,
    pub(crate) version: String,

    // TODO: this needs to be shell-escaped!
    pub(crate) install_commandline: Option<String>,

    pub(crate) commandline: String,

    pub(crate) timeout: Option<Duration>,
}

pub(crate) trait MatrixEntryExt {
    fn the_entry(&'_ self) -> &'_ MatrixEntry;

    fn run(&self) -> bool {
        self.the_entry().run
    }

    fn run_cron(&self) -> bool {
        self.the_entry().run_cron
    }

    fn version(&self) -> &str {
        &(self.the_entry().version)
    }

    fn install_commandline(&self) -> Option<&str> {
        self.the_entry().install_commandline.as_deref_option()
    }

    fn commandline(&self) -> &str {
        &(self.the_entry().commandline)
    }

    fn timeout(&self) -> Option<String> {
        self.the_entry()
            .timeout
            .map(|to| format!("{}s", to.as_secs()))
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

    #[serde(default = "TemplateCIConfig::default_scheduled_test_branches")]
    pub(crate) scheduled_test_branches: Vec<String>,

    #[serde(default = "TemplateCIConfig::default_test_schedule")]
    pub(crate) test_schedule: String,
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
            scheduled_test_branches: vec!["master"].into_iter().map(String::from).collect(),
            test_schedule: "0 0 * * 0".to_string(), // every sunday at 0:00 UTC
        }
    }
}

impl<'a> TemplateCIConfig {
    fn from_manifest(path: Option<&Path>) -> Result<(TemplateCIConfig, PathBuf), Error> {
        #[derive(Debug, Deserialize)]
        struct Metadata {
            #[serde(default)]
            template_ci: Option<TemplateCIConfig>,
        }
        let metadata = cargo_metadata::metadata(path)?;
        let root_dir = match path {
            None => {
                let cargo_path = &metadata.packages[0].manifest_path;
                let p = Path::new(cargo_path);
                p.parent()
                    .unwrap_or_else(|| panic!("The manifest at {:?} should live in a directory", p))
                    .to_path_buf()
            }
            Some(path) => path
                .parent()
                .unwrap_or_else(|| panic!("The given path {:?} should have a parent dir", path))
                .to_path_buf(),
        };
        match &metadata.packages[0].metadata {
            serde_json::Value::Null => Ok((Default::default(), root_dir)),
            md => {
                let metadata_str = md.to_string();
                let config: Metadata = serde_json::from_str(&metadata_str)?;
                Ok((config.template_ci.unwrap_or_default(), root_dir))
            }
        }
    }

    fn from_config_file(
        file_name: impl AsRef<Path>,
        path: Option<&Path>,
    ) -> Result<(TemplateCIConfig, PathBuf), Error> {
        let file_name = file_name.as_ref();

        let default = current_dir()?.join(file_name);
        let path = path.unwrap_or(&default);
        let config_src = read_to_string(path)?;
        let config: TemplateCIConfig = toml::from_str(&config_src)?;
        Ok((
            config,
            path.parent()
                .expect("Impossible: config file has no parent")
                .to_path_buf(),
        ))
    }

    pub(crate) fn merged_configs(
        path: Option<&Path>,
    ) -> Result<(TemplateCIConfig, PathBuf), Error> {
        TemplateCIConfig::from_config_file("template-ci.toml", path)
            .or_else(|_| TemplateCIConfig::from_config_file(".template-ci.toml", path))
            .or_else(|_| TemplateCIConfig::from_manifest(path))
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

    fn default_test_schedule() -> String {
        Self::default().test_schedule
    }

    fn default_scheduled_test_branches() -> Vec<String> {
        Self::default().scheduled_test_branches
    }
}

custom_error! {pub Error
               CargoError{source: cargo_metadata::Error} = "Could not get cargo metadata",
               Deserialization{source: serde_json::Error} = "Could not parse cargo metadata",
               TOMLDeserialization{source: toml::de::Error} = "Could not parse TOML configuration file",
               IO{source: io::Error} = "IO",
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
timeout={secs = 90, nanos = 0}
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
            let (conf, _) = TemplateCIConfig::from_manifest(Some(&f))?;
            assert_eq!(conf.os, "foo");
            assert_eq!(conf.dist, TemplateCIConfig::default().dist);
        }
        Ok(())
    }
}
