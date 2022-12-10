use std::{fs, path::PathBuf};

pub mod config;
pub mod outputs;
pub mod writer;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_cloudformation::Error> {
    let config_path = PathBuf::from("./config.yaml");
    let file = fs::File::open(config_path).expect("Config file missing");

    let config = config::parse(file).unwrap();
    for config_entry in config {
        let outputs = outputs::Stack::new(&config_entry)
            .await
            .get_outputs()
            .await
            .unwrap();

        writer::write(&config_entry, &outputs);
    }

    return Ok(());
}
