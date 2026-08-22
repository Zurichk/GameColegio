//! Control del personaje del jugador: movimiento, salto, gravedad y colisiones.

use bevy::prelude::*;

use crate::audio::{play_step, Sfx};
use crate::camera::ThirdPersonCamera;
use crate::game::{GameState, RestartWorld, RestoreWorld};
use crate::save::Progress;
use crate::world::collision::{resolve_aabbs, Aabb, Collider};
use crate::world::dialog::DialogSession;
use crate::world::quiz::QuizSession;

/// Velocidad de desplazamiento horizontal (m/s).
pub const PLAYER_SPEED: f32 = 5.0;
/// Velocidad vertical inicial del salto (m/s).
pub const JUMP_VELOCITY: f32 = 7.5;
/// Gravedad aplicada al jugador (m/s²).
pub const GRAVITY: f32 = -22.0;
/// Semiextensiones de la caja de colisión del jugador.
pub const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.3, 0.5, 0.3);
/// Posición de salida (pies en el suelo, delante de la recepción).
pub const PLAYER_SPAWN: Vec3 = Vec3::new(0.0, PLAYER_HALF_EXTENTS.y, 16.0);

/// Marca la entidad controlada por el jugador.
#[derive(Component)]
pub struct Player;

/// Velocidad actual del jugador.
#[derive(Component)]
pub struct PlayerVelocity(pub Vec3);

/// Indica si el jugador apoya los pies en el suelo.
#[derive(Component)]
pub struct OnGround(pub bool);

/// Marca la pierna izquierda (para la animación de marcha).
#[derive(Component)]
pub struct LeftLeg;

/// Marca la pierna derecha.
#[derive(Component)]
pub struct RightLeg;

/// Marca el brazo izquierdo.
#[derive(Component)]
pub struct LeftArm;

/// Marca el brazo derecho.
#[derive(Component)]
pub struct RightArm;

/// Fase de la marcha (radianes) para oscilar piernas y brazos.
#[derive(Resource, Default)]
pub struct WalkPhase(pub f32);

/// Plugin de control del jugador.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WalkPhase::default()).add_systems(
            Startup,
            spawn_player,
        );
        app.add_systems(
            Update,
            (
                move_player,
                animate_player.run_if(in_state(GameState::Playing)),
                reset_player.run_if(on_event::<RestartWorld>),
                restore_player.run_if(on_event::<RestoreWorld>),
            ),
        );
    }
}

