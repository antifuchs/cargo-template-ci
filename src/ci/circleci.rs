use serde_derive::Serialize;
use std::{io, path::Path};

use super::CISystem;
use crate::config::MatrixEntryExt;
use crate::{bors, TemplateCIConfig};

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
    fn write_preamble(&self, mut _output: impl io::Write) -> Result<(), super::Error> {
        Ok(())
    }

    /// Checks a bors.toml (if it exists) for the correct CI task names.
    fn validate_config(&self, root: &Path) -> Result<(), super::Error> {
        let bors_cfg = bors::config(root)?;
        if let Some(name) = bors_cfg
            .status
            .iter()
            .find(|&el| el == "ci/circleci: ci_success")
        {
            return Err(bors::Error::BadCircleStatusCheck {
                name: name.to_string(),
            }
            .into());
        }

        if bors_cfg
            .status
            .iter()
            .find(|&el| el == "continuous_integration")
            .is_none()
        {
            return Err(bors::Error::MissingCircleStatusCheck {
                name: "continuous_integration".to_string(),
            }
            .into());
        }
        Ok(())
    }

    fn config_file_name(&self, root: &Path) -> std::path::PathBuf {
        root.join(".circleci/config.yml")
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
