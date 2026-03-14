use strong_api_lib::data_transformer::DataTransformer;
use strong_api_lib::models::measurement::MeasurementsResponse;
use strong_api_lib::models::workout::UserResponse;

fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    ))
    .unwrap_or_else(|_| panic!("fixture '{name}' not found"))
}

fn measurements_from_fixture() -> MeasurementsResponse {
    let json = load_fixture("measurements_response.json");
    serde_json::from_str(&json).unwrap()
}

fn logs_from_fixture() -> Option<Vec<strong_api_lib::models::workout::Log>> {
    let json = load_fixture("user_response.json");
    let user: UserResponse = serde_json::from_str(&json).unwrap();
    user.embedded.log
}

// ---------------------------------------------------------------------------
// Basic transformation — no measurements loaded, names fall back to empty
// ---------------------------------------------------------------------------

#[test]
fn test_transform_without_measurements_produces_workouts() {
    let transformer = DataTransformer::new();
    let logs = logs_from_fixture();
    let workouts = transformer.get_measurements_from_logs(&logs).unwrap();

    assert!(!workouts.is_empty(), "should produce at least one workout");
    // Without a measurement lookup every exercise name is empty string
    for exercise in &workouts[0].exercises {
        assert_eq!(exercise.name, "");
    }
}

// ---------------------------------------------------------------------------
// With measurements: names are resolved and exercises are non-empty
// ---------------------------------------------------------------------------

#[test]
fn test_transform_with_measurements_resolves_names() {
    let transformer =
        DataTransformer::new().with_measurements_response(measurements_from_fixture());
    let logs = logs_from_fixture();
    let workouts = transformer.get_measurements_from_logs(&logs).unwrap();

    let exercises = &workouts[0].exercises;
    assert!(!exercises.is_empty(), "should have at least one exercise");

    // Every exercise whose measurement was found in the lookup has a non-empty name.
    let named = exercises.iter().filter(|e| !e.name.is_empty()).count();
    assert!(
        named > 0,
        "at least one exercise should have a resolved name"
    );
}

// ---------------------------------------------------------------------------
// Exercises come only from groups that have at least one valid (non-rest) set
// ---------------------------------------------------------------------------

#[test]
fn test_only_groups_with_real_sets_become_exercises() {
    let transformer =
        DataTransformer::new().with_measurements_response(measurements_from_fixture());
    let logs = logs_from_fixture();
    let workouts = transformer.get_measurements_from_logs(&logs).unwrap();

    // Count CSGs in the raw fixture log
    let json = load_fixture("user_response.json");
    let user: UserResponse = serde_json::from_str(&json).unwrap();
    let raw_csg_count = user.embedded.log.unwrap()[0].embedded.cell_set_group.len();

    // Transformed exercise count can be at most the number of raw CSGs
    assert!(workouts[0].exercises.len() <= raw_csg_count);
}

// ---------------------------------------------------------------------------
// Sets: weight and reps fields are always present on every set
// ---------------------------------------------------------------------------

#[test]
fn test_sets_have_weight_and_reps() {
    let transformer =
        DataTransformer::new().with_measurements_response(measurements_from_fixture());
    let logs = logs_from_fixture();
    let workouts = transformer.get_measurements_from_logs(&logs).unwrap();

    let exercises = &workouts[0].exercises;
    for exercise in exercises {
        assert!(
            !exercise.sets.is_empty(),
            "exercise {} should have sets",
            exercise.id
        );
        for set in &exercise.sets {
            let _ = set.reps; // always present
            let _ = set.weight; // optional (None for bodyweight)
        }
    }
}

// ---------------------------------------------------------------------------
// Workout metadata is preserved
// ---------------------------------------------------------------------------

