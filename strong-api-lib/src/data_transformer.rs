use crate::json_response::{CellSetGroupLinks, Log, Measurement, MeasurementsResponse};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Set {
    pub id: String,
    pub weight: Option<f32>,
    pub reps: u32,
    pub rpe: Option<f32>,
}

#[derive(Debug)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub sets: Vec<Set>,
}

#[derive(Debug)]
pub struct Workout {
    pub id: String,
    pub name: String,
    pub timezone: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub exercises: Vec<Exercise>,
}

pub struct DataTransformer;

impl DataTransformer {
    pub fn get_measurements_from_logs(
        &self,
        logs_option: &Option<Vec<Log>>,
        measurements_response: &Option<MeasurementsResponse>,
    ) -> Result<Vec<Workout>, serde_json::Error> {
        let logs = match logs_option {
            Some(logs) => logs,
            None => return Ok(Vec::new()),
        };

        //if measurements set, create lookup table
        let mut lookup: HashMap<String, Measurement> = HashMap::new();
        if let Some(measurements) = measurements_response {
            println!(
                "Measurements count: {}/{}",
                measurements.embedded.measurements.len(),
                measurements.total
            );
            for measurement in &measurements.embedded.measurements {
                lookup.insert(measurement.id.clone(), measurement.clone());
            }
        }

        let mut workouts = Vec::new();

        // Process every log.
        for log in logs {
            let workout_id = log.id.clone();
            // Assuming `name` can be converted to a String.
            let workout_name = log.name.clone().unwrap_or_default().to_string();
            let timezone = log.timezone_id.clone();
            let start_date = log.start_date.clone();
            let end_date = log.end_date.clone();

            let mut exercises = Vec::new();

            // Iterate over each cellSetGroup in the log.
            for cell_set_group in &log.embedded.cell_set_group {
                let mut sets = Vec::new();

                // Process each cell set in the group.
                for cell_set in &cell_set_group.cell_sets {
                    // Skip any cell set that represents a rest timer or a note.
                    if !cell_set
                        .cells
                        .iter()
                        .any(|cell| cell.cell_type == "REST_TIMER" || cell.cell_type == "NOTE")
                    {
                        let weight = cell_set
                            .cells
                            .iter()
                            .find(|cell| {
                                cell.cell_type == "OTHER_WEIGHT"
                                    || cell.cell_type == "DUMBBELL_WEIGHT"
                                    || cell.cell_type == "BARBELL_WEIGHT"
                                    || cell.cell_type == "WEIGHTED_BODYWEIGHT"
                            })
                            .and_then(|cell| cell.value.as_ref())
                            .and_then(|s| s.parse::<f32>().ok());

                        let reps = cell_set
                            .cells
                            .iter()
                            .find(|cell| cell.cell_type == "REPS")
                            .and_then(|cell| cell.value.as_ref())
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(0);

                        let rpe = cell_set
                            .cells
                            .iter()
                            .find(|cell| cell.cell_type == "RPE")
                            .and_then(|cell| cell.value.as_ref())
                            .and_then(|s| s.parse::<f32>().ok());

                        let id = cell_set.id.clone();

                        sets.push(Set {
                            id,
                            weight,
                            reps,
                            rpe,
                        });
                    }
                }

                let mut name = String::new();
                let workout_id = DataTransformer::get_workout_id_from_link(&cell_set_group.links);
                // get workout name from measurements if available
                if let Some(measurement) = lookup.get(&workout_id) {
                    name = (measurement.name).to_string();
                }

                // Create an Exercise only if there is at least one valid set.
                if !sets.is_empty() {
                    exercises.push(Exercise {
                        id: cell_set_group.id.clone(),
                        name,
                        sets,
                    });
                }
            }

            workouts.push(Workout {
                id: workout_id,
                exercises,
                name: workout_name,
                timezone,
                start_date,
                end_date,
            });
        }

        Ok(workouts)
    }

    fn get_workout_id_from_link(links: &CellSetGroupLinks) -> String {
        let url = match &links.measurement {
            Some(link) => link.href.clone(),
            None => return String::new(),
        };

        let parts: Vec<&str> = url.split("/").collect();
        parts[parts.len() - 1].to_string()
    }
}
