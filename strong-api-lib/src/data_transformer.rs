use crate::json_response::{CellSet, CellSetGroup, CellSetGroupLinks, Log, Measurement, MeasurementsResponse};
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

pub struct DataTransformer {
    measurements_response: Option<MeasurementsResponse>,
}

impl DataTransformer {
    pub fn new() -> Self {
        Self {
            measurements_response: None,
        }
    }

    pub fn with_measurements_response(
        mut self,
        with_measurements_response: MeasurementsResponse,
    ) -> Self {
        self.measurements_response = Some(with_measurements_response);
        self
    }

    pub fn get_measurements_from_logs(
        &self,
        logs_option: &Option<Vec<Log>>,
    ) -> Result<Vec<Workout>, serde_json::Error> {
        let logs = match logs_option {
            Some(logs) => logs,
            None => return Ok(Vec::new()),
        };

        let lookup = self.create_measurement_lookup();
        let workouts = logs.iter()
            .map(|log| self.process_log_to_workout(log, &lookup))
            .collect();

        Ok(workouts)
    }

    fn create_measurement_lookup(&self) -> HashMap<String, Measurement> {
        let mut lookup: HashMap<String, Measurement> = HashMap::new();

        if let Some(measurements) = &self.measurements_response {
            for measurement in &measurements.embedded.measurements {
                lookup.insert(measurement.id.clone(), measurement.clone());
            }
        }

        lookup
    }

    fn process_log_to_workout(&self, log: &Log, lookup: &HashMap<String, Measurement>) -> Workout {
        let exercises = log
            .embedded
            .cell_set_group
            .iter()
            .filter_map(|cell_set_group| {
                self.process_cell_set_group_to_exercise(cell_set_group, lookup)
            })
            .collect();

        Workout {
            id: log.id.clone(),
            name: log.name.clone().unwrap_or_default().to_string(),
            timezone: log.timezone_id.clone(),
            start_date: log.start_date.clone(),
            end_date: log.end_date.clone(),
            exercises,
        }
    }

    fn process_cell_set_group_to_exercise(
        &self,
        cell_set_group: &CellSetGroup,
        lookup: &HashMap<String, Measurement>,
    ) -> Option<Exercise> {
        let sets: Vec<Set> = cell_set_group
            .cell_sets
            .iter()
            .filter_map(|cell_set| self.process_cell_set_to_set(cell_set))
            .collect();

        if sets.is_empty() {
            return None;
        }

        let workout_id = Self::get_workout_id_from_link(&cell_set_group.links);

        // Get workout name from measurements if available
        let name = lookup
            .get(&workout_id)
            .map(|measurement| measurement.name.to_string())
            .unwrap_or_default();

        Some(Exercise {
            id: cell_set_group.id.clone(),
            name,
            sets,
        })
    }

    fn process_cell_set_to_set(&self, cell_set: &CellSet) -> Option<Set> {
        // Skip rest timers or notes
        if cell_set.cells.iter()
            .any(|cell| matches!(cell.cell_type.as_str(), "REST_TIMER" | "NOTE"))
        {
            return None;
        }

        let weight = cell_set
            .cells
            .iter()
            .find(|cell| {
                matches!(
                    cell.cell_type.as_str(),
                    "OTHER_WEIGHT" | "DUMBBELL_WEIGHT" | "BARBELL_WEIGHT" | "WEIGHTED_BODYWEIGHT"
                )
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

        Some(Set {
            id: cell_set.id.clone(),
            weight,
            reps,
            rpe,
        })
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
