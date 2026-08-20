# GameColegio

Videojuego educativo 3D ambientado en un colegio, desarrollado en **Rust** con el motor **Bevy**.

El objetivo del juego es que el estudiante-jugador recorra las instalaciones de un colegio
virtual, entre en las aulas y supere **cuestionarios de distintas asignaturas** (Matemáticas,
Historia, Informática, etc.) interactuando con los profesores. Las partidas se guardan y
se pueden retomar.

> **Estado actual:** FASE 1 completada (prototipo navegable del colegio)
> + mejora de definición visual + **modo tablero estilo Trivial** con rueda
> de 85 casillas (pista + 6 radios + centro), Estrellitas con **estrella
> dibujada**, reto final Tabú en el centro, fichas animadas, **temporizador
> de 1 minuto por pregunta**, **preguntas abiertas sin opciones** (como el
> Trivial real), **1.826 preguntas** (1.428 con opciones + 398 abiertas,
> 68 por categoría y dificultad), acentos del español y fixes de movimiento.
> **FASES 2–8 completadas** (puertas, profesores, diálogos, cuestionarios,
> pausa, guardado/carga, HUD y ajustes) y **FASE 9 completada** (audio
> ambiente y efectos, animaciones del personaje y partículas).
> **Zona de aprendizaje añadida (19/08/2026)**: "Primeros pasos" (leer y
> escribir con 34 palabras y teclado, y sumar/restar/multiplicar/dividir
> con dificultad creciente) y "Juegos de memoria" (parejas de letras,
> números, mixtas, formas y palabras, más **memoria de secuencia** estilo
> Simón). Además: **confirmación antes de salir del juego** (menú principal
> y pausa) y **mejora gráfica del colegio** (ventanas luminosas, luces de
> techo, taquillas, carteles, canasta de baloncesto y porterías de fútbol).
> **Zona de aprendizaje reorganizada (19/08/2026)**: ahora es un **centro
> con 4 secciones** — **Lengua** (leer y escribir, **ortografía** y
> **ahorcado**), **Matemáticas** (sumar/restar/multiplicar/dividir y
> **cálculo mental con temporizador**), **Ciencias** (**ciencias naturales**
> y **geografía de España**, 62 preguntas) y **Memoria** (los 6 juegos
> existentes).
> Ver [docs/progreso.md](docs/progreso.md) para el detalle de fases.
>
> **Internacionalización completa (20/08/2026)**: todo el juego es
> **trilingüe (Español/English/Français)** con selector de idioma en los
> Ajustes: menús, HUD, diálogos de profesores, los 6 juegos de aprendizaje y
> **el banco completo del tablero con 1.826 preguntas por idioma**. Además:
> **juego nuevo "Mayor, menor o igual"** (Matemáticas) y **despliegue web**
> (WASM + Dockerfile para Coolify).

## Requisitos

- **Rust 1.95+** (edición 2024) y Cargo.
- **Toolchain GNU** (`x86_64-pc-windows-gnu`) con MinGW-w64, configurado en
  `.cargo/config.toml`.
- Los binarios se generan en `target/x86_64-pc-windows-gnu/debug/`.

## Ejecución

```bash
cargo run
```

El primer `cargo run` descarga y compila todas las dependencias de Bevy, por lo que puede
tardar varios minutos. Las compilaciones posteriores son mucho más rápidas.

Para comprobar errores sin generar binario:

```bash
cargo check
```

Para ejecutar los tests (colisiones AABB, etc.):

```bash
cargo test
```

## Despliegue web (Docker / Coolify)

El juego también compila a **WebAssembly** y se sirve como página estática:

```bash
# Build de la imagen (compila el WASM con wasm-bindgen y lo sirve con nginx)
docker build -t gamecolegio .

# Probar localmente
docker run -p 8080:80 gamecolegio
# → abrir http://localhost:8080
```

En **Coolify** basta con apuntar el proyecto al repositorio (usa el
`Dockerfile` de la raíz); la aplicación se sirve en el puerto `80`. Detalles:

