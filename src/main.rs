#![deny(warnings)]

use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
mod macros;

mod bors;
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
    TravisCI,

    #[structopt(name = "circleci", about = "Generate circleci configuration")]
    CircleCI,
}

impl Default for GenerateCommand {
    fn default() -> Self {
        GenerateCommand::TravisCI
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Cmdline::from_args();
    let Cmdline::TemplateCI {
        cmd,
        cargo_manifest,
    } = opts;

    let (conf, dest) = config::TemplateCIConfig::merged_configs(cargo_manifest.as_deref())?;

    let res = match cmd.unwrap_or_default() {
        GenerateCommand::TravisCI => TravisCI::from(conf).render_into_config_file(&dest),
        GenerateCommand::CircleCI => CircleCI::from(conf).render_into_config_file(&dest),
    };
    if let Err(e) = res {
        eprintln!("Generating CI config failed. {}", e);
        return Err(e.into());
    }
    Ok(())
}
