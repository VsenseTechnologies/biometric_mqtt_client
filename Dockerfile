#stage 1 - generate the recipe file for dependencies
FROM rust:slim-bullseye AS planner

WORKDIR /app

RUN cargo install cargo-chef

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

#stage 2 - build our dependencies (cooking the food 😁 🦀 🦀)
FROM rust:slim-bullseye AS dependencies_builder

WORKDIR /app

COPY --from=planner /app/recipe.json .
COPY --from=planner /usr/local/cargo /usr/local/cargo

RUN cargo chef cook --release --recipe-path recipe.json


#stage 3 building the source code

FROM rust:slim-bullseye AS final_builder

WORKDIR /app

COPY . .

COPY --from=dependencies_builder /app/target target

RUN cargo build --release

#stage 4 setting up the final runtime for the application

FROM debian:bullseye-slim

WORKDIR /app

ENV MQTT_CLIENT_ID=server_rust

ENV MQTT_HOST=34.47.150.239

ENV MQTT_PORT=1883

ENV DB_URI=postgresql://fingerprint_user:VAYbmJfOZYyJDBDKL1U7BRxv6OoqRR1h@dpg-cr4r595umphs73drro10-a.oregon-postgres.render.com/fingerprint

ENV REDIS_URI=redis://127.0.0.1/

ENV REDIS_ATTENDENCE_LIST_NAME=attendence_logs

ENV REDIS_INSERT_JSON_NAME=inserts

ENV REDIS_DELETE_JSON_NAME=deletes

COPY --from=final_builder /app/target/release/vsense_biometric_mqtt_client ./main

CMD ["./main"]