- **`Dockerfile`**: build multi-etapa (Rust 1.97 + target `wasm32` +
  `wasm-bindgen` 0.2.127 + `wasm-opt`) → nginx estático.
- **`nginx.conf`**: MIME correcto para `.wasm`, caché larga para assets y
  fallback a `index.html`.
- **`index.html`**: página de arranque que carga `gamecolegio.js`.
- En la web el **guardado de partida y los ajustes** se mantienen solo en
  memoria (no hay sistema de archivos); el resto del juego es idéntico.
- Requiere un navegador con **WebGL2**.

## Controles

| Tecla | Acción |
|-------|--------|
| `W` `A` `S` `D` | Moverse (relativo a la cámara) |
| `Espacio` | Saltar |
| `Ratón` | Rotar la cámara en tercera persona / hacer clic en menús y botones |
| `Esc` | Liberar / recapturar el cursor del ratón |

El cursor queda bloqueado dentro de la ventana solo mientras se explora el
colegio. En los menús y en el modo tablero el cursor es libre para hacer clic.

## Funcionalidades implementadas (FASE 1)

- Ventana de juego **1600×900** titulada "GameColegio" con cielo azul.
- **Suelo exterior** e **interior** del colegio.
- **Edificio del colegio** con:
  - Entrada principal → **recepción** → **pasillo** → **3 aulas**.
  - Aulas de **Matemáticas**, **Historia** e **Informática** (suelos y carteles
    con el color de cada asignatura).
  - Puertas (paneles abiertos) en cada aula, ventanas en la pared trasera.
  - Pizarras, mesa del profesor y pupitres en cada aula; mostrador de recepción.
- **Personaje** construido con primitivas (cuerpo + cabeza).
- **Cámara en tercera persona** con seguimiento suave, control de ratón y límite
  de inclinación.
- **Movimiento WASD**, **salto** y **gravedad**.
- **Colisiones AABB** propias (sin motor de física externo) contra paredes y
  mobiliario, con deslizamiento por las paredes.
- **Iluminación**: sol direccional con sombras, luces puntuales cálidas en el
  interior y luz ambiental.
- **Máquina de estados** (`GameState`): menú principal, exploración libre,
  configuración del tablero y partida de tablero (pausa/cuestionario previstos).
- **Menú principal** con modos de juego: explorar el colegio, jugar al
  tablero y **"Zona de aprendizaje"** (centro con 4 secciones: Lengua,
  Matemáticas, Ciencias y Memoria).
- **Modo tablero (Trivial)**: configuración de jugadores (2–4) y dificultad
  (Fácil/Media/Difícil). El tablero reproduce la **rueda clásica con 6
  radios**: **85 casillas** (54 de pista exterior con **Estrellitas de color en
  los 6 vértices**, 30 de radio y 1 central de salida **más grande**). Los
  jugadores **salen del centro** bajando por un radio (5 casillas de paso
  hasta cada Estrellita), circulan por la pista para completar las 6 Estrellitas
  de color y **vuelven al centro** a superar el **reto final Tabú** (el 7º
  Estrellita) para ganar.
- **Casillas especiales**: por la pista hay casillas de **preguntas de cada
  color** (solo el color, sin números), casillas **Tabú** (círculo rojo con
  "!", adivinanzas sin decir la palabra) y casillas de **dado dibujado**
  (blanco con puntos) que permiten **volver a tirar** (12, como el Trivial
  clásico). **Tras cada tirada se elige la dirección** (izquierda/derecha)
  para poder reunir las Estrellitas en el orden que se quiera, y al **salir del
  centro** se elige por qué radio (Estrellita) bajar. **El dado cuenta
  casillas** (la Estrellita está a 6 del centro): con 1-5 caes en una casilla
  de radio, con 6 en la Estrellita y con 7-8 continúas por la pista. Las fichas
  se **mueven casilla a casilla** por la pista y los radios, y si aciertas
  **repites turno**.
