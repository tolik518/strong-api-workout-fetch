CREATE DATABASE IF NOT EXISTS workouts;
USE workouts;

CREATE TABLE workout_sets
(
    workout_id    UUID,
    workout_name  String,
    timezone      String DEFAULT 'Europe/Berlin',
    start_date    DateTime64(3) DEFAULT now(),
    end_date      DateTime64(3) DEFAULT now(),
    exercise_id   UUID,
    exercise_name String,
    set_id        UUID,
    weight        Float32 DEFAULT 0.0,
    reps          UInt32,
    rpe           Float32 DEFAULT 0.0
)
    ENGINE = ReplacingMergeTree()
ORDER BY (start_date, workout_id, exercise_id, set_id);
