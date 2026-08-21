# Modelos glTF — Colegio fotorealista low-poly

Esta carpeta es el pipeline para el colegio fotorealista.

## Estado actual
- **Fase 1**: El colegio se construye 100% con primitivas + materiales PBR mejorados (sin glTF). Funciona en nativo y WASM sin descargas.
- **Fase 2 (cuando añadas modelos)**: Si colocas aquí archivos `.glb` / `.gltf`, el código los cargará automáticamente y hará fallback a primitivas si no existen.

## Convención de nombres (cuando añadas modelos)
```
assets/models/
  desk.glb        # pupitre (1.2x0.7x0.6) — origen en centro base
  chair.glb       # silla
  blackboard.glb  # pizarra 3.0x1.2
  door.glb        # puerta 1.6x2.2
  tree.glb        # árbol low-poly
  bench.glb       # banco
```

## Cómo exportar desde Blender (recomendado)
- Escala: 1 unidad Bevy = 1 metro.
- Aplicar transformaciones (Ctrl+A) y triangulación moderada (<5k tris por modelo).
- Material PBR con `BaseColor` + `Roughness` (Bevy lo lee del glTF).
- Exportar `glTF Binary (.glb)` con `Materials: Export` + `Compression: Off`.

## Fallback
Si el archivo no existe, `src/world/school.rs` usa `try_load_model()` que detecta
el error de carga y genera la primitiva procedural mejorada. No rompe WASM.
