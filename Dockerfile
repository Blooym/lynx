# ----------
#   SETUP
# ----------
FROM alpine:latest AS setup
RUN adduser -S -s /bin/false -D lynx
RUN mkdir /dir

# -----------
#    BUILD
# -----------
FROM rust:1-alpine AS build
WORKDIR /build
RUN apk add --no-cache --update build-base

# Pre-cache dependencies
COPY ["Cargo.toml", "Cargo.lock", "./"]
RUN mkdir src \
    && echo "// Placeholder" > src/lib.rs \
    && cargo build --release \
    && rm src/lib.rs

# Build
COPY src ./src
RUN cargo build --release

# -----------
#   RUNTIME
# -----------
FROM scratch
WORKDIR /opt

COPY --from=build /build/target/release/lynx /usr/bin/lynx

# Setup deployment image.
COPY --from=setup /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=setup /etc/passwd /etc/passwd
COPY --from=setup /bin/false /bin/false
USER lynx
COPY --from=setup --chown=lynx /dir /srv/lynx

# Set configuration defaults for container builds.
ENV LYNX_ADDRESS=0.0.0.0:5621
ENV LYNX_CONFIG=/etc/lynx/config.toml
ENV RUST_LOG=info
EXPOSE 5621

ENTRYPOINT ["/usr/bin/lynx"]
