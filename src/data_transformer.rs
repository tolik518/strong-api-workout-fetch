use crate::user_response::CellSetGroup;

#[derive(Debug)]
pub struct Set {
    id: String,
    // We assume each exercise set may have a weight, number of reps, and optional RPE.
    weight: Option<f32>,
    reps: u32,
    rpe: Option<f32>,
}

#[derive(Debug)]
pub struct Workout {
    id: String,
    sets: Vec<Set>,
}


pub trait DataTransformer {
    fn transform(&self, json_data: &CellSetGroup) -> Result<Workout, serde_json::Error>;
}

pub(crate) struct DataTransformerImpl;

impl DataTransformer for DataTransformerImpl {
    fn transform(&self, raw_workout: &CellSetGroup) -> Result<Workout, serde_json::Error> {
        let mut exercises = Vec::new();

        // Process each cell set.
        for cell_set in &raw_workout.cell_sets {
            if !(cell_set.cells.iter().any(|cell| cell.cell_type == "REST_TIMER" || cell.cell_type == "NOTE")) {
                let weight = cell_set
                    .cells
                    .iter()
                    .find(|cell|
                        cell.cell_type == "OTHER_WEIGHT" ||
                        cell.cell_type == "DUMBBELL_WEIGHT" ||
                        cell.cell_type == "BARBELL_WEIGHT" ||
                        cell.cell_type == "WEIGHTED_BODYWEIGHT"
                    )
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

                exercises.push(Set { id, weight, reps, rpe });
            }
        }

        Ok(Workout {
            id: raw_workout.clone().id,
            sets: exercises,
        })
    }
}