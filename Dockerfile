FROM rust:1.62 AS base
ENV SQLX_OFFLINE true
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

# -------------------------------------
FROM base AS dev
RUN apt-get install -y libssl-dev
# RUN apt-get update && apt-get -y install cmake protobuf-compiler
RUN cargo install cargo-watch
WORKDIR /usr/src/app
COPY . .

FROM base AS builder
ADD . /twitar
WORKDIR /twitar
RUN cargo build --release -p twitar
# RUN cargo install sqlx-cli
# RUN cargo sqlx prepare --merged

FROM debian:buster-slim as debian
ARG APP=/usr/src/app
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
ENV TZ=Etc/UTC \
    APP_USER=appuser
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}


FROM debian AS prod
WORKDIR /usr/src/app
COPY --from=builder /twitar/sqlx-data.json ${APP}/sqlx-data.json
COPY --from=builder /twitar/Cargo.toml ${APP}/Cargo.toml
COPY --from=builder /twitar/target/release/twitar ${APP}/twitar
RUN echo $APP_USER
RUN echo $APP
COPY --from=builder /twitar/configuration ${APP}/configuration
RUN chown -R $APP_USER:$APP_USER $APP
USER $APP_USER
WORKDIR ${APP}
EXPOSE ${APP_PORT}
CMD ["cargo", "watch", "-x", "run", "-p", "twitar"]