/// Crea el personaje del jugador a partir de primitivas simples.
fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.45, 0.85),
        perceptual_roughness: 0.6,
        ..default()
    });
    let skin_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.80, 0.65),
        perceptual_roughness: 0.5,
        ..default()
    });
    let shoe_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.13, 0.12),
        perceptual_roughness: 0.8,
        ..default()
    });

    let torso_mesh = meshes.add(Cuboid::new(0.5, 0.7, 0.3));
    let leg_mesh = meshes.add(Cylinder::new(0.09, 0.45));
    let arm_mesh = meshes.add(Cylinder::new(0.07, 0.55));
    let head_mesh = meshes.add(Sphere::new(0.18));
    let shoe_mesh = meshes.add(Cuboid::new(0.18, 0.1, 0.28));

    commands
        .spawn((
            Player,
            Collider::new(PLAYER_HALF_EXTENTS),
            PlayerVelocity(Vec3::ZERO),
            OnGround(true),
            Transform::from_translation(PLAYER_SPAWN),
            // Necesario para que los hijos renderizables (cuerpo/cabeza)
            // reciban InheritedVisibility correctamente (evita warning B0004).
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Grupo visual. El origen del modelo está en los PIES (las piernas
            // llegan a y = -0.005), pero la raíz es el CENTRO de la caja de
            // colisión (y = PLAYER_HALF_EXTENTS.y). Bajamos el modelo medio
            // cuerpo para que los pies toquen el suelo.
            parent
                .spawn((
                    Transform::from_xyz(0.0, -PLAYER_HALF_EXTENTS.y, 0.0),
                    // Necesario: Mesh3d (obligatorio en las partes del cuerpo)
                    // exige Visibility y, sin él en este padre, salta B0004.
                    Visibility::default(),
                ))
                .with_children(|body| {
                    // Piernas (con marcador para la animación de marcha).
                    body.spawn((
                        LeftLeg,
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_xyz(-0.14, 0.22, 0.0),
                    ));
                    body.spawn((
                        RightLeg,
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_xyz(0.14, 0.22, 0.0),
                    ));
                    // Torso.
                    body.spawn((
                        Mesh3d(torso_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_xyz(0.0, 0.65, 0.0),
                    ));
                    // Brazos (con marcador para la animación de marcha).
                    body.spawn((
                        LeftArm,
                        Mesh3d(arm_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_xyz(-0.38, 0.72, 0.0),
                    ));
                    body.spawn((
                        RightArm,
                        Mesh3d(arm_mesh.clone()),
                        MeshMaterial3d(body_material.clone()),
                        Transform::from_xyz(0.38, 0.72, 0.0),
                    ));
                    // Cabeza.
                    body.spawn((
                        Mesh3d(head_mesh.clone()),
                        MeshMaterial3d(skin_material.clone()),
                        Transform::from_xyz(0.0, 1.1, 0.0),
                    ));
                    // Zapatos.
                    body.spawn((
                        Mesh3d(shoe_mesh.clone()),
                        MeshMaterial3d(shoe_material.clone()),
                        Transform::from_xyz(-0.14, 0.06, 0.03),
                    ));
                    body.spawn((
                        Mesh3d(shoe_mesh.clone()),
                        MeshMaterial3d(shoe_material.clone()),
                        Transform::from_xyz(0.14, 0.06, 0.03),
                    ));
                });
        });
}

/// Aplica el movimiento del jugador: WASD, salto, gravedad, colisiones y suelo.
/// Durante un diálogo o un cuestionario el jugador queda quieto.
fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    dialog: Option<Res<DialogSession>>,
    quiz: Option<Res<QuizSession>>,
    camera_q: Query<&ThirdPersonCamera>,
    walls: Query<(&GlobalTransform, &Collider), Without<Player>>,
    mut player_q: Query<
        (&mut Transform, &mut PlayerVelocity, &mut OnGround),
        With<Player>,
    >,
) {
    if dialog.is_some() || quiz.is_some() {
        return;
    }
    // Limitar el paso evita que un tirón de FPS convierta un solo movimiento
    // en un salto capaz de atravesar una pared delgada.
    let dt = time.delta_secs().min(0.05);
    let Ok(camera) = camera_q.single() else {
        return;
    };
    let Ok((mut transform, mut velocity, mut on_ground)) = player_q.single_mut() else {
        return;
    };

    // Direcciones relativas a la orientación de la cámara. La cámara mira
    // hacia (sin(yaw), 0, cos(yaw)), así que esa es la dirección "adelante".
    let forward = Vec3::new(camera.yaw.sin(), 0.0, camera.yaw.cos());
    let right = Vec3::new(-camera.yaw.cos(), 0.0, camera.yaw.sin());

    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }

    let horizontal = if direction.length_squared() > 0.0 {
        direction.normalize() * PLAYER_SPEED
    } else {
        Vec3::ZERO
    };

    // Velocidad horizontal inmediata + gravedad.
    velocity.0.x = horizontal.x;
    velocity.0.z = horizontal.z;
    velocity.0.y += GRAVITY * dt;

    // Salto.
    if keys.just_pressed(KeyCode::Space) && on_ground.0 {
        velocity.0.y = JUMP_VELOCITY;
    }

    let candidate = transform.translation + velocity.0 * dt;

    // Colisiones AABB con paredes y mobiliario.
    let aabbs: Vec<Aabb> = walls
        .iter()
        .map(|(wall_tf, collider)| {
            Aabb::from_center_half_extents(wall_tf.translation(), collider.half_extents)
        })
        .collect();
    let resolved = resolve_aabbs(transform.translation, candidate, PLAYER_HALF_EXTENTS, &aabbs);
    transform.translation = resolved;

    // Suelo: la cara superior del terreno está en y = 0.
    if transform.translation.y <= PLAYER_HALF_EXTENTS.y {
        transform.translation.y = PLAYER_HALF_EXTENTS.y;
        velocity.0.y = 0.0;
        on_ground.0 = true;
    } else {
        on_ground.0 = false;
    }
}

