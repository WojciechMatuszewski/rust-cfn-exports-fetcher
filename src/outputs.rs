use std::collections::HashMap;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudformation::Region;

pub struct Stack {
    pub stack_name: String,

    client: aws_sdk_cloudformation::Client,
}

impl Stack {
    pub async fn new(stack_name: String, region: Option<String>) -> Self {
        let region = match region {
            Some(provided_region) => Region::new(provided_region),
            None => RegionProviderChain::default_provider()
                .region()
                .await
                .unwrap(),
        };

        let region_for_config = region.clone();
        let sdk_config = aws_config::from_env()
            .region(region_for_config)
            .load()
            .await;
        let client = aws_sdk_cloudformation::Client::new(&sdk_config);

        return Self { stack_name, client };
    }

    pub async fn generate_outputs(&self) -> Result<(), aws_sdk_cloudformation::Error> {
        let result = self
            .client
            .describe_stacks()
            .stack_name(&self.stack_name)
            .send()
            .await;

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
            println!("Stack: {} not found", &self.stack_name);
            return Ok(());
        }

        let outputs = stacks
            .get(0)
            .expect("Should not get here")
            .outputs()
            .unwrap_or_else(|| &[]);

        if outputs.len() == 0 {
            println!("Stack: {} does not have any outputs", &self.stack_name);
            return Ok(());
        }

        let typings_contents = generate_typings_file(outputs);
        println!("{}", typings_contents);

        let json_contents = generate_json_file(outputs);
        println!("{}", json_contents);

        return Ok(());
    }
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
