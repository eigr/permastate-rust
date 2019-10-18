# -*- mode: dockerfile -*-
#
# An example Dockerfile showing how to build a Rust executable using this
# image, and deploy it with a tiny Alpine Linux container.

# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust or OpenSSL.
ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

RUN sudo apt-get update && sudo apt-get install -y upx-ucl

# Add our source code.
ADD . ./

# Fix permissions on source code.
RUN sudo chown -R rust:rust /home/rust

# Build our application.
RUN cargo build --release

RUN /usr/bin/upx --brute /home/rust/src/target/x86_64-unknown-linux-musl/release/cloudstate

# Now, we need to build our _real_ Docker container, copying in `using-diesel`.
FROM scratch
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/cloudstate \
    /usr/local/bin/

CMD ["/usr/local/bin/cloudstate"]

