use std::io;

use super::CISystem;
use crate::TemplateCIConfig;

use askama::Template;

#[derive(Template, Debug)]
#[template(path = "circleci.yml")]
pub(crate) struct CircleCI<'a> {
    conf: TemplateCIConfig<'a>,
}

impl<'a> From<TemplateCIConfig<'a>> for CircleCI<'a> {
    fn from(conf: TemplateCIConfig<'a>) -> Self {
        CircleCI { conf }
    }
}

impl<'a> CISystem<'a> for CircleCI<'a> {
    fn write_preamble(&self, mut output: impl io::Write) -> Result<(), super::Error> {
        writeln!(&mut output, "# {:?}", self.conf)?;
        Ok(())
    }
}
