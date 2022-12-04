use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};
use validator::{Validate, ValidationError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("File {0} not found")]
    FileNotFound(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Validation errors: {0}")]
    ValidationError(String),

    #[error("Unknown error occurred: {0}")]
    Unknown(String),
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct File {
    location: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ConfigEntry {
    stack_name: String,
    region: Option<String>,
    #[validate(custom = "validate_json_file")]
    json: File,
    #[validate(custom = "validate_typescript_file")]
    typescript: File,
}

type Config = Vec<ConfigEntry>;
// Abstract the `path` into some kind of trait or generic io thingy so that we can test it?
// Or use a temp dir abstraction
pub fn parse(path: &PathBuf) -> Result<(), Error> {
    let contents = match fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => Err(Error::FileNotFound(path.display().to_string())),
            _ => Err(Error::Unknown(error.to_string())),
        },
    }?;

    let config: Config = match serde_yaml::from_str(&contents) {
        Ok(data) => Ok(data),
        Err(error) => Err(Error::ParsingError(error.to_string())),
    }?;

    for config_entry in config {
        match config_entry.validate() {
            Ok(_) => (),
            Err(error) => return Err(Error::ValidationError(error.to_string())),
        }
    }

    return Ok(());
}

fn validate_json_file(json_file: &File) -> Result<(), ValidationError> {
    let file_extension = json_file.location.extension().unwrap();
    if file_extension != "json" {
        return Err(ValidationError::new(
            "The JSON file location has to end with `.json`",
        ));
    }

    return Ok(());
}

fn validate_typescript_file(typescript_file: &File) -> Result<(), ValidationError> {
    let file_extension = typescript_file.location.extension().unwrap();
    if file_extension != "ts" {
        return Err(ValidationError::new(
            "The TypeScript file location has to end with `.ts`",
        ));
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use tempfile::tempdir;

    use super::parse;

    #[test]
    fn file_does_not_exist() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Brian was here. Briefly.").unwrap();

        // How to assert on specific error??
        let result = parse(&file_path);
        assert_eq!(result.is_err(), true)
    }
}
