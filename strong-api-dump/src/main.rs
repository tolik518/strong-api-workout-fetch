mod clickhouse_saver;

use dotenv::dotenv;
use reqwest::Url;
use std::env;
use strong_api_lib::data_transformer::DataTransformer;
use strong_api_lib::json_response::UserResponse;
use strong_api_lib::strong_api::{Includes, StrongApi};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    #[allow(unused_variables)]
    let username = env::var("STRONG_USER").expect("STRONG_USER must be set");
    #[allow(unused_variables)]
    let password = env::var("STRONG_PASS").expect("STRONG_PASS must be set");
    let strong_backend = env::var("STRONG_BACKEND").expect("STRONG_BACKEND must be set");
    let url = Url::parse(&strong_backend).expect("STRONG_BACKEND is not a valid URL");

    let start = std::time::Instant::now();

    let mut strong_api = StrongApi::new(url);
    let mut clickhouse_saver = clickhouse_saver::ClickHouseSaver::new(
        env::var("CLICKHOUSE_URL").expect("CLICKHOUSE_URL must be set").as_str(),
        env::var("CLICKHOUSE_PASS").expect("CLICKHOUSE_PASS must be set").as_str(),
        env::var("CLICKHOUSE_USER").expect("CLICKHOUSE_USER must be set").as_str(),
        env::var("CLICKHOUSE_TABLE").expect("CLICKHOUSE_TABLE must be set").as_str(),
    );

    strong_api
        .login(username.as_str(), password.as_str())
        .await?;

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

    let user = strong_api.get_user("", 500, vec![Includes::Log]).await?;

    //let response_text = std::fs::read_to_string("response.json")?;
    //let user: UserResponse = serde_json::from_str(&response_text)?;

    println!(
        "Measurements count: {}/{}",
        &measurements_response.embedded.measurements.len(),
        &measurements_response.total
    );

    let data_transformer = DataTransformer::new().with_measurements_response(measurements_response);

    let workouts = data_transformer
        .get_measurements_from_logs(&user.embedded.log)
        .expect("Couldn't read workouts");

    println!("Workout count: {}", workouts.len());

    workouts.iter().for_each(|workout| {
        println!("Workout: {}", workout.name);
        println!("Date: {:?}", workout.start_date);
        workout.exercises.iter().for_each(|exercise| {
            println!("Name: {}", exercise.name);
            exercise.sets.iter().for_each(|set| {
                println!("Set: {:?}", set);
            });
        });
    });

    let end = start.elapsed();

    println!("Time elapsed: {:?}", end);

    Ok(())
}
