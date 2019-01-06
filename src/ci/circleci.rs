use serde_derive::Serialize;
use std::io;

use super::CISystem;
use crate::config::MatrixEntryExt;
use crate::TemplateCIConfig;

use askama::Template;

#[derive(Template, Debug)]
#[template(path = "circleci.yml")]
pub(crate) struct CircleCI {
    conf: TemplateCIConfig,
    filters: Filters,
}

impl From<TemplateCIConfig> for CircleCI {
    fn from(conf: TemplateCIConfig) -> Self {
        CircleCI {
            filters: Filters::from_config(&conf),
            conf,
        }
    }
}

impl CISystem for CircleCI {
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
    fn from_config(_conf: &TemplateCIConfig) -> Filters {
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