- **Tamaños y maquetación**: **fondo oscuro opaco** (no se ve el colegio ni
  el personaje detrás del tablero) con panel redondeado; casillas de pista
  de **42 px** y radios de **34 px**; el **panel de Estrellitas está en un
  lateral derecho** para no tapar el tablero (en el centro solo queda un hub
  circular de salida), y las **fichas estrella miden 42 px** (lo mismo que
  una casilla: al moverse se ve claramente sobre qué casilla están).
- **Colores de las categorías según el Trivial real**: azul=Geografía,
  amarillo=Historia, verde=Ciencia y Naturaleza, púrpura=Arte y Literatura
  (rosa y naranja —Entretenimiento y Deportes y Ocio en el original— los
  usan Informática y Matemáticas). La **leyenda** bajo el tablero muestra
  cada color con su tipo de pregunta y el significado de casillas especiales
  (★ Estrellita, dado, Tabú).
- **Fichas estrella de 7 puntas**: cada ficha es una **estrella de 7
  esquinas** que se **rellena como las Estrellitas** se van consiguiendo: sus 6
  puntas se iluminan con el color de cada categoría y la 7ª se enciende en
  dorado al completar los 6 (reto final).
- **1.826 preguntas en total**: 1.428 con 4 opciones (68 por categoría y
  dificultad) + 398 abiertas. El banco cubre tablas, porcentajes,
  ecuaciones, geometría, historia, geografía, informática, ciencias,
  arte, literatura y adivinanzas Tabú, con opciones **barajadas** en cada
  partida (la correcta nunca está siempre en la A).
- **Temporizador de 1 minuto**: cada pregunta (normal o reto final) muestra
  un contador "⏱ m:ss"; si se agota el tiempo cuenta como fallo y **pasa el
  turno**.
- **Estrellitas con estrella dibujada**: los vértices muestran una **estrella
  blanca de 5 puntas** dibujada (ya no el glifo "★" del texto).
- **Colegio detallado**: jardín con fuente y flores, papeleras, ordenadores
  en Informática, libros en Matemáticas, corcho con notas, cartel
  "BIENVENIDOS", **nubes en el cielo, seto que rodea el patio, arbustos,
  chimenea y antena en el tejado**, camino hasta la salida, **mástil con
  bandera, sol brillante en el cielo y macizos de flores junto al camino**.
- **Fase 2 — Puertas e interacción**: las 3 puertas de las aulas y la
  **entrada principal** son **puertas deslizantes**: con la **tecla E** se
  abren/cierran cuando estás cerca (la colisión las sigue y bloquean el vano
  solo al cerrarse), con **aviso en pantalla** que indica la acción
  disponible.
- **Fase 3 — Profesores (NPC)**: hay **un profesor en cada aula** (modelo con
  primitivas y **corbata del color de la asignatura**) que **patrulla frente
  a la pizarra** con pausas y giros. Si te acercas, **se detiene y se gira
  hacia ti**; al alejarte, retoma la patrulla. Llevan una **etiqueta
  flotante** con su asignatura.
- **Fase 4 — Sistema de diálogos**: con la **tecla E** cerca de un profesor
  se abre una **caja de diálogo** con su nombre (en el color de la
  asignatura) y sus líneas, **personalizadas por asignatura** (Matemáticas,
  Historia e Informática hablan de su materia). Se avanza con **Espacio** o
  **clic izquierdo**; se cierra al terminar o al alejarte. Mientras hablas,
  el jugador queda quieto y las puertas no responden.
- **Fase 5 — Cuestionarios**: con la **tecla Q** cerca de un profesor se abre
  un **cuestionario de 3 preguntas** (Fácil, Media y Difícil) de su
  asignatura. Las **opciones A/B/C/D** se responden con **clic o teclas
  1-4**; cada respuesta muestra una **animación de feedback** (verde/rojo).
  Al final: aciertos, fallos y **nota** — la asignatura se **supera
  acertando las 3** (Sobresaliente).
