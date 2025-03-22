CREATE DATABASE IF NOT EXISTS workouts;
USE workouts;

CREATE TABLE workout_sets
(
    `workout_id` String,
    `workout_name` String,
    `timezone` Nullable(String),
    `start_date` Nullable(DateTime),
    `end_date` Nullable(DateTime),
    `exercise_id` String,
    `exercise_name` String,
    `set_id` String,
    `weight` Nullable(Float32),
    `reps` UInt32,
    `rpe` Nullable(Float32)
)
    ENGINE = MergeTree
ORDER BY ifNull(start_date, toDateTime('1970-01-01 00:00:00'))
