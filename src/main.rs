use std::path::PathBuf;

pub mod config;
pub mod outputs;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_cloudformation::Error> {
    let config_path = PathBuf::from("./config.yaml");
    let config = config::parse(&config_path).unwrap();

    for config_entry in config {
        outputs::Stack::new(config_entry.stack_name.unwrap(), config_entry.region)
            .await
            .generate_outputs()
            .await
            .unwrap()
    }

    return Ok(());
}
