use clickhouse::Row;
use chrono::NaiveDateTime;
use std::error::Error;
use strong_api_lib::data_transformer::Workout;

/// This flattened struct represents one set with its workout and exercise context.
#[derive(Row, Debug)]
pub struct WorkoutSet {
    pub workout_id: String,
    pub workout_name: String,
    pub timezone: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub exercise_id: String,
    pub exercise_name: String,
    pub set_id: String,
    pub weight: Option<f32>,
    pub reps: u32,
    pub rpe: Option<f32>,
}

pub struct ClickHouseSaver {
    client: clickhouse::Client,
    table_name: String,
}

impl ClickHouseSaver {
    pub fn new(url: &str, username: &str, password: &str, table_name: &str) -> Self {
        Self {
            client: clickhouse::Client::default()
                .with_url(url)
                .with_user(username)
                .with_password(password),
            table_name: table_name.to_string(),
        }
    }

    /// Saves a given workout into ClickHouse by flattening its nested data into rows.
    ///
    /// # Arguments
    ///
    /// * `workout` - A reference to the Workout struct.
    ///
    /// # Returns
    ///
    /// A Result indicating success or any error encountered.
    pub async fn save_workout(&self, workout: &Workout) -> Result<(), Box<dyn Error>> {
        let mut rows = Vec::new();

        // Flatten the data from the nested Workout structure.
        for exercise in &workout.exercises {
            for set in &exercise.sets {
                let start_dt = workout.start_date.as_ref()
                    .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());
                let end_dt = workout.end_date.as_ref()
                    .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());

                rows.push(WorkoutSet {
                    workout_id: workout.id.clone(),
                    workout_name: workout.name.clone(),
                    timezone: workout.timezone.clone(),
                    start_date: start_dt,
                    end_date: end_dt,
                    exercise_id: exercise.id.clone(),
                    exercise_name: exercise.name.clone(),
                    set_id: set.id.clone(),
                    weight: set.weight,
                    reps: set.reps,
                    rpe: set.rpe,
                });
            }
        }

        let mut inserter = self.client.insert(&self.table_name).await?;
        inserter.write(&rows).await?;
        inserter.end().await?;

        println!("Workout data inserted successfully!");
        Ok(())
    }
}