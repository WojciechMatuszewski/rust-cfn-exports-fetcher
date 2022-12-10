use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
use validator::{Validate, ValidationError};

#[derive(thiserror::Error, Debug, PartialEq)]
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
pub struct ConfigFile {
    pub location: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ConfigEntry {
    #[validate(required)]
    pub stack_name: Option<String>,

    pub region: Option<String>,

    #[validate(custom = "validate_json_file")]
    pub json: ConfigFile,

    #[validate(custom = "validate_typescript_file")]
    pub typescript: ConfigFile,
}

type Config = Vec<ConfigEntry>;
pub fn parse(mut reader: impl io::Read) -> Result<Config, Error> {
    let mut contents = String::new();
    reader.read_to_string(&mut contents).map_err(|error| {
        return Error::ParsingError(error.to_string());
    })?;

    let config: Config = match serde_yaml::from_str(&contents) {
        Ok(data) => Ok(data),
        Err(error) => Err(Error::ParsingError(error.to_string())),
    }?;

    for config_entry in &config {
        match config_entry.validate() {
            Ok(_) => (),
            Err(error) => return Err(Error::ValidationError(error.to_string())),
        }
    }

    return Ok(config);
}

fn validate_json_file(json_file: &ConfigFile) -> Result<(), ValidationError> {
    let file_extension = match json_file.location.extension() {
        Some(extension) => extension,
        None => {
            return Err(ValidationError::new(
                "Unable to parse the extension of the typescript file location",
            ))
        }
    };
    if file_extension != "json" {
        return Err(ValidationError::new(
            "The JSON file location has to end with `.json`",
        ));
    }

    return Ok(());
}

fn validate_typescript_file(typescript_file: &ConfigFile) -> Result<(), ValidationError> {
    let file_extension = match typescript_file.location.extension() {
        Some(extension) => extension,
        None => {
            return Err(ValidationError::new(
                "Unable to parse the extension of the typescript file location",
            ))
        }
    };
    if file_extension != "ts" {
        return Err(ValidationError::new(
            "The TypeScript file location has to end with `.ts`",
        ));
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use super::parse;
    use super::Config;
    use super::ConfigEntry;
    use super::ConfigFile;
    use super::Error;

    #[test]
    fn parsing_error() {
        // Fails when used with non-utf-8 characters
        let input = io::Cursor::new(String::from("Côte d'Ivoire"));

        let result = parse(input);
        assert_eq!(true, result.is_err());
        match result.err().unwrap() {
            Error::ParsingError(_) => {}
            _ => panic!("Expected `ValidationError` error"),
        }
    }

    #[test]
    fn file_missing_stack_name() {
        let config_entry = ConfigEntry {
            stack_name: None,
            json: ConfigFile {
                location: PathBuf::from("foo.json"),
            },
            typescript: ConfigFile {
                location: PathBuf::from("something.ts"),
            },
            region: None,
        };

        let config: Config = vec![config_entry];
        let config_contents = serde_yaml::to_string(&config).unwrap();
        let input = io::Cursor::new(config_contents);

        let result = parse(input);
        assert_eq!(true, result.is_err());
        match result.err().unwrap() {
            Error::ValidationError(_) => {}
            _ => panic!("Expected `ValidationError` error"),
        }
    }

    #[test]
    fn parses_the_config() {
        let config_entry = ConfigEntry {
            stack_name: Some(String::from("stack_name")),
            json: ConfigFile {
                location: PathBuf::from("foo.json"),
            },
            typescript: ConfigFile {
                location: PathBuf::from("something.ts"),
            },
            region: None,
        };

        let config: Config = vec![config_entry];
        let config_contents = serde_yaml::to_string(&config).unwrap();
        let input = io::Cursor::new(config_contents);

        let result = parse(input);
        assert_eq!(false, result.is_err());
    }
}
