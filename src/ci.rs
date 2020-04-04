use askama;
use std::env::current_dir;
use std::fs::create_dir_all;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use custom_error::custom_error;

pub(crate) mod circleci;
pub(crate) mod travis;

custom_error! {pub Error
               TemplateError{source: askama::Error} = "could not render template",
               IOError{source: io::Error} = "could not write to CI config",
               PersistError{source: tempfile::PersistError} = "could not overwrite",
}

pub(crate) trait CISystem: askama::Template {
    /// Writes any comments / preamble / debug data to the CI config file.
    fn write_preamble(&self, output: impl io::Write) -> Result<(), Error>;

    /// Renders the CI system template and writes it to either the
    /// given config file or to the default location.
    fn render_into_config_file(&self, root: &Path) -> Result<(), Error> {
        let dest = self.config_file_name(root);
        let dest_dir = match dest.parent() {
            Some(dir) => dir.to_path_buf(),
            None => current_dir()?,
        };
        create_dir_all(&dest_dir)?;
        let output = tempfile::NamedTempFile::new_in(dest_dir)?;

        self.write_preamble(&output)?;
        writeln!(&output, "{}", self.render()?)?;
        output.persist(dest)?;
        Ok(())
    }

    /// Returns a configuration file name from the root of the repo.
    fn config_file_name(&self, root: &Path) -> PathBuf;
}

#[cfg(test)]
mod tests {
    use super::CISystem;
    use askama;
    use custom_error::custom_error;
    use std::fmt;
    use std::fs;
    use std::io;
    use tempfile;

    custom_error! {Error
                   IO{source: io::Error} = "IO",
                   Fmt{source: fmt::Error} = "fmt",
                   Tempfile{source: tempfile::PersistError} = "Test setup/teardown",
                   CIError{source: super::Error} = "error from the CI config mechanics",
    }

    struct NonSystem {}

    impl askama::Template for NonSystem {
        fn render_into(&self, _writer: &mut dyn fmt::Write) -> askama::Result<()> {
            Ok(())
        }

        fn extension(&self) -> Option<&str> {
            None
        }
    }

    impl CISystem for NonSystem {
        fn write_preamble(&self, _output: impl io::Write) -> Result<(), super::Error> {
            Ok(())
        }
        fn config_file_name(&self, root: &std::path::Path) -> std::path::PathBuf {
            root.join("does_not_exist.tmp")
        }
    }

    #[test]
    fn without_existing_file() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        {
            let sys = NonSystem {};
            let path = dir.path();
            sys.render_into_config_file(path)?;
            Ok(())
        }
    }

    #[test]
    fn when_dir_does_not_exist() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        {
            let sys = NonSystem {};
            let path = dir.path();
            sys.render_into_config_file(path)?;
            Ok(())
        }
    }

    #[test]
    fn when_dir_exists() -> Result<(), Error> {
        let dir = tempfile::tempdir()?;
        {
            let sys = NonSystem {};
            let path = dir.path();
            fs::create_dir(path.join("dir_exists"))?;
            sys.render_into_config_file(path)?;
            Ok(())
        }
    }
}
