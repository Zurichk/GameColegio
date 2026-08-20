//! Efectos de sonido (Fase 8 y 9).
//!
//! Todos los WAV se generan de forma procedimental en `assets/sfx/` el
//! primer arranque (no hace falta ningún asset binario en el repo):
//!
//! - `click.wav`    — clic de la interfaz (880 Hz, corto).
//! - `door.wav`     — deslizamiento de puerta (barrido descendente).
//! - `success.wav`  — fanfarria al superar una asignatura (arpegio).
//! - `step.wav`     — paso del personaje al caminar (golpe grave corto).
//! - `ambient.wav`  — viento suave en bucle (ruido suavizado), volumen bajo.

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

/// Frecuencia de muestreo común a todos los WAV.
#[cfg(not(target_arch = "wasm32"))]
const SAMPLE_RATE: u32 = 22_050;

/// Sonidos del juego, listos para reproducir.
#[derive(Resource)]
pub struct Sfx {
    /// Clic de los botones de la interfaz.
    pub click: Handle<AudioSource>,
    /// Apertura/cierre de una puerta.
    pub door: Handle<AudioSource>,
    /// Asignatura superada.
    pub success: Handle<AudioSource>,
    /// Paso del jugador al caminar.
    pub step: Handle<AudioSource>,
}

/// Marca la entidad de audio ambiente para poder ajustar su volumen en vivo.
#[derive(Component)]
pub struct AmbientMusic;

/// Plugin de efectos de sonido.
pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_sfx);
    }
}

/// Genera los WAV si no existen y guarda los manejadores; lanza el bucle de
/// ambiente a volumen bajo (la pantalla de ajustes lo controla después).
fn setup_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    ensure_sfx_files();
    let ambient: Handle<AudioSource> = asset_server.load("sfx/ambient.wav");
    commands.insert_resource(Sfx {
        click: asset_server.load("sfx/click.wav"),
        door: asset_server.load("sfx/door.wav"),
        success: asset_server.load("sfx/success.wav"),
        step: asset_server.load("sfx/step.wav"),
    });
    commands.spawn((
        AmbientMusic,
        AudioPlayer(ambient),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.5)),
    ));
}

/// Reproduce un efecto one-shot (se destruye solo al terminar).
pub fn play_sound(commands: &mut Commands, handle: Handle<AudioSource>) {
    commands.spawn((AudioPlayer(handle), PlaybackSettings::DESPAWN));
}

/// Reproduce el clic de la interfaz.
pub fn play_click(commands: &mut Commands, sfx: &Sfx) {
    play_sound(commands, sfx.click.clone());
}

/// Reproduce el sonido de puerta.
pub fn play_door(commands: &mut Commands, sfx: &Sfx) {
    play_sound(commands, sfx.door.clone());
}

/// Reproduce la fanfarria de asignatura superada.
pub fn play_success(commands: &mut Commands, sfx: &Sfx) {
    play_sound(commands, sfx.success.clone());
}

/// Reproduce un paso.
pub fn play_step(commands: &mut Commands, sfx: &Sfx) {
    play_sound(commands, sfx.step.clone());
}

