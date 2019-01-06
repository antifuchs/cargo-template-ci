use askama;
use std::io;
use std::io::Write;
use std::path::PathBuf;

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
    fn render_into_config_file(&self, destination: PathBuf) -> Result<(), Error> {
        let dest = destination.as_path();
        // TODO: canonicalization fails if the file does not exist:
        let dest = dest.canonicalize()?;
        let dest_dir = dest.parent().unwrap(); // TODO: that unwrap
        let output = tempfile::NamedTempFile::new_in(dest_dir)?;

        self.write_preamble(&output)?;
        writeln!(&output, "{}", self.render()?)?;
        output.persist(dest)?;
        Ok(())
    }
}
