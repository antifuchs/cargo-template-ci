use std::collections::HashMap;

use cargo_metadata;
use serde::de::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use serde_json;
use std::path::PathBuf;
use structopt::StructOpt;

mod ci;

use crate::ci::{circleci::CircleCI, travis::TravisCI, CISystem};

macro_rules! define_matrix_entry {
    ($name:ident,
     ($run_default:expr,
      $version_default:expr,
      $commandline_default:expr)) => {
        #[derive(Debug)]
        struct $name<'a> {
            run: bool,
            version: &'a str,
            // TODO: this needs to be shell-escaped!
            install_commandline: Option<String>,
            commandline: String,
        }

        impl<'a> Default for $name<'a> {
            fn default() -> Self {
                $name {
                    run: $run_default,
                    version: $version_default,
                    install_commandline: None,
                    commandline: $commandline_default.unwrap_or("/bin/false".to_owned()),
                }
            }
        }

        // Since we can't easily (or at all?) pass default expresisons
        // to serde, we have to define our own
        // deserializer. Thankfully, you can deserialize into an
        // intermediate struct and then assign / default the values
        // from Default::default().
        impl<'de: 'a, 'a> Deserialize<'de> for $name<'a> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                #[derive(Deserialize)]
                struct DeserializationStruct<'a> {
                    run: Option<bool>,
                    version: Option<&'a str>,
                    install_commandline: Option<String>,
                    commandline: Option<String>,
                }
                impl<'a> Default for DeserializationStruct<'a> {
                    fn default() -> Self {
                        DeserializationStruct {
                            run: Some($run_default),
                            version: Some($version_default),
                            install_commandline: None,
                            commandline: $commandline_default,
                        }
                    }
                }
                let raw: DeserializationStruct = DeserializationStruct::deserialize(deserializer)?;
                let res = $name {
                    run: raw.run.or(DeserializationStruct::default().run).unwrap(),
                    version: raw
                        .version
                        .or(DeserializationStruct::default().version)
                        .unwrap(),
                    install_commandline: raw
                        .install_commandline
                        .or(DeserializationStruct::default().install_commandline),
                    commandline: raw
                        .commandline
                        .or(DeserializationStruct::default().commandline)
                        .expect("Matrix entries need a commandline"),
                };
                Ok(res)
            }
        }
    };
}

define_matrix_entry!(
    BenchEntry,
    (false, "nightly", Some("cargo bench".to_owned()))
);
define_matrix_entry!(
    ClippyEntry,
    (
        true,
        "nightly",
        Some("cargo clippy -- -D warnings".to_owned())
    )
);
define_matrix_entry!(RustfmtEntry, (true, "stable", Some("cargo fmt".to_owned())));

define_matrix_entry!(CustomEntry, (false, "stable", None));

#[derive(Debug, Deserialize)]
pub(crate) struct TemplateCIConfig<'a> {
    #[serde(borrow)]
    #[serde(default)]
    bench: BenchEntry<'a>,

    #[serde(borrow)]
    #[serde(default)]
    clippy: ClippyEntry<'a>,

    #[serde(borrow)]
    #[serde(default)]
    rustfmt: RustfmtEntry<'a>,

    #[serde(borrow)]
    #[serde(default)]
    additional_matrix_entries: HashMap<&'a str, CustomEntry<'a>>,

    #[serde(default = "TemplateCIConfig::default_cache")]
    cache: &'a str,

    #[serde(default = "TemplateCIConfig::default_os")]
    os: &'a str,

    #[serde(default = "TemplateCIConfig::default_dist")]
    dist: &'a str,

    #[serde(default = "TemplateCIConfig::default_versions")]
    #[serde(borrow)]
    versions: Vec<&'a str>,

    #[serde(default = "TemplateCIConfig::default_test_commandline")]
    test_commandline: String,
}

impl<'a> Default for TemplateCIConfig<'a> {
    fn default() -> Self {
        TemplateCIConfig {
            clippy: Default::default(),
            bench: Default::default(),
            rustfmt: Default::default(),
            additional_matrix_entries: Default::default(),
            dist: "xenial",
            cache: "cargo",
            os: "linux",
            versions: vec!["stable", "nightly"],
            test_commandline: "cargo test --verbose --all".to_owned(),
        }
    }
}

impl<'a> TemplateCIConfig<'a> {
    fn default_cache() -> &'a str {
        Self::default().cache
    }

    fn default_os() -> &'a str {
        Self::default().os
    }

    fn default_dist() -> &'a str {
        Self::default().dist
    }

    fn default_versions() -> Vec<&'a str> {
        Self::default().versions
    }

    fn default_test_commandline() -> String {
        Self::default().test_commandline
    }
}

#[derive(Debug, Deserialize)]
struct Metadata<'a> {
    #[serde(default)]
    #[serde(borrow)]
    template_ci: TemplateCIConfig<'a>,
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cargo-template-ci",
    about = "Generate a reasonable CI config file from Cargo.toml"
)]
enum Cmdline {
    #[structopt(name = "template-ci")]
    TemplateCI {
        #[structopt(subcommand)]
        cmd: Option<GenerateCommand>,
    },
}

#[derive(StructOpt, Debug)]
enum GenerateCommand {
    #[structopt(name = "travis", about = "Generate travis-ci configuration")]
    TravisCI {
        #[structopt(long = "travis-config", help = "Path to travis CI yaml config")]
        config_path: Option<String>,
    },

    #[structopt(name = "circleci", about = "Generate circleci configuration")]
    CircleCI,
}

impl Default for GenerateCommand {
    fn default() -> Self {
        GenerateCommand::TravisCI { config_path: None }
    }
}

fn main() {
    let opts = Cmdline::from_args();

    let md = cargo_metadata::metadata(None).expect("Could not get cargo metadata");
    let pkg_metadata = md.packages[0].metadata.to_string();
    let config: Metadata<'_> = serde_json::from_str(&pkg_metadata).expect("Could not parse config");

    let Cmdline::TemplateCI { cmd } = opts;
    match cmd.unwrap_or_default() {
        GenerateCommand::TravisCI { config_path } => {
            TravisCI::from(config.template_ci)
                .render_into_config_file(PathBuf::from(
                    config_path.unwrap_or_else(|| ".travis.yml".to_string()),
                ))
                .expect("Failed to generate travis config");
        }
        GenerateCommand::CircleCI => {
            CircleCI::from(config.template_ci)
                .render_into_config_file(PathBuf::from(".circleci/config.yml"))
                .expect("Failed to generate travis config");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
