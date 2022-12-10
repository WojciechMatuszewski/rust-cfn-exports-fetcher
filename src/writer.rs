use std::fs::{self};
use std::{collections::HashMap, path::PathBuf};

use crate::config::ConfigEntry;

pub fn write(config_entry: &ConfigEntry, outputs: &Vec<aws_sdk_cloudformation::model::Output>) {
    write_json(&config_entry.json.location, &outputs);
    write_file_typings(&config_entry.typescript.location, &outputs);
}

fn write_json(path: &PathBuf, outputs: &Vec<aws_sdk_cloudformation::model::Output>) {
    let init: HashMap<&str, &str> = HashMap::new();
    let contents = outputs.into_iter().fold(init, |mut acc, output| {
        let key = output.output_key().unwrap();
        let value = output.output_value().unwrap();

        acc.insert(key, value);
        return acc;
    });

    let file_contents = serde_json::to_string(&contents).unwrap();
    fs::write(path, file_contents).unwrap();
}

fn write_file_typings(path: &PathBuf, outputs: &Vec<aws_sdk_cloudformation::model::Output>) {
    let contents = outputs
        .into_iter()
        .fold(String::from(""), |mut acc, output| {
            let output_key = output.output_key().unwrap();
            let type_entry = format!("{}: string,\n", output_key);

            acc.push_str(&type_entry);
            return acc;
        });

    let file_contents = format!(
        "declare module NodeJs {{ interface ProcessEnv {{ {} }} }}",
        contents
    );
    fs::write(path, file_contents).unwrap();
}
