use crate::data_transformer::{DataTransformer, DataTransformerImpl};
use crate::strong_api::{Includes, StrongApi};
use dotenv::dotenv;
use reqwest::Url;
use std::env;

mod data_transformer;
mod strong_api;
mod user_response;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let username = env::var("STRONG_USER").expect("STRONG_USER must be set");
    let password = env::var("STRONG_PASS").expect("STRONG_PASS must be set");
    let strong_backend = env::var("STRONG_BACKEND").expect("STRONG_BACKEND must be set");

    let url = Url::parse(&strong_backend).ok().expect("STRONG_BACKEND is not a valid URL");

    let mut strong_api = StrongApi::new(url);

    strong_api
        .login(username.as_str(), password.as_str())
        .await?;
    let user = strong_api.get_user("", 500, vec![Includes::Log]).await?;

    let workouts = DataTransformerImpl
        .transform(&user.embedded.log)
        .expect("Couldn't read workouts");

    println!("Workout count: {}", workouts.len());

    workouts.iter().for_each(|workout| {
        println!("Workout: {}", workout.name);
        workout.exercises.iter().for_each(|exercise| {
            println!("Exercise: {}", exercise.id);
            exercise.sets.iter().for_each(|set| {
                println!("Set: {:?}", set);
            });
        });
    });

    Ok(())
}