- **Fase 6 — Menú principal, pausa y resultados**: el menú principal tiene
  botón de **"Salir del juego"**. Durante la exploración, la **tecla Esc**
  abre el **menú de pausa** (el juego se congela y el cursor se libera) con
  **Reanudar / Reiniciar partida / Menú principal / Salir del juego**:
  reiniciar vuelve a poner al jugador en la salida con las puertas abiertas.
  La pantalla de resultados del cuestionario se cierra con clic, Q, Enter o
  Esc.
- **Fase 7 — Guardado y carga de partidas**: el progreso se persiste en
  **`savegame.json`** (JSON): **asignaturas superadas**, **posición del
  jugador** y **estado de las puertas**. Guardado **automático** al superar
  una asignatura y al volver al menú principal, y **manual** con el botón
  "Guardar partida" de la pausa. El menú principal muestra **"Continuar"**
  cuando hay partida guardada y restaura todo al cargar.
- **Fase 8 — HUD, ajustes y sonido**: durante la exploración se muestra un
  **HUD** con la **sala actual** (Patio/Aulas/Pasillo/Recepción) y los
  **chips de las asignaturas** que se **iluminan al superar** cada materia.
  La pantalla de **Ajustes** (botón en el menú principal y en la pausa)
  permite cambiar la **sensibilidad del ratón** (1–10) y el **volumen**
  (0–100 %), con persistencia en **`settings.json`** y sonido de **clic de
  interfaz** procedural (`assets/sfx/click.wav`).
- **Fase 9 — Pulido final**: **audio ambiente** (viento en bucle, ajustable
  en vivo) y **efectos de sonido** procedurales (puertas al abrir/cerrar,
  **pasos al caminar**, **fanfarria al superar una asignatura**);
  **animaciones del personaje** (piernas y brazos oscilando al caminar,
  brazos levantados en el aire y vuelta suave a la posición neutral);
  **partículas**: **confeti de colores** con gravedad al superar una
  asignatura y **hojas cayendo** sobre el patio.
- **Zona de aprendizaje (19/08/2026)**: nuevo menú **"Primeros pasos"** con
  **Leer y escribir** (9 rondas: emparejar palabra-dibujo, pista-palabra y
  escribir con teclado, 34 palabras con `normalize_answer` tolerante) y
  **Sumar / Restar / Multiplicar / Dividir** (10 operaciones con dificultad
  creciente y opciones A/B/C/D); y nuevo menú **"Juegos de memoria"** con
  parejas de **letras (6)**, **números (8)**, **mixtas (10)**, **formas
  (8)** y **palabras (6)**, más **"Memoria de secuencia"** (estilo Simón:
  se ilumina una secuencia de hasta 8 colores que hay que repetir con clics;
  al fallar se vuelve a mostrar). Todo como overlay de UI a pantalla
  completa con estados propios en el módulo `src/learning/`.
- **Zona de aprendizaje reorganizada en 4 secciones (19/08/2026)**: el menú
  principal abre ahora el centro **"Zona de aprendizaje"** con **Lengua**,
  **Matemáticas**, **Ciencias** y **Memoria**. Juegos nuevos:
  **Ortografía** (30 frases con hueco y errores típicos b/v, c/s/z, h, y/ll
  y tildes), **Ahorcado** (48 palabras con pista de categoría, 6 fallos,
  muñeco dibujado por etapas y teclado en pantalla A-Z + Ñ + vocales con
  tilde), **Cálculo mental** (10 operaciones con 12 segundos por pregunta),
  **Ciencias naturales** (30 preguntas de 4 opciones) y **Geografía de
  España** (32 preguntas de 4 opciones).
- **Confirmación de salida (19/08/2026)**: al pulsar **"Salir del juego"**
  (menú principal o pausa) se abre un modal **"¿Seguro que quieres salir
  del juego?"** con "Sí, salir" / "Cancelar" (o Esc para cancelar); solo se
  cierra la aplicación al confirmar.
- **Mayor, menor o igual (20/08/2026)**: nuevo juego de Matemáticas en el
  que se muestran dos números y hay que elegir si el primero es mayor (>),
  menor (<) o igual (=); 10 rondas con dificultad creciente, feedback
  inmediato y nota final.
