# =============================================================================
# GameColegio — imagen para Coolify
#
# Compila el juego a WebAssembly (wasm32) y lo sirve con nginx estático.
# El resultado es una página jugable en cualquier navegador con WebGL2.
#
#   docker build -t gamecolegio .
#   docker run -p 8080:80 gamecolegio
# =============================================================================

# ---- Etapa 1: build del WASM ------------------------------------------------
FROM rust:1.85-bookworm AS build

# Dependencias de sistema que Bevy necesita (alsa/udev se usan en build
# scripts aunque el target final sea wasm; pkg-config es imprescindible).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libasound2-dev \
        libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Target de compilación web + herramienta de generación de bindings JS.
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.127 --locked

WORKDIR /app

# Capa de dependencias: primero solo Cargo.toml/Cargo.lock para aprovechar
# la caché de Docker (el build de Bevy es largo).
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
COPY index.html ./

# Compila en modo release (optimizado).
RUN cargo build --release --target wasm32-unknown-unknown

# Genera los bindings JS de arranque para la web (directorio `web/`).
RUN wasm-bindgen \
        --target web \
        --out-dir web \
        target/wasm32-unknown-unknown/release/gamecolegio.wasm

# ---- Etapa 2: servidor web estático -----------------------------------------
FROM nginx:1.27-alpine

# Configuración: los .wasm se sirven con el MIME correcto y con caché larga
# (necesario para que el navegador los cargue bien).
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Bindings WASM + página de arranque.
COPY --from=build /app/web/ /usr/share/nginx/html/
COPY --from=build /app/index.html /usr/share/nginx/html/index.html

# Assets del juego (fuentes y efectos de sonido).
COPY --from=build /app/assets/ /usr/share/nginx/html/assets/

EXPOSE 80