FROM lukemathwalker/cargo-chef:latest-rust-1.78.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Build our project
RUN cargo build --release --bin mc-control

FROM alpine:3.19.1 as runtime
RUN apk add kubectl
COPY --from=builder /app/target/release/mc-control mc-control
ENTRYPOINT ["./mc-control"]
