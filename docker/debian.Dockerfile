ARG DEBIAN_VERSION=bookworm
FROM debian:${DEBIAN_VERSION}

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswscale-dev \
    libswresample-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-editing-services1.0-dev \
    libgtk-3-dev \
    libx11-dev \
    libxrandr-dev \
    libxinerama-dev \
    libxcursor-dev \
    libxtst-dev \
    libxi-dev \
    nodejs \
    npm \
    curl \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install tauri-cli --version ^2.0

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src-tauri/ ./src-tauri/

COPY package*.json ./
COPY src/ ./src/
COPY next.config.mjs tsconfig.json ./

RUN npm ci --only=production

RUN npm run build

RUN cargo tauri build --target x86_64-unknown-linux-gnu

FROM debian:${DEBIAN_VERSION}

RUN apt-get update && apt-get install -y \
    libgcc \
    libstdc++6 \
    libavcodec58 \
    libavformat58 \
    libavutil56 \
    libswscale5 \
    libswresample3 \
    libgstreamer1.0-0 \
    libgstreamer-plugins-base1.0-0 \
    libgstreamer-plugins-good1.0-0 \
    libgstreamer-plugins-ugly1.0-0 \
    libgstreamer-plugins-bad1.0-0 \
    libgstreamer-editing-services1.0-0 \
    libgtk-3-0 \
    libgail-3-0 \
    libgdk-pixbuf2.0-0 \
    libx11-6 \
    libxrandr2 \
    libxinerama1 \
    libxcursor1 \
    libxtst6 \
    libxi6 \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

RUN addgroup --system --gid 1001 aether && \
    adduser --system --uid 1001 --ingroup aether --home /app aether

WORKDIR /app

COPY --from=0 /app/src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/deb/aether_*.deb /tmp/

RUN dpkg -i /tmp/aether_*.deb || true && \
    apt-get install -f -y && \
    rm /tmp/aether_*.deb

USER aether

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD aether --version || exit 1

ENTRYPOINT ["aether"]
