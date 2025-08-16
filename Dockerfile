FROM rust:bookworm AS builder
WORKDIR /usr/src/strong-api-fetch
COPY . .
RUN cd strong-api-fetch && cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    cron

WORKDIR /usr/strong-api-fetch
COPY --from=builder /usr/local/cargo/bin/strong-api-fetch /usr/bin/strong-api-fetch

# will run the cron job every day at 18:00, 18:30, 19:00, 19:30, 20:00, and 20:30
RUN echo "0,30 18-20 * * * root /usr/bin/strong-api-fetch >> /var/log/cron.log 2>&1" > /etc/cron.d/strong-api-fetch

# Ensure the cron job file has proper permissions
RUN chmod 0644 /etc/cron.d/strong-api-fetch && \
    chmod +x /usr/bin/strong-api-fetch

# Install the new cron job
RUN crontab /etc/cron.d/strong-api-fetch

# Create the log file so that it exists when cron writes to it
RUN touch /var/log/cron.log

# Run cron in the foreground
CMD ["cron", "-f"]