//! Partículas y detalles visuales (Fase 9).
//!
//! - **Confeti**: al superar una asignatura, un puñado de trocitos de colores
//!   sale despedido desde la posición del profesor y cae con gravedad,
//!   girando, durante ~2 s. Acompañado de la fanfarria de éxito.
//! - **Hojas**: unas hojas caen lentamente sobre el patio (decoración
//!   ambiental), con balanceo lateral y rotación suave.

use bevy::prelude::*;
use rand::Rng;

use crate::audio::{play_success, Sfx};
use crate::game::GameState;
use crate::player::Player;
use crate::world::quiz::QuizSession;
use crate::world::Teacher;

/// Asignatura que ya celebró su superación (evita repetir confeti/fanfarria
/// en cada frame mientras la pantalla de resultados sigue abierta).
#[derive(Resource, Default)]
pub struct Celebrated(pub Option<String>);

/// Plugin de efectos visuales.
pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Celebrated::default())
            .add_systems(Startup, spawn_leaves)
            .add_systems(
                Update,
                (check_celebrate, update_confetti, update_leaves)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---- Confeti ----------------------------------------------------------------

/// Un trozo de confeti: velocidad, giro y vida restante.
#[derive(Component)]
pub struct Confetti {
    velocity: Vec3,
    spin: Vec3,
    life: Timer,
}

/// Detecta el final de un cuestionario superado y lanza el confeti y la
/// fanfarria una sola vez.
fn check_celebrate(
    mut commands: Commands,
    sfx: Res<Sfx>,
    quiz: Option<Res<QuizSession>>,
    mut celebrated: ResMut<Celebrated>,
    player_q: Query<&Transform, With<Player>>,
    teachers: Query<(&Teacher, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(session) = quiz else {
        celebrated.0 = None;
        return;
    };
    if !(session.done && session.passed()) {
        return;
    }
    if celebrated.0.as_deref() == Some(session.subject) {
        return;
    }
    celebrated.0 = Some(session.subject.to_string());
    play_success(&mut commands, &sfx);

    // El confeti sale desde la posición del profesor de la asignatura.
    let center = teachers
        .iter()
        .find(|(teacher, _)| teacher.subject == session.subject)
        .map(|(_, tf)| tf.translation)
        .or_else(|| player_q.single().ok().map(|tf| tf.translation))
        .unwrap_or(Vec3::new(0.0, 1.0, 0.0));

    spawn_confetti(&mut commands, &mut meshes, &mut materials, center);
}

/// Crea `CONFETTI_COUNT` trocitos de colores alrededor del punto dado.
fn spawn_confetti(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
) {
    const CONFETTI_COUNT: usize = 36;
    const COLORS: [Color; 6] = [
        Color::srgb(1.0, 0.85, 0.2),  // amarillo
        Color::srgb(0.95, 0.35, 0.35), // rojo
        Color::srgb(0.30, 0.70, 0.95), // azul
        Color::srgb(0.35, 0.85, 0.40), // verde
        Color::srgb(0.95, 0.45, 0.85), // rosa
        Color::srgb(1.0, 0.60, 0.25),  // naranja
    ];
    let mesh = meshes.add(Cuboid::new(0.08, 0.06, 0.02));
    let mut rng = rand::thread_rng();

    for _ in 0..CONFETTI_COUNT {
        let color = COLORS[rng.gen_range(0..COLORS.len())];
        let material = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.4,
            ..default()
        });
        let dir = Vec3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(0.4..1.2),
            rng.gen_range(-1.0..1.0),
        )
        .normalize_or_zero();
        let speed = rng.gen_range(2.0..4.5);
        commands.spawn((
            Confetti {
                velocity: dir * speed + Vec3::new(0.0, 2.0, 0.0),
                spin: Vec3::new(
                    rng.gen_range(-6.0..6.0),
                    rng.gen_range(-6.0..6.0),
                    rng.gen_range(-6.0..6.0),
                ),
                life: Timer::from_seconds(rng.gen_range(1.6..2.4), TimerMode::Once),
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(center + Vec3::new(0.0, 0.8, 0.0)),
            Visibility::default(),
        ));
    }
}

/// Mueve el confeti con gravedad, lo hace girar y lo destruye al agotarse.
fn update_confetti(
    time: Res<Time>,
    mut commands: Commands,
    mut pieces: Query<(Entity, &mut Transform, &mut Confetti)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut confetti) in &mut pieces {
        confetti.velocity.y -= 9.8 * dt;
        tf.translation += confetti.velocity * dt;
        tf.rotation *= Quat::from_euler(
            EulerRot::XYZ,
            confetti.spin.x * dt,
            confetti.spin.y * dt,
            confetti.spin.z * dt,
        );
        confetti.life.tick(time.delta());
        if confetti.life.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ---- Hojas del patio --------------------------------------------------------

/// Una hoja que cae: velocidad de caída, fase/speed de balanceo y posición
/// base en X/Z (el movimiento en X oscila alrededor de `base_x`).
#[derive(Component)]
pub struct Leaf {
    speed: f32,
    sway_phase: f32,
    sway_speed: f32,
    base_x: f32,
}

/// Crea las hojas que caen sobre el patio (decoración ambiental).
fn spawn_leaves(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const LEAF_COUNT: usize = 24;
    const LEAF_COLORS: [Color; 3] = [
        Color::srgb(0.40, 0.75, 0.30),
        Color::srgb(0.55, 0.80, 0.25),
        Color::srgb(0.85, 0.75, 0.25),
    ];
    let mesh = meshes.add(Cuboid::new(0.22, 0.02, 0.14));
    let mut rng = rand::thread_rng();

    for _ in 0..LEAF_COUNT {
        let color = LEAF_COLORS[rng.gen_range(0..LEAF_COLORS.len())];
        let material = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.6,
            ..default()
        });
        let base_x = rng.gen_range(-20.0..20.0);
        let base_z = rng.gen_range(-12.0..22.0);
        commands.spawn((
            Leaf {
                speed: rng.gen_range(0.25..0.5),
                sway_phase: rng.gen_range(0.0..std::f32::consts::TAU),
                sway_speed: rng.gen_range(0.8..1.6),
                base_x,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(base_x, rng.gen_range(4.0..9.0), base_z),
            Visibility::default(),
        ));
    }
}

/// Hace caer las hojas con balanceo lateral y rotación suave; al llegar al
/// suelo se recolocan arriba (el ciclo es infinito).
fn update_leaves(time: Res<Time>, mut leaves: Query<(&mut Transform, &mut Leaf)>) {
    let dt = time.delta_secs();
    for (mut tf, mut leaf) in &mut leaves {
        leaf.sway_phase += leaf.sway_speed * dt;
        tf.translation.y -= leaf.speed * dt;
        tf.translation.x = leaf.base_x + leaf.sway_phase.sin() * 0.8;
        tf.rotation = Quat::from_rotation_z(leaf.sway_phase.sin() * 0.5)
            * Quat::from_rotation_y(leaf.sway_phase * 0.4);
        if tf.translation.y < -0.5 {
            // Nueva hoja arriba, en otra posición X.
            leaf.sway_phase = 0.0;
            leaf.base_x += 6.0;
            if leaf.base_x > 20.0 {
                leaf.base_x -= 40.0;
            }
            tf.translation.y = 5.0 + leaf.sway_speed * 2.0;
        }
    }
}