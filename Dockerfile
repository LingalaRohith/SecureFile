# Use the Rust official image
FROM rust:latest

# Install MySQL client for health checks and debugging
RUN apt-get update && apt-get install -y default-mysql-client

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Install Rust dependencies
RUN cargo build --release

# Expose the port your application runs on (if applicable)
EXPOSE 8080

# Set the command to run your application
CMD ["cargo", "run", "--release"]

