use serde_derive::Serialize;
use std::io;

use super::CISystem;
use crate::TemplateCIConfig;

use askama::Template;

#[derive(Template, Debug)]
#[template(path = "circleci.yml")]
pub(crate) struct CircleCI<'a> {
    conf: TemplateCIConfig<'a>,
    filters: Filters,
}

impl<'a> From<TemplateCIConfig<'a>> for CircleCI<'a> {
    fn from(conf: TemplateCIConfig<'a>) -> Self {
        CircleCI {
            filters: Filters::from_config(&conf),
            conf,
        }
    }
}

impl<'a> CISystem<'a> for CircleCI<'a> {
    fn write_preamble(&self, mut output: impl io::Write) -> Result<(), super::Error> {
        writeln!(&mut output, "# {:?}", self.conf)?;
        Ok(())
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct SpecificFilters {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    only: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    ignore: Vec<String>,
}

#[derive(Serialize, Debug)]
pub(crate) struct Filters {
    branches: SpecificFilters,
    tags: SpecificFilters,
}

impl Filters {
    fn from_config<'a>(_conf: &'a TemplateCIConfig<'a>) -> Filters {
        // TODO: fill in configurable branches
        let branch_ignore_patterns = vec![r"/.*\.tmp/"];
        let tags = vec![r"/^v\d+\.\d+\.\d+.*$/"];
        Filters {
            branches: SpecificFilters {
                only: vec![],
                ignore: branch_ignore_patterns
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            tags: SpecificFilters {
                only: tags.into_iter().map(String::from).collect(),
                ignore: vec![],
            },
        }
    }
}
