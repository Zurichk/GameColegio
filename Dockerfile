# =============================================================================
# GameColegio — imagen para Coolify
#
# Compila el juego a WebAssembly (wasm32) y lo sirve con nginx estático.
# El resultado es una página jugable en cualquier navegador con WebGL2.
#
#   docker build -t gamecolegio .
#   docker run -p 5005:5005 gamecolegio
# =============================================================================

# ---- Etapa 1: build del WASM ------------------------------------------------
# NOTA sobre el tag: `rust:1.97-bookworm` SÍ existe (verificado en tu VPS con
# `docker run --rm rust:1.97-bookworm rustc --version` -> 1.97.1).
# Se usa el tag flotante `1-bookworm` para no pinchar en futuros releases.
FROM rust:1-bookworm AS build

# Dependencias de sistema que Bevy necesita.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libasound2-dev \
        libudev-dev \
        clang \
        lld \
        curl \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Binaryen del repositorio Debian puede ser demasiado antiguo para las tablas
# externref que genera wasm-bindgen. Binaryen >= 110 contiene la corrección de
# exportación de tablas; se fija una versión reciente para que Coolify no
# dependa de la versión disponible en la distribución base.
ARG BINARYEN_VERSION=126
RUN curl -fsSL -o /tmp/binaryen.tar.gz \
        "https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz" \
    && echo "e487e0eac1f02a6739816c617270b033e5d3f8ca90439301fd0286460322fd76  /tmp/binaryen.tar.gz" | sha256sum -c - \
    && tar -xzf /tmp/binaryen.tar.gz -C /opt \
    && ln -sf "/opt/binaryen-version_${BINARYEN_VERSION}/bin/wasm-opt" /usr/local/bin/wasm-opt \
    && rm -f /tmp/binaryen.tar.gz \
    && wasm-opt --version

# Target de compilación web + herramienta de generación de bindings JS.
RUN rustup target add wasm32-unknown-unknown \
    && curl -fsSL -o /tmp/wb.tar.gz \
        https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.127/wasm-bindgen-0.2.127-x86_64-unknown-linux-musl.tar.gz \
    && tar -xzf /tmp/wb.tar.gz -C /usr/local \
    && rm -f /tmp/wb.tar.gz \
    && ln -sf /usr/local/wasm-bindgen-0.2.127-x86_64-unknown-linux-musl/wasm-bindgen /usr/local/bin/wasm-bindgen \
    && wasm-bindgen --version

WORKDIR /app

# Build WASM — getrandom usa los backends JS declarados en Cargo.toml y
# wasm-opt usa Binaryen 126 para no corromper las tablas externref.
# Se deja CARGO_BUILD_JOBS=1 para no agotar la memoria del VPS de 2 GB.
ENV CARGO_INCREMENTAL=0
ENV CARGO_NET_RETRY=3
ENV CARGO_BUILD_JOBS=1

# 1) Caché de dependencias: primero solo los manifiestos y un binario vacío,
#    para que Docker no recompile todo Bevy cuando cambia solo el código.
COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release --target wasm32-unknown-unknown \
    && rm -rf src

# 2) El código real del juego + assets.
COPY src ./src
COPY assets ./assets
COPY index.html ./

# Compila en modo release (optimizado). `touch` fuerza recompilación del
# crate principal aunque la caché dummy exista.
RUN touch src/main.rs \
    && cargo build --release --target wasm32-unknown-unknown

# Genera los bindings JS de arranque para la web (directorio `web/`).
RUN wasm-bindgen \
        --target web \
        --out-dir web \
        target/wasm32-unknown-unknown/release/gamecolegio.wasm

# Optimiza el WASM — `-Oz` reduce el tamaño y se habilitan las extensiones que
# Bevy usa. Binaryen 126 conserva correctamente las tablas externref.
# Si falla, el build sigue (wasm-opt es opcional, solo reduce tamaño).
RUN wasm-opt --strip-debug -Oz \
        --enable-bulk-memory --enable-reference-types \
        web/gamecolegio_bg.wasm \
        -o web/gamecolegio_bg.wasm || cp web/gamecolegio_bg.wasm web/gamecolegio_bg.wasm.bak

# ---- Etapa 2: servidor web estático -----------------------------------------
FROM nginx:1.27-alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/web/ /usr/share/nginx/html/
COPY --from=build /app/index.html /usr/share/nginx/html/index.html
COPY --from=build /app/assets/ /usr/share/nginx/html/assets/

EXPOSE 5005
