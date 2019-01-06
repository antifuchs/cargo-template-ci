use std::io;

use super::CISystem;
use crate::TemplateCIConfig;

use askama::Template;

#[derive(Template, Debug)]
#[template(path = "travis.yml")]
pub(crate) struct TravisCI<'a> {
    conf: TemplateCIConfig<'a>,
}

impl<'a> From<TemplateCIConfig<'a>> for TravisCI<'a> {
    fn from(conf: TemplateCIConfig<'a>) -> Self {
        TravisCI { conf }
    }
}

impl<'a> CISystem<'a> for TravisCI<'a> {
    fn write_preamble(&self, mut output: impl io::Write) -> Result<(), super::Error> {
        writeln!(&mut output, "# {:?}", self.conf)?;
        Ok(())
    }
}
