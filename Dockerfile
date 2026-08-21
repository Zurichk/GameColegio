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
# Si Coolify falló con "Unable to find image ... locally" fue un fallo
# transitorio de pull (rate limit / red). Tras hacer `docker pull` manual
# ya queda cacheado. Para evitar que vuelva a pasar se usa el tag flotante
# `1-bookworm` (= última estable 1.x), que siempre resuelve a 1.97.x hoy y
# no se rompe cuando salga 1.98. Si prefieres pin exacto, cambia a
# `rust:1.97.1-bookworm`.
FROM rust:1-bookworm AS build

# Dependencias de sistema que Bevy necesita (alsa/udev se usan en build
# scripts aunque el target final sea wasm; clang/lld para build scripts que
# compilan C; pkg-config es imprescindible; curl para descargar wasm-bindgen
# precompilado).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libasound2-dev \
        libudev-dev \
        clang \
        lld \
        binaryen \
        curl \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Target de compilación web + herramienta de generación de bindings JS.
# wasm-bindgen-cli se descarga precompilado (compilarlo desde fuente tarda
# 5-10 min y consume mucha RAM en servidores pequeños como el de 2GB).
RUN rustup target add wasm32-unknown-unknown \
    && curl -fsSL -o /tmp/wb.tar.gz \
        https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.127/wasm-bindgen-0.2.127-x86_64-unknown-linux-musl.tar.gz \
    && tar -xzf /tmp/wb.tar.gz -C /usr/local \
    && rm -f /tmp/wb.tar.gz \
    && ln -sf /usr/local/wasm-bindgen-0.2.127-x86_64-unknown-linux-musl/wasm-bindgen /usr/local/bin/wasm-bindgen \
    && wasm-bindgen --version

WORKDIR /app

# Optimizaciones para VPS pequeño (2GB): 1 job, sin incremental, reintentos
ENV CARGO_INCREMENTAL=0
ENV CARGO_NET_RETRY=3
ENV CARGO_BUILD_JOBS=1

# 1) Caché de dependencias: primero solo los manifiestos y un binario vacío,
#    para que Docker no recompile todo Bevy cuando cambia solo el código.
#    IMPORTANTE: .cargo/ está en .dockerignore para no filtrar el config de
#    Windows (linker absoluto) al builder Linux.
COPY Cargo.toml Cargo.lock ./

# Truco de caché clásico pero ajustado para 2GB:
# - Solo 1 job para no hacer OOM (Bevy 0.16 + wasm es muy pesado).
# - Se borra solo el artefacto dummy, NO se hace `find -delete` agresivo que
#   rompía la caché en la versión anterior.
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

# Optimiza el WASM (reduce mucho el tamaño: Bevy en debug pesa decenas de MB).
RUN wasm-opt -O3 \
        --enable-bulk-memory \
        web/gamecolegio_bg.wasm \
        -o web/gamecolegio_bg.wasm

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

# Puerto interno del contenedor (5005): en Coolify debes mapear
# "Port Exposed = 5005" en la config del servicio, no 80/3000.
EXPOSE 5005
