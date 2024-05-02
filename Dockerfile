FROM rust:latest

# Set the working directory in the container to /my
WORKDIR /usr/src/my-app

# Copy the Rust project files to the working directory
COPY . .

# Build the Rust app
RUN cargo build
RUN cargo install diesel_cli --no-default-features --features postgres
# Set the command to run the Rust app
ENTRYPOINT ["/bin/bash", "-c", "./wait-for-it.sh db:5432 -q -- diesel setup && cargo run --release"]