/// Escribe todos los WAV en `assets/sfx/` si todavía no existen.
/// En la web (WASM) los WAV se sirven como assets HTTP (ya incluidos en el
/// repo), por lo que no hay nada que generar: no-op.
#[cfg(not(target_arch = "wasm32"))]
fn ensure_sfx_files() {
    let Some(root) = asset_root() else {
        return;
    };
    let dir = root.join("assets").join("sfx");
    let files: [(&str, fn() -> Vec<u8>); 5] = [
        ("click.wav", wav_gen::click_wav_bytes),
        ("door.wav", wav_gen::door_wav_bytes),
        ("success.wav", wav_gen::success_wav_bytes),
        ("step.wav", wav_gen::step_wav_bytes),
        ("ambient.wav", wav_gen::ambient_wav_bytes),
    ];
    for (name, generator) in files {
        let path = dir.join(name);
        if path.exists() {
            continue;
        }
        if let Err(err) = fs::create_dir_all(&dir).and_then(|_| fs::write(&path, generator())) {
            bevy::log::warn!("no se pudo generar {name}: {err}");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn ensure_sfx_files() {}

/// Carpeta raíz del proyecto (donde está `assets`).
#[cfg(not(target_arch = "wasm32"))]
fn asset_root() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("BEVY_ASSET_ROOT") {
        return Some(Path::new(&root).to_path_buf());
    }
    std::env::current_dir().ok()
}

// ---- Generación de WAV ------------------------------------------------------
// En la web (WASM) los WAV ya están en `assets/sfx/` y se sirven por HTTP;
// toda esta sección es específica del escritorio (no-op en WASM).

#[cfg(not(target_arch = "wasm32"))]
mod wav_gen {
    use super::SAMPLE_RATE;

/// Escribe la cabecera RIFF/WAVE (PCM mono 16 bits a 22050 Hz) y el chunk
/// "data" con el tamaño de `data_len` bytes. Devuelve el offset de las
/// muestras (44 bytes).
pub fn write_wav_header(wav: &mut Vec<u8>, data_len: usize) {
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&((36 + data_len) as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // 1 canal
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes()); // bloque
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits por muestra
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_len as u32).to_le_bytes());
}

/// Añade una muestra senoidal mono (con amplitud y envolvente).
fn push_sample(
    wav: &mut Vec<u8>,
    phase: f32,
    freq: f32,
    amplitude: f32,
    envelope: f32,
) {
    let sample = (phase * 2.0 * std::f32::consts::PI * freq).sin() * amplitude * envelope;
    let value = (sample * i16::MAX as f32) as i16;
    wav.extend_from_slice(&value.to_le_bytes());
}

/// Genera un WAV PCM mono: un pitido corto de 880 Hz con fundidos de
/// entrada/salida para que no suene a "pop" (clic de interfaz).
pub fn click_wav_bytes() -> Vec<u8> {
    const DURATION: f32 = 0.12;
    let n = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut wav = Vec::with_capacity(44 + n * 2);
    write_wav_header(&mut wav, n * 2);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let fade_in = (t / 0.008).min(1.0);
        let fade_out = ((DURATION - t) / 0.02).min(1.0);
        let fade = fade_in.min(fade_out).max(0.0);
        push_sample(&mut wav, t, 880.0, 0.5, fade);
    }
    wav
}

