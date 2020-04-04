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

#[cfg(test)]
mod test {
    use super::super::Error;
    use super::*;
    use io::Write;
    use std::{fs::File, io};
    use tempfile;

    #[test]
    fn validate_old_style_bors_config() -> Result<(), Box<dyn std::error::Error>> {
        let sys = CircleCI::from(TemplateCIConfig::default());
        let tmp = tempfile::tempdir()?;
        let dir = tmp.path();
        File::create(dir.join("bors.toml"))?
            .write_all("status = [\"ci/circleci: ci_success\"]".as_bytes())?;
        match sys.validate_config(dir) {
            Err(Error::BorsConfig {
                source: bors::Error::BadCircleStatusCheck { name },
            }) => {
                assert_eq!(name, "ci/circleci: ci_success");
            }
            other => {
                panic!("Expected an error, got {:?}", other);
            }
        }
        Ok(())
    }

    #[test]
    fn validate_missing_status_check() -> Result<(), Box<dyn std::error::Error>> {
        let sys = CircleCI::from(TemplateCIConfig::default());
        let tmp = tempfile::tempdir()?;
        let dir = tmp.path();
        File::create(dir.join("bors.toml"))?.write_all("status = [\"welp\"]".as_bytes())?;
        match sys.validate_config(dir) {
            Err(Error::BorsConfig {
                source: bors::Error::MissingCircleStatusCheck { name },
            }) => {
                assert_eq!(name, "continuous_integration");
            }
            other => {
                panic!("Expected an error, got {:?}", other);
            }
        }
        Ok(())
    }
}
