use custom_error::custom_error;
use io::Read;
use serde_derive::Deserialize;
use std::{fs::File, io, path::Path};

#[derive(PartialEq, Debug, Deserialize)]
pub(crate) struct BorsConfig {
    pub(crate) status: Vec<String>,
}

custom_error! {pub Error
               IOError{source: io::Error} = "could not read bors-ng config",
               Toml{source: toml::de::Error} = "could not parse bors-ng config as TOML",
               BadCircleStatusCheck{name: String} = "Bad status check {:?}: Use \"continuous_integration\"",
               MissingCircleStatusCheck{name: String} = "Missing status check {:?}",
}

pub(crate) fn config(root: &Path) -> Result<BorsConfig, Error> {
    let mut f = File::open(root.join("bors.toml"))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(toml::from_str(&buf)?)
}
