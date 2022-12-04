use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};
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
pub fn parse(path: &PathBuf) -> Result<Config, Error> {
    let contents = match fs::read_to_string(path) {
        Ok(raw_contents) => Ok(raw_contents),
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => Err(Error::FileNotFound(path.display().to_string())),
            _ => Err(Error::Unknown(error.to_string())),
        },
    }?;

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
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use super::parse;
    use super::Config;
    use super::ConfigEntry;
    use super::ConfigFile;
    use super::Error;
    use tempfile::tempdir;

    #[test]
    fn file_does_not_exist() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");

        let result = parse(&file_path);
        assert_eq!(true, result.is_err());
        match result.err().unwrap() {
            Error::FileNotFound(_) => {}
            _ => panic!("Expected `FileNotFound` error"),
        }
    }

    #[test]
    fn file_wrong_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Not yaml").unwrap();

        let result = parse(&file_path);
        assert_eq!(true, result.is_err());
        match result.err().unwrap() {
            Error::ParsingError(_) => {}
            _ => panic!("Expected `FileNotFound` error"),
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

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", config_contents).unwrap();

        let result = parse(&file_path);
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

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", config_contents).unwrap();

        let result = parse(&file_path);
        assert_eq!(false, result.is_err());
    }
}
