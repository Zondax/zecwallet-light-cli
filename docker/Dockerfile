FROM ubuntu:20.04
LABEL Description="Rust compile env for Linux + Windows (cross)"

ARG DEBIAN_FRONTEND=noninteractive

RUN apt update -y
RUN apt install -y build-essential mingw-w64 gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf \
                    curl vim wget git unzip \
                    udev coreutils libudev-dev libusb-1.0-0-dev pkg-config

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- --default-toolchain none -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup toolchain install 1.66

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add armv7-unknown-linux-gnueabihf

# Append the linker to the cargo config for Windows cross compile
RUN echo "[target.x86_64-pc-windows-gnu]" >> /root/.cargo/config && \
    echo "linker = '/usr/bin/x86_64-w64-mingw32-gcc'" >> /root/.cargo/config

RUN echo "[target.aarch64-unknown-linux-gnu]" >> /root/.cargo/config && \
    echo "linker = '/usr/bin/aarch64-linux-gnu-gcc'" >> /root/.cargo/config

RUN echo "[target.armv7-unknown-linux-gnueabihf]" >> /root/.cargo/config && \
    echo "linker = '/usr/bin/arm-linux-gnueabihf-gcc'" >> /root/.cargo/config

ENV CC_x86_64_unknown_linux_musl="gcc" 
ENV CC_aarch64_unknown_linux_gnu="aarch64-linux-gnu-gcc" 
ENV CC_armv7_unknown_linux_gnueabhihf="arm-linux-gnueabihf-gcc"

# Install protoc
RUN cd /tmp && wget https://github.com/protocolbuffers/protobuf/releases/download/v21.2/protoc-21.2-linux-x86_64.zip && \
    unzip protoc-21.2-linux-x86_64.zip -d /usr/

# # For windows cross compilation, use a pre-build binary. Remember to set the
# # SODIUM_LIB_DIR for windows cross compilation
# RUN cd /opt && wget https://download.libsodium.org/libsodium/releases/libsodium-1.0.17-mingw.tar.gz && \
#     tar xvf libsodium-1.0.17-mingw.tar.gz

# # Cargo fetch the dependencies so we don't download them over and over again
# RUN cd /tmp && git clone --depth=1 https://github.com/Zondax/zecwallet-light-cli.git && \
#     cd zecwallet-light-cli && \
#     cargo fetch && \
#     cd /tmp && rm -rf zecwallet-light-cli

# This is a bug fix for the windows cross compiler for Rust.
# RUN cp /usr/x86_64-w64-mingw32/lib/crt2.o  /root/.rustup/toolchains/1.49.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-pc-windows-gnu/crt2.o

