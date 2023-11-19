VERSION 0.7

builder:
    FROM DOCKERFILE ./docker

src:
    FROM +builder
    COPY --dir . /opt/zecwallet-light-cli
    WORKDIR /opt/zecwallet-light-cli

linux:
    FROM +src
    RUN cargo build --release
    SAVE ARTIFACT target/release/zecwallet-cli

win:
    FROM +src
    ENV SODIUM_LIB_DIR /opt/libsodium-win64/lib
    RUN cargo build --release --target x86_64-pc-windows-gnu
    SAVE ARTIFACT target/x86_64-pc-windows-gnu/release/zecwallet-cli.exe