/// Genera un WAV de puerta: un barrido descendente (400→120 Hz) con un
/// ligero ruido, 0,3 s. Suena a deslizamiento mecánico.
pub fn door_wav_bytes() -> Vec<u8> {
    const DURATION: f32 = 0.3;
    let n = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut wav = Vec::with_capacity(44 + n * 2);
    write_wav_header(&mut wav, n * 2);
    let mut seed: u32 = 42;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let progress = t / DURATION;
        let freq = 400.0 - 280.0 * progress;
        // Envolvente: ataque rápido, salida suave.
        let env = (t / 0.01).min(1.0) * (1.0 - progress * 0.4);
        // Ruido determinista (LCG).
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let noise = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let tonal = (t * 2.0 * std::f32::consts::PI * freq).sin() * 0.35 + noise * 0.15;
        let value = (tonal * env * i16::MAX as f32) as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

/// Genera una fanfarria corta de éxito: arpegio ascendente do-mi-sol (C5-E5-G5)
/// de 0,5 s en total, con fundidos para evitar clics.
pub fn success_wav_bytes() -> Vec<u8> {
    const DURATION: f32 = 0.5;
    const NOTES: [f32; 3] = [523.25, 659.25, 783.99]; // C5, E5, G5.
    const NOTE_LEN: f32 = 0.16;
    let n = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut wav = Vec::with_capacity(44 + n * 2);
    write_wav_header(&mut wav, n * 2);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let mut sample = 0.0;
        for (note, freq) in NOTES.iter().enumerate() {
            let start = note as f32 * NOTE_LEN;
            let local = t - start;
            if (0.0..NOTE_LEN).contains(&local) {
                let fade_in = (local / 0.01).min(1.0);
                let fade_out = ((NOTE_LEN - local) / 0.02).min(1.0);
                let env = fade_in.min(fade_out).max(0.0);
                sample += (local * 2.0 * std::f32::consts::PI * freq).sin() * 0.3 * env;
            }
        }
        let value = (sample * i16::MAX as f32) as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

/// Genera un paso: golpe grave (90 Hz) muy corto con caída rápida, 0,09 s.
pub fn step_wav_bytes() -> Vec<u8> {
    const DURATION: f32 = 0.09;
    let n = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut wav = Vec::with_capacity(44 + n * 2);
    write_wav_header(&mut wav, n * 2);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (t / 0.004).min(1.0) * (1.0 - t / DURATION).max(0.0);
        let sample = (t * 2.0 * std::f32::consts::PI * 90.0).sin() * 0.5 * env;
        let value = (sample * i16::MAX as f32) as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

/// Genera el ambiente: 3 s de ruido suavizado (marrón) que suena a viento
/// lejano. Se reproduce en bucle con volumen bajo.
pub fn ambient_wav_bytes() -> Vec<u8> {
    const DURATION: f32 = 3.0;
    let n = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut wav = Vec::with_capacity(44 + n * 2);
    write_wav_header(&mut wav, n * 2);
    let mut seed: u32 = 7;
    let mut smoothed = 0.0f32;
    for _ in 0..n {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let noise = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        // Suavizado fuerte: "ruido marrón" (más grave, menos chirriante).
        smoothed = smoothed * 0.97 + noise * 0.03;
        // Envolvente lenta para que el bucle no se note.
        let t = smoothed;
        let value = (t * 0.6 * i16::MAX as f32) as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

} // mod wav_gen

#[cfg(test)]
mod tests {
    use super::wav_gen::*;
    use super::*;

    /// Comprueba que la cabecera RIFF/WAVE de un WAV generado es coherente
    /// (tamaño RIFF = longitud - 8, chunks fmt/data en su sitio).
    fn assert_valid_header(wav: &[u8]) {
        assert!(wav.len() > 44, "WAV demasiado corto");
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        let riff_size = u32::from_le_bytes(wav[4..8].try_into().unwrap());
        assert_eq!(riff_size as usize, wav.len() - 8);
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1); // mono
        assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16); // 16 bits
    }

    #[test]
    fn all_wavs_have_valid_headers() {
        assert_valid_header(&click_wav_bytes());
        assert_valid_header(&door_wav_bytes());
        assert_valid_header(&success_wav_bytes());
        assert_valid_header(&step_wav_bytes());
        assert_valid_header(&ambient_wav_bytes());
    }

    #[test]
    fn sounds_have_sensible_durations() {
        // Clic ~0,12 s, puerta ~0,3 s, éxito ~0,5 s, paso ~0,09 s, ambiente 3 s.
        let samples = |wav: &[u8]| (wav.len() - 44) / 2;
        assert!((samples(&click_wav_bytes()) as f32 / SAMPLE_RATE as f32 - 0.12).abs() < 0.01);
        assert!((samples(&door_wav_bytes()) as f32 / SAMPLE_RATE as f32 - 0.3).abs() < 0.01);
        assert!((samples(&success_wav_bytes()) as f32 / SAMPLE_RATE as f32 - 0.5).abs() < 0.01);
        assert!((samples(&step_wav_bytes()) as f32 / SAMPLE_RATE as f32 - 0.09).abs() < 0.01);
        assert!((samples(&ambient_wav_bytes()) as f32 / SAMPLE_RATE as f32 - 3.0).abs() < 0.01);
    }
}