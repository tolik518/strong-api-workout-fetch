# strong-api-lib

Rust client library for the [Strong App](https://www.strong.app/) backend API. Fetches workout logs and exercise definitions, then transforms them into a clean `Workout → Exercise → Set` model.

## Getting Started

```toml
[dependencies]
strong-api-lib = { path = "../strong-api-lib" }
tokio = { version = "1", features = ["full"] }
```

## Usage

```rust
use reqwest::Url;
use strong_api_lib::strong_api::{StrongApi, Includes};
use strong_api_lib::data_transformer::DataTransformer;

let url = Url::parse("https://your-strong-backend.example.com").unwrap();
let mut api = StrongApi::new(url);

// Authenticate
api.login("user@example.com", "password").await?;

// Refresh token (uses internally stored tokens)
api.refresh().await?;

// With the "full" feature, you can pass tokens explicitly:
// api.refresh_by_tokens(access_token, refresh_token).await?;

// Fetch exercise definitions (paginated)
let page1 = api.get_measurements(1).await?;
let page2 = api.get_measurements(2).await?;
let measurements = page1.merge(page2);

// Fetch user data with workout logs
let user = api.get_user("", 500, vec![Includes::Log]).await?;

// Transform into domain model
let transformer = DataTransformer::new()
    .with_measurements_response(measurements);

let workouts = transformer
    .get_measurements_from_logs(&user.embedded.log)?;

for workout in &workouts {
    for exercise in &workout.exercises {
        for set in &exercise.sets {
            println!("{} — {} × {:.1} kg",
                exercise.name, set.reps, set.weight.unwrap_or(0.0));
        }
    }
}
```

## `Includes` Variants

Used with `get_user()` to select which embedded resources to return:

`Log`, `Measurement`, `Tag`, `Widget`, `Template`, `Folder`, `MeasuredValue`

## Domain Model

```
Workout
├── id, name, timezone, start_date, end_date
└── exercises: Vec<Exercise>
      ├── id, name
      └── sets: Vec<Set>
            └── id, weight, reps, rpe
```

## Features

| Feature | Description |
|---|---|
| `full` | Enables `refresh_by_tokens()` — refresh auth with externally persisted tokens |

## API Endpoints

| Method | Path | Function |
|---|---|---|
| `POST` | `auth/login` | `login()` |
| `POST` | `auth/login/refresh` | `refresh()` / `refresh_by_tokens()` |
| `GET` | `api/users/{user_id}` | `get_user()` |
| `GET` | `api/measurements?page={n}` | `get_measurements()` |
| `GET` | `api/logs/{user_id}` | `get_logs_raw()` |