- **Traducción completa ES/EN/FR (20/08/2026)**: selector de idioma en los
  Ajustes (Español/English/Français) que traduce menús, HUD, diálogos y
  **todas las preguntas** (los 6 juegos de aprendizaje y el banco del tablero
  con 1.826 preguntas por idioma).
- **Despliegue web (20/08/2026)**: el juego compila a **WASM** y se sirve
  con nginx mediante el `Dockerfile` (listo para **Coolify**).
- **Mejora gráfica del colegio (19/08/2026)**: **ventanas con cristal
  luminoso** (emisión suave), **paneles de luz de techo** (emisivos) en las
  3 aulas, pasillo y recepción, **taquillas de colores** en el pasillo,
  **carteles** en la cara interior de las aulas, **canasta de baloncesto**
  (poste, tablero de cristal y aro) al fondo del patio y **dos porterías de
  fútbol** en el césped delantero.

## Arquitectura

El código sigue una separación por módulos, cada uno con su plugin de Bevy:

```
src/
├── main.rs              # Punto de entrada: ventana, plugins y ensure_asset_root
├── game/
│   └── mod.rs           # GameState (máquina de estados), GamePlugin y eventos
├── menu.rs              # Menú principal (explorar, continuar, ajustes, tablero, salir)
├── learning/
│   ├── mod.rs           # Zona de aprendizaje: plugin y helpers de UI
│   ├── menu.rs          # Centro de aprendizaje + secciones Lengua/Matemáticas/Ciencias/Memoria
│   ├── reading.rs       # Leer y escribir (9 rondas, teclado, 34 palabras)
│   ├── spelling.rs      # Ortografía (30 frases con hueco, errores típicos)
│   ├── hangman.rs       # Ahorcado (48 palabras, teclado en pantalla, 6 fallos)
│   ├── math.rs          # Sumar/restar/multiplicar/dividir (10 rondas, A/B/C/D)
│   ├── mental.rs        # Cálculo mental (12 s por operación)
│   ├── compare.rs       # Mayor, menor o igual (10 rondas, > < =)
│   ├── trivia.rs        # Ciencias naturales (30) y Geografía de España (32)
│   ├── memory.rs        # Juegos de memoria (letras/números/mixtas/formas/palabras)
│   └── sequence.rs      # Memoria de secuencia (estilo Simón)
├── pause.rs             # Menú de pausa (Esc): reanudar/guardar/ajustes/reiniciar/menú/salir
├── save.rs              # Persistencia: Progress, savegame.json (JSON) y carga
├── settings.rs          # Ajustes: sensibilidad y volumen (settings.json)
├── hud.rs               # HUD de exploración: sala actual y asignaturas superadas
├── audio.rs             # Sonidos procedurales (clic, puerta, éxito, pasos, ambiente)
├── fx.rs                # Partículas: confeti al superar asignatura y hojas del patio
├── board/
│   ├── mod.rs           # Lógica del modo tablero (dado, turnos, Estrellitas)
│   ├── questions.rs     # Categorías, dificultades y banco de 1.826 preguntas
│   └── ui.rs            # Configuración y tablero en anillo (UI)
├── world/
│   ├── mod.rs           # WorldPlugin: iluminación + puertas (E) y aviso
│   ├── school.rs        # Construcción del colegio 3D
│   ├── teacher.rs       # Profesores NPC: patrulla, detección y diálogos
│   ├── dialog.rs        # Caja de diálogo (E), avance con espacio/clic
│   ├── quiz.rs          # Cuestionarios (Q): opciones A/B/C/D y nota
│   ├── textures.rs      # Texturas procedurales
│   └── collision.rs     # Colisiones AABB (con tests unitarios)
├── player/
│   └── mod.rs           # Personaje: WASD, salto, gravedad, colisiones y animación de marcha
└── camera/
    └── mod.rs           # Cámara en tercera persona (sensibilidad ajustable)
assets/
├── fonts/
│   └── Roboto-Regular.ttf  # Fuente con acentos del español
└── sfx/
    ├── click.wav            # Clic de interfaz (generado en el primer arranque)
    ├── door.wav             # Puerta (generado)
    ├── success.wav          # Fanfarria de asignatura superada (generado)
    ├── step.wav             # Paso del personaje (generado)
    └── ambient.wav          # Viento ambiente en bucle (generado)
```

