//! Cámara en tercera persona que sigue y rota alrededor del jugador.

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use crate::game::GameState;
use crate::player::Player;
use crate::settings::Settings;
use crate::world::collision::{Aabb, Collider};

/// Distancia de la cámara al jugador.
const CAMERA_DISTANCE: f32 = 6.0;
/// Altura del punto al que mira la cámara sobre los pies del jugador.
const LOOK_HEIGHT: f32 = 1.6;
/// Límite de inclinación vertical (radianes).
const PITCH_LIMIT: f32 = 1.2;
/// Sensibilidad del ratón.
const MOUSE_SENSITIVITY: f32 = 0.0035;
/// Factor de suavizado del seguimiento (más alto = más rígido).
const SMOOTHNESS: f32 = 8.0;

/// Orientación y parámetros de la cámara en tercera persona.
#[derive(Component)]
pub struct ThirdPersonCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub look_height: f32,
    pub pitch_limit: f32,
    pub mouse_sensitivity: f32,
    pub smoothness: f32,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.25,
            distance: CAMERA_DISTANCE,
            look_height: LOOK_HEIGHT,
            pitch_limit: PITCH_LIMIT,
            mouse_sensitivity: MOUSE_SENSITIVITY,
            smoothness: SMOOTHNESS,
        }
    }
}

/// Plugin de la cámara en tercera persona.
pub struct ThirdPersonCameraPlugin;

impl Plugin for ThirdPersonCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            // El cursor se bloquea solo mientras se explora el colegio.
            // Al pausar (Escape) se libera automáticamente con OnExit; al
            // reanudar se vuelve a capturar con OnEnter.
            .add_systems(OnEnter(GameState::Playing), lock_cursor)
            .add_systems(OnExit(GameState::Playing), unlock_cursor)
            .add_systems(
                Update,
                update_camera.run_if(in_state(GameState::Playing)),
            );
    }
}

/// Crea la cámara y el fondo de la escena.
fn spawn_camera(mut commands: Commands) {

    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.72, 0.86, 1.0)),
            ..default()
        },
        // Mapeo de tonos cinematográfico: más contraste y saturación.
        Tonemapping::AcesFitted,
        DistanceFog {
            color: Color::srgb(0.72, 0.86, 1.0),
            falloff: FogFalloff::Linear {
                start: 40.0,
                end: 130.0,
            },
            ..default()
        },
        ThirdPersonCamera::default(),
        Transform::from_xyz(0.0, 4.0, 22.0).looking_at(Vec3::new(0.0, 1.6, 16.0), Vec3::Y),
    ));
}

/// Captura el cursor dentro de la ventana.
fn lock_cursor(mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;
}

/// Libera el cursor.
fn unlock_cursor(mut window: Single<&mut Window>) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

/// Mueve la cámara alrededor del jugador según el ratón y la sigue suavemente.
///
/// La cámara no atraviesa paredes: si la posición deseada queda dentro de un
/// collider del mundo (el edificio del colegio, el mobiliario…), se acerca al
/// jugador muestreando el segmento jugador → deseado hasta hallar el punto más
/// lejano sin colisión.
fn update_camera(
    time: Res<Time>,
    settings: Res<Settings>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cameras: Query<(&mut Transform, &mut ThirdPersonCamera), Without<Player>>,
    players: Query<&Transform, With<Player>>,
    walls: Query<(&Transform, &Collider), (Without<Player>, Without<ThirdPersonCamera>)>,
) {
    let Ok(player_tf) = players.single() else {
        return;
    };
    let Ok((mut cam_tf, mut cam)) = cameras.single_mut() else {
        return;
    };

    // Rotación con el ratón (la sensibilidad se ajusta desde los Ajustes).
    let sensitivity = cam.mouse_sensitivity * settings.sensitivity_multiplier();
    for motion in mouse_motion.read() {
        cam.yaw -= motion.delta.x * sensitivity;
        cam.pitch -= motion.delta.y * sensitivity;
    }
    let pitch_limit = cam.pitch_limit;
    let clamped_pitch = cam.pitch.clamp(-pitch_limit, pitch_limit);
    cam.pitch = clamped_pitch;

    // Posición objetivo detrás del jugador.
    let target = player_tf.translation + Vec3::new(0.0, cam.look_height, 0.0);
    let offset = Vec3::new(
        cam.yaw.sin() * cam.pitch.cos(),
        cam.pitch.sin(),
        cam.yaw.cos() * cam.pitch.cos(),
    ) * cam.distance;
    let desired = target - offset;

    // Cajas de colisión del mundo (paredes, mobiliario, edificio).
    let wall_aabbs: Vec<Aabb> = walls
        .iter()
        .map(|(wall_tf, collider)| {
            Aabb::from_center_half_extents(wall_tf.translation, collider.half_extents)
        })
        .collect();

    // Muestreo del segmento jugador → deseado (16 puntos): el punto más lejano
    // que no esté dentro de ninguna caja es la posición válida de la cámara.
    // Así la cámara se pega a la pared en lugar de atravesarla.
    let mut cam_pos = desired;
    for step in (0..=16).rev() {
        let t = step as f32 / 16.0;
        let probe = target.lerp(desired, t);
        let cam_aabb = Aabb::from_center_half_extents(probe, Vec3::splat(0.15));
        if !wall_aabbs.iter().any(|w| w.overlaps(&cam_aabb)) {
            cam_pos = probe;
            break;
        }
    }

    // Seguimiento suave.
    let t = (cam.smoothness * time.delta_secs()).min(1.0);
    let new_translation = cam_tf.translation.lerp(cam_pos, t);
    cam_tf.translation = new_translation;
    cam_tf.look_at(target, Vec3::Y);
}