use askama::Template;
use cargo_metadata;
use serde::de::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use serde_json;

macro_rules! define_matrix_entry {
    ($name:ident, ($run_default:expr, $version_default:expr, $allow_failure_default:expr)) => {
        #[derive(Debug)]
        struct $name<'a> {
            run: bool,
            version: &'a str,
            allow_failure: bool,
        }

        impl<'a> Default for $name<'a> {
            fn default() -> Self {
                $name {
                    run: $run_default,
                    version: $version_default,
                    allow_failure: $allow_failure_default,
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
                }
                let raw: DeserializationStruct = DeserializationStruct::deserialize(deserializer)?;
                let res = $name {
                    run: raw.run.unwrap_or(Self::default().run),
                    version: raw.version.unwrap_or(Self::default().version),
                    allow_failure: raw.allow_failure.unwrap_or(Self::default().allow_failure),
                };
                Ok(res)
            }
        }
    };
}

define_matrix_entry!(BenchEntry, (false, "nightly", false));
define_matrix_entry!(ClippyEntry, (true, "nightly", false));
define_matrix_entry!(RustfmtEntry, (true, "stable", false));

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

    #[serde(default = "TemplateCIConfig::default_os")]
    os: &'a str,

    #[serde(default = "TemplateCIConfig::default_dist")]
    dist: &'a str,

    #[serde(default = "TemplateCIConfig::default_versions")]
    #[serde(borrow)]
    versions: Vec<&'a str>,
}

impl<'a> Default for TemplateCIConfig<'a> {
    fn default() -> Self {
        TemplateCIConfig {
            clippy: Default::default(),
            bench: Default::default(),
            rustfmt: Default::default(),
            dist: "xenial",
            os: "linux",
            versions: vec!["stable", "beta", "nightly"],
        }
    }
}

impl<'a> TemplateCIConfig<'a> {
    fn default_os() -> &'a str {
        Self::default().os
    }

    fn default_dist() -> &'a str {
        Self::default().dist
    }

    fn default_versions() -> Vec<&'a str> {
        Self::default().versions
    }
}

#[derive(Debug, Deserialize)]
struct Metadata<'a> {
    #[serde(default)]
    #[serde(borrow)]
    template_ci: TemplateCIConfig<'a>,
}

fn main() {
    let md = cargo_metadata::metadata(None).expect("Could not get cargo metadata");
    let pkg_metadata = md.packages[0].metadata.to_string();
    let config: Metadata<'_> = serde_json::from_str(&pkg_metadata).expect("Could not parse config");

    println!("# {:?}", config.template_ci);
    println!("{}", config.template_ci.render().unwrap());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
