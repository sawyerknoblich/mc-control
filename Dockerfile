FROM lukemathwalker/cargo-chef:latest-rust-1.78.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY /server .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as server_builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
COPY /server .
# Build our project
RUN cargo build --release --bin mc-control

FROM node:22.1.0-alpine3.18 as ui_builder
WORKDIR /app
COPY /ui .
RUN npm ci
RUN npm run build

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update -y && \
    apt-get install -y apt-transport-https ca-certificates curl gnupg2
RUN curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.30/deb/Release.key | \
    gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
RUN chmod 644 /etc/apt/keyrings/kubernetes-apt-keyring.gpg
RUN echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.30/deb/ /' >/etc/apt/sources.list.d/kubernetes.list
RUN chmod 644 /etc/apt/sources.list.d/kubernetes.list
RUN apt-get update && \
    apt-get install -y kubectl

COPY --from=server_builder /app/target/release/mc-control mc-control
COPY --from=ui_builder /app/dist/ ui/
ENTRYPOINT ["./mc-control"]