/// Vuelve a colocar al jugador en la salida con velocidad y suelo reiniciados
/// cuando llega un evento `RestartWorld` (botón "Reiniciar partida" de la
/// pausa).
fn reset_player(
    mut player_q: Query<(&mut Transform, &mut PlayerVelocity, &mut OnGround), With<Player>>,
    mut restart: EventReader<RestartWorld>,
) {
    let mut triggered = false;
    for _ in restart.read() {
        triggered = true;
    }
    if !triggered {
        return;
    }
    for (mut transform, mut velocity, mut on_ground) in &mut player_q {
        transform.translation = PLAYER_SPAWN;
        velocity.0 = Vec3::ZERO;
        on_ground.0 = true;
    }
}

/// Coloca al jugador en la posición guardada al llegar un evento
/// `RestoreWorld` (botón "Continuar" del menú principal).
fn restore_player(
    progress: Option<Res<Progress>>,
    mut player_q: Query<(&mut Transform, &mut PlayerVelocity, &mut OnGround), With<Player>>,
    mut restore: EventReader<RestoreWorld>,
) {
    let mut triggered = false;
    for _ in restore.read() {
        triggered = true;
    }
    if !triggered {
        return;
    }
    let Some(progress) = progress else {
        return;
    };
    for (mut transform, mut velocity, mut on_ground) in &mut player_q {
        transform.translation = progress.player_pos;
        velocity.0 = Vec3::ZERO;
        on_ground.0 = true;
    }
}

/// Anima el cuerpo del personaje (Fase 9):
///
/// - **Al caminar**: piernas y brazos oscilan en oposición (la fase avanza
///   con la velocidad) y suena un **paso** cada vez que la fase cruza el
///   cero (dos pasos por ciclo de marcha).
/// - **En el aire**: los brazos se levantan ligeramente.
/// - **Quieto**: las extremidades vuelven suavemente a la posición neutral.
fn animate_player(
    time: Res<Time>,
    mut phase: ResMut<WalkPhase>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    player_q: Query<(&PlayerVelocity, &OnGround), With<Player>>,
    mut limbs: Query<
        (
            &mut Transform,
            Option<&LeftLeg>,
            Option<&RightLeg>,
            Option<&LeftArm>,
            Option<&RightArm>,
        ),
        Without<Player>,
    >,
) {
    let dt = time.delta_secs();
    let Ok((velocity, on_ground)) = player_q.single() else {
        return;
    };
    let speed = Vec2::new(velocity.0.x, velocity.0.z).length();

    if on_ground.0 && speed > 0.2 {
        // Marcha: la fase avanza más rápido cuanto más deprisa se camina.
        let prev = phase.0;
        phase.0 += dt * (4.0 + speed * 1.2);
        let swing = phase.0.sin() * 0.55;
        // Un paso en cada cruce de cero de la fase.
        if prev.sin().signum() != phase.0.sin().signum() {
            play_step(&mut commands, &sfx);
        }
        for (mut tf, left_leg, right_leg, left_arm, right_arm) in &mut limbs {
            if left_leg.is_some() {
                tf.rotation = Quat::from_rotation_x(swing);
            } else if right_leg.is_some() {
                tf.rotation = Quat::from_rotation_x(-swing);
            } else if left_arm.is_some() {
                tf.rotation = Quat::from_rotation_x(-swing * 0.8);
            } else if right_arm.is_some() {
                tf.rotation = Quat::from_rotation_x(swing * 0.8);
            }
        }
    } else if !on_ground.0 {
        // En el aire: brazos ligeramente levantados.
        for (mut tf, _left_leg, _right_leg, left_arm, right_arm) in &mut limbs {
            if left_arm.is_some() {
                tf.rotation = Quat::from_rotation_x(-1.1);
            } else if right_arm.is_some() {
                tf.rotation = Quat::from_rotation_x(1.1);
            }
        }
    } else {
        // Quieto: vuelven a la posición neutral suavemente.
        phase.0 = 0.0;
        for (mut tf, _left_leg, _right_leg, _left_arm, _right_arm) in &mut limbs {
            tf.rotation = tf
                .rotation
                .slerp(Quat::IDENTITY, (dt * 10.0).min(1.0));
        }
    }
}