#[test]
fn test_workout_metadata_is_present() {
    let transformer = DataTransformer::new();
    let logs = logs_from_fixture();
    let workouts = transformer.get_measurements_from_logs(&logs).unwrap();

    let workout = &workouts[0];
    assert!(!workout.id.is_empty());
    assert!(workout.timezone.is_some(), "timezone should be present");
    assert!(workout.start_date.is_some(), "start_date should be present");
    assert!(workout.end_date.is_some(), "end_date should be present");
}

// ---------------------------------------------------------------------------
// Measurement ID extraction from link (replaces former inline unit tests)
// Tested indirectly through the public API: if the ID is extracted correctly
// the name lookup succeeds; if the link is absent the name is empty.
// ---------------------------------------------------------------------------

fn make_log_with_measurement_link(
    measurement_href: Option<&str>,
) -> Vec<strong_api_lib::models::workout::Log> {
    use serde_json::json;
    use strong_api_lib::models::common::Link;
    use strong_api_lib::models::workout::{
        Cell, CellSet, CellSetGroup, CellSetGroupEmbedded, CellSetGroupLinks, Log, LogEmbedded,
    };

    vec![Log {
        id: "log-link-test".to_string(),
        embedded: LogEmbedded {
            cell_set_group: vec![CellSetGroup {
                id: "csg-link-test".to_string(),
                links: CellSetGroupLinks {
                    measurement: measurement_href.map(|href| Link {
                        href: href.to_string(),
                    }),
                },
                embedded: CellSetGroupEmbedded {},
                cell_sets: vec![CellSet {
                    id: "cs-link-test".to_string(),
                    is_completed: Some(true),
                    cells: vec![
                        Cell {
                            id: "c1".to_string(),
                            cell_type: "BARBELL_WEIGHT".to_string(),
                            value: Some("80".to_string()),
                        },
                        Cell {
                            id: "c2".to_string(),
                            cell_type: "REPS".to_string(),
                            value: Some("5".to_string()),
                        },
                    ],
                }],
            }],
        },
        links: json!({}),
        timezone_id: None,
        created: "2024-01-01T00:00:00Z".to_string(),
        last_changed: "2024-01-01T00:00:00Z".to_string(),
        name: None,
        access: "private".to_string(),
        start_date: None,
        end_date: None,
        log_type: "WORKOUT".to_string(),
    }]
}

#[test]
fn test_measurement_id_extracted_from_link_resolves_name() {
    // Use a known measurement ID from the fixture
    let measurements = measurements_from_fixture();
    let known = &measurements.embedded.measurements[0];
    let known_id = known.id.clone();
    let expected_name = known.name.to_string();

    let href = format!("/api/users/00000000-0000-0000-0000-000000000001/measurements/{known_id}");
    let logs = make_log_with_measurement_link(Some(&href));

    let transformer = DataTransformer::new().with_measurements_response(measurements);
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();

    assert_eq!(workouts[0].exercises[0].name, expected_name);
}

#[test]
fn test_missing_measurement_link_gives_empty_name() {
    let logs = make_log_with_measurement_link(None);

    let transformer =
        DataTransformer::new().with_measurements_response(measurements_from_fixture());
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();

    assert_eq!(workouts[0].exercises[0].name, "");
}

// ---------------------------------------------------------------------------
// Cell type coverage — NOTE filter, all weight variants, RPE, missing REPS
// ---------------------------------------------------------------------------

fn make_log_with_cells(
    cells: Vec<(String, Option<String>)>,
) -> Vec<strong_api_lib::models::workout::Log> {
    use serde_json::json;
    use strong_api_lib::models::workout::{
        Cell, CellSet, CellSetGroup, CellSetGroupEmbedded, CellSetGroupLinks, Log, LogEmbedded,
    };

    let cells: Vec<Cell> = cells
        .into_iter()
        .enumerate()
        .map(|(i, (cell_type, value))| Cell {
            id: format!("c{i}"),
            cell_type,
            value,
        })
        .collect();

    vec![Log {
        id: "log-cell-test".to_string(),
        embedded: LogEmbedded {
            cell_set_group: vec![CellSetGroup {
                id: "csg-cell-test".to_string(),
                links: CellSetGroupLinks { measurement: None },
                embedded: CellSetGroupEmbedded {},
                cell_sets: vec![CellSet {
                    id: "cs-cell-test".to_string(),
                    is_completed: Some(true),
                    cells,
                }],
            }],
        },
        links: json!({}),
        timezone_id: None,
        created: "2024-01-01T00:00:00Z".to_string(),
        last_changed: "2024-01-01T00:00:00Z".to_string(),
        name: None,
        access: "private".to_string(),
        start_date: None,
        end_date: None,
        log_type: "WORKOUT".to_string(),
    }]
}

