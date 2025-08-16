FROM rust:bookworm AS builder
WORKDIR /usr/src/strong-api-fetch
COPY . .
RUN cd strong-api-fetch && RUSTFLAGS="-C debuginfo=2" cargo install --path . --debug

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    cron

WORKDIR /usr/strong-api-fetch
COPY --from=builder /usr/local/cargo/bin/strong-api-fetch /usr/bin/strong-api-fetch
COPY .env /.env
# will run the cron job every day at 18:00, 18:30, 19:00, 19:30, 20:00, and 20:30
RUN echo "0,30 02-03 * * * root RUST_BACKTRACE=1 RUST_LOG=debug /usr/bin/strong-api-fetch >> /var/log/cron.log 2>&1" > /etc/cron.d/strong-api-fetch

# Ensure the cron job file has proper permissions
RUN chmod 0644 /etc/cron.d/strong-api-fetch && \
    chmod +x /usr/bin/strong-api-fetch

# Create the log file so that it exists when cron writes to it
RUN touch /var/log/cron.log

COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

CMD ["/docker-entrypoint.sh"]