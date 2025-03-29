# Strong API data fetch

This repo provides access to the Strong API to save workouts to your own database.
The URL to the API backend is not provided here, but you can find it through the Strong app.

## How to use

1. Clone this repo
2. Create .env file in the root directory with the following content:
    ```ini
    STRONG_BACKEND=https://strong_backend_url.com
    
    STRONG_USER=your_strong_user
    STRONG_PASS=your_strong_pass

    CLICKHOUSE_HOST=clickhouse
    CLICKHOUSE_USER=default
    CLICKHOUSE_PASS=
    CLICKHOUSE_TABLE=strong
    ```
3. Run with `cargo run`