use std::io;

use super::CISystem;
use crate::config::MatrixEntryExt;
use crate::TemplateCIConfig;

use askama::Template;

#[derive(Template, Debug)]
#[template(path = "travis.yml")]
pub(crate) struct TravisCI {
    conf: TemplateCIConfig,
}

impl From<TemplateCIConfig> for TravisCI {
    fn from(conf: TemplateCIConfig) -> Self {
        TravisCI { conf }
    }
}

impl CISystem for TravisCI {
    fn write_preamble(&self, mut _output: impl io::Write) -> Result<(), super::Error> {
        Ok(())
    }
}
