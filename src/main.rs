#![deny(warnings)]

use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
mod macros;

mod ci;
mod config;

use crate::ci::{circleci::CircleCI, travis::TravisCI, CISystem};
pub(crate) use crate::config::TemplateCIConfig;

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
        #[structopt(long = "manifest", help = "Path to Cargo.toml", parse(from_os_str))]
        cargo_manifest: Option<PathBuf>,
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

fn main() -> Result<(), Box<std::error::Error>> {
    let opts = Cmdline::from_args();
    let Cmdline::TemplateCI {
        cmd,
        cargo_manifest,
    } = opts;

    let conf: config::TemplateCIConfig =
        config::TemplateCIConfig::from_manifest(cargo_manifest.as_ref().map(|pb| pb.as_path()))?;

    match cmd.unwrap_or_default() {
        GenerateCommand::TravisCI { config_path } => {
            TravisCI::from(conf).render_into_config_file(PathBuf::from(
                config_path.unwrap_or_else(|| ".travis.yml".to_string()),
            ))?;
        }
        GenerateCommand::CircleCI => {
            CircleCI::from(conf).render_into_config_file(PathBuf::from(".circleci/config.yml"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
