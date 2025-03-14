use reqwest::Url;
use std::env;
use dotenv::dotenv;
use serde::Deserialize;
use serde_json::Value::String;
use crate::strong_api::{Includes, StrongApi};
use crate::data_transformer::{DataTransformer, DataTransformerImpl};

mod strong_api;
mod user_response;
mod data_transformer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let username = env::var("STRONG_USER").expect("STRONG_USER must be set");
    let password = env::var("STRONG_PASS").expect("STRONG_PASS must be set");
    let strong_backend = env::var("STRONG_BACKEND").expect("STRONG_BACKEND must be set");

    let url = Url::parse(&*strong_backend).ok().unwrap();

    let mut strong_api = StrongApi::new(url);

    strong_api.login(username.as_str(), password.as_str()).await?;
    let user = strong_api.get_user(
        "",
        500,
        vec![
            Includes::Log,
            Includes::Folder,
            Includes::Measurement,
            Includes::MeasuredValue,
            Includes::Tag,
            Includes::Template,
            Includes::Widget
        ]
    ).await?;



    for log in &user.embedded.log {
        if let Some(start_date) = &log.start_date {
            print!("{}: ", start_date);
        }

        let display_name = log.name.as_ref()
            .and_then(|name| name.custom.as_deref().or(name.en.as_deref()))
            .unwrap_or("Unknown");

        println!("{}", display_name);
        for cell_set_group in &log.embedded.cell_set_group {
            /*for cell_set in &cell_set_group.cell_sets {
                for cell in &cell_set.cells {
                    print!("{} - {:?} | ", cell.cell_type, cell.value);
                }
            }*/
            println!();
            dbg!(DataTransformerImpl.transform(cell_set_group).expect("TODO: panic message"));
        }
        println!();
    }


    /*strong_api.get_logs().await?;

    dbg!(strong_api.access_token);
    dbg!(strong_api.refresh_token);
    dbg!(strong_api.user_id);*/

    Ok(())
}