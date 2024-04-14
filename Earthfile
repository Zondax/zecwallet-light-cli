VERSION 0.7
ARG --global APP_VERSION=1.8.0

builder:
    FROM DOCKERFILE ./docker

src:
    FROM +builder
    WORKDIR /opt/zecwallet-light-cli

    # We copy over only the bare minimum files for dependency resolution
    COPY Cargo.* rust-toolchain ./
    COPY lib/Cargo.toml ./lib/
    COPY lib/src/lib.rs ./lib/src/
    COPY cli/Cargo.toml ./cli/
    COPY cli/src/main.rs ./cli/src/

    # We also change `CARGO_HOME` to a new folder which we can cache
    ENV CARGO_HOME ./.cargo
    CACHE ./.cargo
    # And get all the required dependencies
    RUN cargo fetch

    # Then, we copy over all the sources, this way we don't need to update
    # the depedencies when we just update the sources
    COPY --dir lib cli ./

linux:
    FROM +src
    RUN cargo build --release
    SAVE ARTIFACT target/release/zecwallet-cli AS LOCAL build/zecwallet-cli-v$APP_VERSION

win:
    FROM +src

    # FIXME: replace libsodium with crypto_secretbox
    RUN cd /opt && wget https://web.archive.org/web/20220529105948if_/https://download.libsodium.org/libsodium/releases/libsodium-1.0.17-mingw.tar.gz \
        && tar xvf libsodium-1.0.17-mingw.tar.gz
    ENV SODIUM_LIB_DIR /opt/libsodium-win64/lib

    RUN cargo build --release --target x86_64-pc-windows-gnu
    SAVE ARTIFACT target/x86_64-pc-windows-gnu/release/zecwallet-cli.exe AS LOCAL build/zecwallet-cli-v$APP_VERSION.exe
