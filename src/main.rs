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

#[derive(Debug, Deserialize)]
struct MatrixEntry<'a> {
    run: bool,
    version: &'a str,
}

impl<'a> MatrixEntry<'a> {
    fn new(version: &'a str, run: bool) -> MatrixEntry<'a> {
        MatrixEntry { version, run }
    }
}

#[derive(Template, Debug, Deserialize)]
#[template(path = "travis.yml")]
struct TemplateCIConfig<'a> {
    #[serde(borrow)]
    #[serde(default = "TemplateCIConfig::default_bench")]
    bench: MatrixEntry<'a>,

    #[serde(borrow)]
    #[serde(default = "TemplateCIConfig::default_clippy")]
    clippy: MatrixEntry<'a>,

    #[serde(borrow)]
    #[serde(default = "TemplateCIConfig::default_rustfmt")]
    rustfmt: MatrixEntry<'a>,

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
            clippy: MatrixEntry::new("nightly", true),
            bench: MatrixEntry::new("nightly", false),
            rustfmt: MatrixEntry::new("stable", true),
            dist: "xenial",
            os: "linux",
            versions: vec!["stable", "beta", "nightly"],
        }
    }
}

impl<'a> TemplateCIConfig<'a> {
    fn default_bench() -> MatrixEntry<'a> {
        Self::default().bench
    }
    fn default_clippy() -> MatrixEntry<'a> {
        Self::default().clippy
    }
    fn default_rustfmt() -> MatrixEntry<'a> {
        Self::default().rustfmt
    }
    fn default_dist() -> &'a str {
        Self::default().dist
    }
    fn default_os() -> &'a str {
        Self::default().os
    }
    fn default_versions() -> Vec<&'a str> {
        Self::default().versions
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