### Dependencias

| Crate | Versión | Propósito |
|-------|---------|-----------|
| `bevy` | 0.16 | Motor de juego completo (ECS, render 3D, input, ventana) |
| `rand` | 0.8 | Aleatoriedad del dado y selección de preguntas |
| `serde` | 1 | Serialización del guardado de partida (`savegame.json`) |
| `serde_json` | 1 | Formato JSON del guardado |

## Roadmap

Ver [docs/progreso.md](docs/progreso.md). Resumen:

- **FASE 1** ✅ Prototipo navegable del colegio (completada).
- **Mejoras** ✅ Definición visual + modo tablero estilo Trivial (completado):
  tablero hexagonal con reto final Tabú, fichas animadas, 1.826 preguntas
  barajadas, acentos y fixes de movimiento.
- **FASE 2** ✅ Puertas interactivas y sistema de interacción con tecla `E`
  (completada).
- **FASE 3** ✅ Profesores (NPC) en las aulas: modelo con primitivas, patrulla
  frente a la pizarra y detección de cercanía del jugador (completada).
- **FASE 4** ✅ Sistema de diálogos: caja con texto, avance con espacio/clic y
  líneas personalizadas por profesor (completada).
- **FASE 5** ✅ Cuestionarios: 3 preguntas por asignatura con opciones
  A/B/C/D, feedback y nota final (completada).
- **FASE 6** ✅ Menú principal (con "Salir"), pausa con Esc
  (reanudar/reiniciar/menú/salir) y pantalla de resultados del cuestionario
  (completada).
- **FASE 7** ✅ Guardado y carga de partidas: `savegame.json` con
  asignaturas superadas, posición y puertas; guardado automático/manual y
  botón "Continuar" (completada).
- **FASE 8** ✅ HUD de exploración (sala actual y asignaturas superadas),
  pantalla de ajustes (volumen y sensibilidad, `settings.json`) y sonido de
  interfaz (clic procedural) (completada).
- **FASE 9** ✅ Pulido final: audio ambiente y efectos (puertas, pasos,
  fanfarria), animaciones del personaje (marcha) y partículas (confeti y
  hojas) (completada).
- **Zona de aprendizaje** ✅ Centro "Zona de aprendizaje" con 4 secciones:
  Lengua (leer y escribir, ortografía, ahorcado), Matemáticas (operaciones,
  cálculo mental), Ciencias (ciencias naturales, geografía de España) y
  Memoria (parejas de letras, números, mixtas, formas y palabras + memoria
  de secuencia) desde el menú principal (completada 19/08/2026).
- **Confirmación de salida** ✅ Modal "¿Seguro que quieres salir?" al pulsar
  "Salir del juego" en el menú principal y en la pausa (completada 19/08/2026).
- **Mejora gráfica del colegio** ✅ Ventanas luminosas, luces de techo,
  taquillas, carteles, canasta de baloncesto y porterías de fútbol
  (completada 19/08/2026).
- **Corrección de visibilidad** ✅ `Visibility::Visible` → `Inherited` en los
  botones para que los modales y paneles ocultos no se dibujen encima
  (Bevy 0.16 hace que `Visible` ignore al padre) (completada 19/08/2026).
- **Internacionalización** ✅ Español/English/Français en toda la interfaz y
  en **todas las preguntas** (6 juegos de aprendizaje + banco del tablero de
  1.826 por idioma) (completada 20/08/2026).
- **Juego "Mayor, menor o igual"** ✅ Nueva actividad de Matemáticas
  (completada 20/08/2026).
- **Despliegue web** ✅ Compilación WASM + Dockerfile para Coolify
  (completada 20/08/2026).
- **Cámara sin atravesar paredes** ✅ La cámara en tercera persona evita
  colisionar con el edificio muestreando el segmento jugador→deseado
  (completada 20/08/2026).