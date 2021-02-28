FROM rust:1.50 as builder

# little caching trick - edit: does this work lol
RUN USER=root cargo new --bin league-a-lot
WORKDIR ./league-a-lot
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/league_a_lot*
RUN cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

ENV TZ=Etc/UTC \
    APP_USER=loluser \
    ROCKET_ADDRESS=0.0.0.0

#RUN groupadd $APP_USER \
    #&& useradd -g $APP_USER $APP_USER \
    #&& mkdir -p ${APP}

COPY --from=builder /league-a-lot/target/release/league-a-lot ${APP}/league-a-lot

#RUN chown -R $APP_USER:$APP_USER ${APP}

#USER $APP_USER
WORKDIR ${APP}

CMD ["./league-a-lot"]
