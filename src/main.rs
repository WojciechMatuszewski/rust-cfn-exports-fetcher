use std::{collections::HashMap, fs, path::PathBuf};

use aws_config::meta::region::RegionProviderChain;
use aws_types::region::Region;
use futures;
use serde::{Deserialize, Serialize};

pub mod config;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_cloudformation::Error> {
    let config_path = PathBuf::from("./config.yaml");
    let result = config::parse(&config_path);
    dbg!(result);

    // let config = load_config();
    // dbg!(&config);

    // let handles = config.iter().fold(vec![], |mut acc, config_entry| {
    //     let handle = tokio::spawn(generate(config_entry.to_owned()));
    //     acc.push(handle);
    //     return acc;
    // });
    // futures::future::join_all(handles).await;

    return Ok(());
}

fn generate_typings_file(outputs: &[aws_sdk_cloudformation::model::Output]) -> String {
    let contents = outputs
        .into_iter()
        .fold(String::from(""), |mut acc, output| {
            let output_key = output.output_key().unwrap();
            let type_entry = format!("{}: string,\n", output_key);
            acc.push_str(&type_entry);
            return acc;
        });

    return format!(
        "declare module NodeJs {{ interface ProcessEnv {{ {} }} }}",
        contents
    );
}

fn generate_json_file(outputs: &[aws_sdk_cloudformation::model::Output]) -> String {
    let init: HashMap<&str, &str> = HashMap::new();
    let contents = outputs.into_iter().fold(init, |mut acc, output| {
        let key = output.output_key().unwrap();
        let value = output.output_value().unwrap();
        acc.insert(key, value);
        return acc;
    });

    return serde_json::to_string(&contents).unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConfigEntry {
    region: Option<String>,
    stack_name: String,
}

fn load_config() -> Vec<ConfigEntry> {
    let contents =
        fs::read_to_string("config.yaml").expect("Should have been able to read the file");

    let config: Vec<ConfigEntry> =
        serde_yaml::from_str(&contents).expect("Failed to parse the config file");

    return config;
}

async fn generate(config: ConfigEntry) -> Result<(), aws_sdk_cloudformation::Error> {
    let region = match config.region {
        Some(provided_region) => Region::new(provided_region),
        None => RegionProviderChain::default_provider()
            .region()
            .await
            .unwrap(),
    };
    let stack_name = config.stack_name.as_str();

    let sdk_config = aws_config::from_env().region(region).load().await;
    let client = aws_sdk_cloudformation::Client::new(&sdk_config);

    let result = client.describe_stacks().stack_name(stack_name).send().await;
    let result = match result {
        Ok(data) => data,
        Err(aws_sdk_cloudformation::types::SdkError::ServiceError { err, .. }) => {
            println!("Service error: {}", err.message().unwrap());
            return Ok(());
        }
        Err(err) => {
            println!("Unknown error: {}", err.to_string());
            return Ok(());
        }
    };

    let stacks = result.stacks().unwrap_or_else(|| &[]);
    if stacks.len() == 0 {
        println!("Stack: {} not found", stack_name);
        return Ok(());
    }

    let outputs = stacks
        .get(0)
        .expect("Should not get here")
        .outputs()
        .unwrap_or_else(|| &[]);

    if outputs.len() == 0 {
        println!("Stack: {} does not have any outputs", stack_name);
        return Ok(());
    }

    let typings_contents = generate_typings_file(outputs);
    println!("{}", typings_contents);

    let json_contents = generate_json_file(outputs);
    println!("{}", json_contents);

    return Ok(());
}
