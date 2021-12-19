FROM debian:stretch-slim AS builder

RUN apt update && apt install -y musl-tools musl-dev openssl libssl-dev pkg-config curl

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > script.sh
RUN chmod +x script.sh
RUN ./script.sh -y
ENV PATH $PATH:$HOME/.cargo/bin

RUN /root/.cargo/bin/rustup update stable

WORKDIR /app

ADD src/ /app/src
ADD Cargo.toml /app
ADD Cargo.lock /app

RUN update-ca-certificates

# Create appuser
ENV USER=worker
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN /root/.cargo/bin/cargo clean
RUN /root/.cargo/bin/cargo build --release

FROM debian:stable-slim

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/timhatdiehandandermaus /app/timhatdiehandandermaus
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

ENV TZ Europe/Berlin

# Use an unprivileged user.
USER worker:worker

CMD ["/app/timhatdiehandandermaus"]
