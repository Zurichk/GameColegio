//! Cámara en tercera persona que sigue y rota alrededor del jugador.

use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{DistanceFog, FogFalloff, ScreenSpaceAmbientOcclusion};
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
/// Radio de seguridad de la cámara para no rozar una pared.
const CAMERA_CLEARANCE: f32 = 0.18;

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

/// Crea la cámara y el fondo de la escena — ahora con PBR fotorealista.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.72, 0.86, 1.0)),
            ..default()
        },
        // Mapeo de tonos cinematográfico: más contraste y saturación.
        Tonemapping::AcesFitted,
        Bloom {
            intensity: 0.12,
            ..default()
        },
        ScreenSpaceAmbientOcclusion {
            quality_level: bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::Low,
            ..default()
        },
        DistanceFog {
            color: Color::srgb(0.72, 0.86, 1.0),
            falloff: FogFalloff::Linear {
                start: 40.0,
                end: 130.0,
            },
            ..default()
        },
        // SSAO en Bevy 0.16 exige Msaa::Off (si no, error en log y SSAO desactivado).
        // Se usa SMAA vía post-proceso si se quiere anti-aliasing con SSAO.
        Msaa::Off,
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

/// Devuelve la primera fracción del segmento que entra en una AABB expandida.
/// El método de slabs no depende de la densidad de muestreo y no puede saltar
/// paredes finas entre dos puntos consecutivos.
fn segment_entry_t(start: Vec3, end: Vec3, box_aabb: &Aabb) -> Option<f32> {
    let direction = end - start;
    let mut near: f32 = 0.0;
    let mut far: f32 = 1.0;
    for axis in 0..3 {
        if direction[axis].abs() < f32::EPSILON {
            if start[axis] < box_aabb.min[axis] || start[axis] > box_aabb.max[axis] {
                return None;
            }
            continue;
        }
        let inv_direction = 1.0 / direction[axis];
        let mut axis_near = (box_aabb.min[axis] - start[axis]) * inv_direction;
        let mut axis_far = (box_aabb.max[axis] - start[axis]) * inv_direction;
        if axis_near > axis_far {
            std::mem::swap(&mut axis_near, &mut axis_far);
        }
        near = near.max(axis_near);
        far = far.min(axis_far);
        if near > far {
            return None;
        }
    }
    if far >= 0.0 && near <= 1.0 && near >= 0.0 { Some(near) } else { None }
}

/// Mueve la cámara alrededor del jugador según el ratón y la sigue suavemente.
///
/// La cámara no atraviesa paredes: si la posición deseada queda dentro de un
/// collider del mundo (el edificio del colegio, el mobiliario…), se acerca al
/// jugador hasta el primer impacto continuo del segmento jugador → cámara.
fn update_camera(
    time: Res<Time>,
    settings: Res<Settings>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut cameras: Query<(&mut Transform, &mut ThirdPersonCamera), Without<Player>>,
    players: Query<&Transform, With<Player>>,
    walls: Query<(&GlobalTransform, &Collider), (Without<Player>, Without<ThirdPersonCamera>)>,
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
            Aabb::from_center_half_extents(wall_tf.translation(), collider.half_extents)
        })
        .collect();

    // Intersección continua del segmento con las paredes expandidas por el
    // radio de la cámara. Se coloca justo antes del primer impacto.
    let camera_half_extents = Vec3::splat(CAMERA_CLEARANCE);
    let mut allowed_t: f32 = 1.0;
    for wall in &wall_aabbs {
        let expanded = Aabb::from_center_half_extents(
            (wall.min + wall.max) * 0.5,
            (wall.max - wall.min) * 0.5 + camera_half_extents,
        );
        if let Some(entry_t) = segment_entry_t(target, desired, &expanded) {
            allowed_t = allowed_t.min((entry_t - 0.01).max(0.0));
        }
    }
    let cam_pos = target.lerp(desired, allowed_t);

    // Seguimiento suave.
    let t = (cam.smoothness * time.delta_secs()).min(1.0);
    let mut new_translation = cam_tf.translation.lerp(cam_pos, t);
    let current_aabb = Aabb::from_center_half_extents(new_translation, camera_half_extents);
    if wall_aabbs.iter().any(|wall| wall.overlaps(&current_aabb)) {
        new_translation = cam_pos;
    }
    cam_tf.translation = new_translation;
    cam_tf.look_at(target, Vec3::Y);
}

#[cfg(test)]
mod tests {
    use super::segment_entry_t;
    use crate::world::collision::Aabb;
    use bevy::prelude::Vec3;

    #[test]
    fn segment_detects_thin_wall_without_sampling() {
        let wall = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::new(0.05, 2.0, 2.0));
        let entry = segment_entry_t(Vec3::new(-4.0, 1.0, 0.0), Vec3::new(4.0, 1.0, 0.0), &wall);
        assert!(entry.is_some());
        assert!((entry.unwrap() - 0.49375).abs() < 0.001);
    }

    #[test]
    fn segment_ignores_wall_outside_path() {
        let wall = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 5.0), Vec3::splat(0.5));
        assert!(segment_entry_t(Vec3::new(-4.0, 1.0, 0.0), Vec3::new(4.0, 1.0, 0.0), &wall).is_none());
    }
}