#[test]
fn test_note_cell_type_is_excluded() {
    let logs = make_log_with_cells(vec![
        ("NOTE".to_string(), Some("Good session".to_string())),
        ("REPS".to_string(), Some("10".to_string())),
    ]);
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();
    assert!(
        workouts[0].exercises.is_empty(),
        "NOTE cell should cause the group to be filtered out"
    );
}

#[test]
fn test_dumbbell_weight_cell_type() {
    let logs = make_log_with_cells(vec![
        ("DUMBBELL_WEIGHT".to_string(), Some("20".to_string())),
        ("REPS".to_string(), Some("12".to_string())),
    ]);
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();
    assert_eq!(workouts[0].exercises[0].sets[0].weight, Some(20.0));
}

#[test]
fn test_other_weight_cell_type() {
    let logs = make_log_with_cells(vec![
        ("OTHER_WEIGHT".to_string(), Some("15.5".to_string())),
        ("REPS".to_string(), Some("8".to_string())),
    ]);
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();
    assert_eq!(workouts[0].exercises[0].sets[0].weight, Some(15.5));
}

#[test]
fn test_weighted_bodyweight_cell_type() {
    let logs = make_log_with_cells(vec![
        ("WEIGHTED_BODYWEIGHT".to_string(), Some("10".to_string())),
        ("REPS".to_string(), Some("15".to_string())),
    ]);
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();
    assert_eq!(workouts[0].exercises[0].sets[0].weight, Some(10.0));
}

#[test]
fn test_no_weight_cell_gives_none() {
    // Only REPS, no weight cell at all
    let logs = make_log_with_cells(vec![("REPS".to_string(), Some("10".to_string()))]);
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();
    assert_eq!(workouts[0].exercises[0].sets[0].weight, None);
}

#[test]
fn test_rpe_cell_type() {
    let logs = make_log_with_cells(vec![
        ("BARBELL_WEIGHT".to_string(), Some("100".to_string())),
        ("REPS".to_string(), Some("5".to_string())),
        ("RPE".to_string(), Some("9".to_string())),
    ]);
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();
    assert_eq!(workouts[0].exercises[0].sets[0].rpe, Some(9.0));
}

#[test]
fn test_missing_reps_value_defaults_to_zero() {
    let logs = make_log_with_cells(vec![
        ("BARBELL_WEIGHT".to_string(), Some("60".to_string())),
        ("REPS".to_string(), None), // value is None
    ]);
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&Some(logs)).unwrap();
    assert_eq!(workouts[0].exercises[0].sets[0].reps, 0);
}

#[test]
fn test_empty_logs_vec_returns_empty_workouts() {
    let transformer = DataTransformer::new();
    let workouts = transformer
        .get_measurements_from_logs(&Some(vec![]))
        .unwrap();
    assert!(workouts.is_empty());
}

#[test]
fn test_logs_option_none_returns_empty_workouts() {
    let transformer = DataTransformer::new();
    let workouts = transformer.get_measurements_from_logs(&None).unwrap();
    assert!(workouts.is_empty());
}

#[test]
fn test_data_transformer_default_equals_new() {
    // Exercises the Default impl (derived via `impl Default for DataTransformer`)
    let by_default: DataTransformer = Default::default();
    let workouts = by_default.get_measurements_from_logs(&None).unwrap();
    assert!(workouts.is_empty());
}
