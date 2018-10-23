use std::collections::HashMap;

use askama::Template;
use cargo_metadata;
use serde::de::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use serde_json;
use std::io::Write;
use std::path::Path;
use tempfile;

macro_rules! define_matrix_entry {
    ($name:ident,
     ($run_default:expr,
      $version_default:expr,
      $allow_failure_default:expr,
      $commandline_default:expr)) => {
        #[derive(Debug)]
        struct $name<'a> {
            run: bool,
            version: &'a str,
            allow_failure: bool,
            // TODO: this needs to be shell-escaped!
            install_commandline: Option<String>,
            commandline: String,
        }

        impl<'a> Default for $name<'a> {
            fn default() -> Self {
                $name {
                    run: $run_default,
                    version: $version_default,
                    allow_failure: $allow_failure_default,
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
                    allow_failure: Option<bool>,
                    install_commandline: Option<String>,
                    commandline: Option<String>,
                }
                impl<'a> Default for DeserializationStruct<'a> {
                    fn default() -> Self {
                        DeserializationStruct {
                            run: Some($run_default),
                            version: Some($version_default),
                            allow_failure: Some($allow_failure_default),
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
                    allow_failure: raw
                        .allow_failure
                        .or(DeserializationStruct::default().allow_failure)
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
    (false, "nightly", false, Some("cargo bench".to_owned()))
);
define_matrix_entry!(
    ClippyEntry,
    (
        true,
        "nightly",
        false,
        Some("cargo clippy -- -D warnings".to_owned())
    )
);
define_matrix_entry!(
    RustfmtEntry,
    (true, "stable", false, Some("cargo fmt".to_owned()))
);

define_matrix_entry!(CustomEntry, (false, "stable", false, None));

#[derive(Template, Debug, Deserialize)]
#[template(path = "travis.yml")]
struct TemplateCIConfig<'a> {
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
            versions: vec!["stable", "beta", "nightly"],
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

    fn has_any_matrix_entries(&self) -> bool {
        self.bench.run
            || self.clippy.run
            || self.rustfmt.run
            || self.additional_matrix_entries.iter().any(|(_, r)| r.run)
    }

    fn has_any_allowed_failures(&self) -> bool {
        self.has_any_matrix_entries() && self.bench.allow_failure
            || self.clippy.allow_failure
            || self.rustfmt.allow_failure
            || self
                .additional_matrix_entries
                .iter()
                .any(|(_, r)| r.allow_failure)
    }
}

#[derive(Debug, Deserialize)]
struct Metadata<'a> {
    #[serde(default)]
    #[serde(borrow)]
    template_ci: TemplateCIConfig<'a>,
}

fn main() {
    let app = clap::App::new("cargo-template-ci").subcommand(
        clap::SubCommand::with_name("template-ci").arg(
            clap::Arg::with_name("travis-config")
                .long("travis-config")
                .value_name("PATH")
                .takes_value(true),
        ),
    );
    let matches = app.get_matches();

    let md = cargo_metadata::metadata(None).expect("Could not get cargo metadata");
    let pkg_metadata = md.packages[0].metadata.to_string();
    let config: Metadata<'_> = serde_json::from_str(&pkg_metadata).expect("Could not parse config");

    // rewrite .travis.yml:
    let dest = Path::new(matches.value_of("travis-config").unwrap_or(".travis.yml"));
    let dest = dest.canonicalize().expect("could not canonicalize path");
    let dest_dir = dest.parent().unwrap();
    let output =
        tempfile::NamedTempFile::new_in(dest_dir).expect("Could not create temporary file");

    writeln!(&output, "# {:?}", config.template_ci).unwrap();
    writeln!(&output, "{}", config.template_ci.render().unwrap()).unwrap();
    output.persist(dest).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
