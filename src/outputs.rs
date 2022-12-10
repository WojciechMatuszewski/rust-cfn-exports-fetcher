

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudformation::Region;

use crate::config::ConfigEntry;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Service error ocurred: {0}.")]
    ServiceError(String),

    #[error("Unknown error ocurred: {0}.")]
    UnknownError(String),

    #[error("Stack not found")]
    NotFoundError(String),
}

pub struct Stack {
    pub stack_name: String,

    client: aws_sdk_cloudformation::Client,
}

impl Stack {
    pub async fn new(config_entry: &ConfigEntry) -> Self {
        let region = config_entry.region.as_ref();
        let stack_name = config_entry.stack_name.as_ref().unwrap().clone();

        let region = match region {
            Some(provided_region) => Region::new(provided_region.clone()),
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

    pub async fn get_outputs(&self) -> Result<Vec<aws_sdk_cloudformation::model::Output>, Error> {
        let result = self
            .client
            .describe_stacks()
            .stack_name(&self.stack_name)
            .send()
            .await;

        let result = match result {
            Ok(data) => data,
            Err(aws_sdk_cloudformation::types::SdkError::ServiceError { err, .. }) => {
                return Err(Error::ServiceError(err.to_string()));
            }
            Err(err) => return Err(Error::UnknownError(err.to_string())),
        };

        let stacks = result.stacks().unwrap_or_else(|| &[]);
        if stacks.len() == 0 {
            return Err(Error::NotFoundError(self.stack_name.clone()));
        }

        let outputs = stacks
            .get(0)
            .expect("Should not get here")
            .outputs()
            .unwrap_or_else(|| &[])
            .to_vec();

        return Ok(outputs);
    }

    // pub async fn generate_outputs(&self) -> Result<(), aws_sdk_cloudformation::Error> {
    //     let typings_contents = generate_typings_file(outputs);
    //     println!("{}", typings_contents);

    //     let json_contents = generate_json_file(outputs);
    //     println!("{}", json_contents);

    //     return Ok(());
    // }
}

// fn generate_typings_file(outputs: &[aws_sdk_cloudformation::model::Output]) -> String {}

// fn generate_json_file(outputs: &[aws_sdk_cloudformation::model::Output]) -> String {}
