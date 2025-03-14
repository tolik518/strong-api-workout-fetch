use crate::user_response::Log;

#[allow(dead_code, unused)]
#[derive(Debug)]
pub struct Set {
    id: String,
    // Each exercise set may have a weight, number of reps, and optional RPE.
    weight: Option<f32>,
    reps: u32,
    rpe: Option<f32>,
}

#[allow(dead_code, unused)]
#[derive(Debug)]
pub struct Exercise {
    pub(crate) id: String,
    pub(crate) sets: Vec<Set>,
}

#[allow(dead_code, unused)]
#[derive(Debug)]
pub struct Workout {
    id: String,
    pub(crate) exercises: Vec<Exercise>,
    pub(crate) name: String,
    timezone: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
}

/// A trait for transforming raw API log data into domain-specific Workouts.
pub trait DataTransformer {
    fn transform(&self, logs: &Option<Vec<Log>>) -> Result<Vec<Workout>, serde_json::Error>;
}

pub(crate) struct DataTransformerImpl;

impl DataTransformer for DataTransformerImpl {
    fn transform(&self, logs_option: &Option<Vec<Log>>) -> Result<Vec<Workout>, serde_json::Error> {
        let mut logs = match logs_option {
            Some(logs) => logs,
            None => return Ok(Vec::new()),
        };

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

                // Create an Exercise only if there is at least one valid set.
                if !sets.is_empty() {
                    exercises.push(Exercise {
                        id: cell_set_group.id.clone(),
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
}
