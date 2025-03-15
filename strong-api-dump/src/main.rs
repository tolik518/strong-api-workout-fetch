use dotenv::dotenv;
use reqwest::Url;
use std::env;
use strong_api_lib::data_transformer::DataTransformer;
use strong_api_lib::json_response::{MeasurementsResponse, UserResponse};
use strong_api_lib::strong_api::{Includes, StrongApi};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let username = env::var("STRONG_USER").expect("STRONG_USER must be set");
    let password = env::var("STRONG_PASS").expect("STRONG_PASS must be set");
    let strong_backend = env::var("STRONG_BACKEND").expect("STRONG_BACKEND must be set");
    let url = Url::parse(&strong_backend).expect("STRONG_BACKEND is not a valid URL");

    let strong_api = StrongApi::new(url);

    /*
    strong_api
        .login(username.as_str(), password.as_str())
        .await?;
    */

    let measurements_response;
    // check if measurements.json file exist, if not, fetch the data from the API
    if !std::path::Path::new("measurements.json").exists() {
        println!("Fetching measurements from API");
        let measurements_response_page1 = strong_api.get_measurements(1).await?;
        let measurements_response_page2 = strong_api.get_measurements(2).await?;
        measurements_response = measurements_response_page1.merge(measurements_response_page2);
        let measurements_json = serde_json::to_string(&measurements_response)?;
        std::fs::write("measurements.json", measurements_json)?;
    } else {
        println!("Reading measurements from file");
        let measurements_json = std::fs::read_to_string("measurements.json")?;
        measurements_response = serde_json::from_str(&measurements_json)?;
    }

    //let user = strong_api.get_user("", 500, vec![Includes::Log]).await?;

    let response_text = std::fs::read_to_string("response.json")?;
    let user: UserResponse = serde_json::from_str(&response_text)?;

    let workouts = DataTransformer
        .get_measurements_from_logs(&user.embedded.log, &Some(measurements_response))
        .expect("Couldn't read workouts");

    println!("Workout count: {}", workouts.len());

    workouts.iter().for_each(|workout| {
        println!("Workout: {}", workout.name);
        workout.exercises.iter().for_each(|exercise| {
            println!("Name: {}", exercise.name);
            exercise.sets.iter().for_each(|set| {
                println!("Set: {:?}", set);
            });
        });
    });

    Ok(())
}