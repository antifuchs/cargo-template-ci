use askama::Template;
use cargo_metadata;
use serde_derive::Deserialize;
use serde_json;

#[derive(Debug, Deserialize)]
struct Metadata<'a> {
    #[serde(default)]
    #[serde(borrow)]
    template_ci: TemplateCIConfig<'a>,
}

#[derive(Template, Debug, Deserialize)]
#[template(path = "travis.yml")]
struct TemplateCIConfig<'a> {
    #[serde(default = "TemplateCIConfig::default_run_clippy")]
    run_clippy: bool,

    #[serde(default = "TemplateCIConfig::default_run_benchmark")]
    run_benchmark: bool,

    #[serde(default = "TemplateCIConfig::default_run_rustfmt")]
    run_rustfmt: bool,

    #[serde(default = "TemplateCIConfig::default_dist")]
    dist: &'a str,

    #[serde(default = "TemplateCIConfig::default_versions")]
    #[serde(borrow)]
    versions: Vec<&'a str>,

    #[serde(default = "TemplateCIConfig::default_benchmark_version")]
    benchmark_version: &'a str,

    #[serde(default = "TemplateCIConfig::default_rustfmt_version")]
    rustfmt_version: &'a str,

    #[serde(default = "TemplateCIConfig::default_clippy_version")]
    clippy_version: &'a str,
}

impl<'a> Default for TemplateCIConfig<'a> {
    fn default() -> Self {
        TemplateCIConfig {
            run_clippy: false,
            run_benchmark: false,
            run_rustfmt: true,
            dist: "xenial",
            versions: vec!["stable", "beta", "nightly"],
            benchmark_version: "nightly",
            rustfmt_version: "stable",
            clippy_version: "nightly",
        }
    }
}

impl<'a> TemplateCIConfig<'a> {
    fn default_run_rustfmt() -> bool {
        Self::default().run_rustfmt
    }
    fn default_run_clippy() -> bool {
        Self::default().run_clippy
    }
    fn default_run_benchmark() -> bool {
        Self::default().run_benchmark
    }
    fn default_dist() -> &'a str {
        Self::default().dist
    }
    fn default_versions() -> Vec<&'a str> {
        Self::default().versions
    }
    fn default_clippy_version() -> &'a str {
        Self::default().clippy_version
    }
    fn default_benchmark_version() -> &'a str {
        Self::default().benchmark_version
    }
    fn default_rustfmt_version() -> &'a str {
        Self::default().rustfmt_version
    }
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
