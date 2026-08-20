//! Profesores (NPC): modelo con primitivas en cada aula, patrulla simple y
//! detección de cercanía del jugador (Fase 3).
//!
//! Cada profesor camina de un lado a otro frente a su pizarra, entre la mesa
//! del profesor y la primera fila de pupitres. Cuando el jugador se acerca,
//! se detiene y se gira para mirarle.

use bevy::prelude::*;

use crate::game::GameState;
use crate::player::Player;
use crate::world::collision::Collider;

/// Velocidad de patrulla del profesor (m/s).
const TEACHER_SPEED: f32 = 1.1;
/// Distancia (metros) a la que el profesor detecta al jugador y se detiene.
const NOTICE_DISTANCE: f32 = 2.6;
/// Pausa (s) en los extremos de la patrulla antes de dar la vuelta.
const PAUSE_AT_END: f32 = 1.2;
/// Semiextensiones de la caja de colisión del profesor (como el jugador).
const TEACHER_HALF_EXTENTS: Vec3 = Vec3::new(0.3, 0.5, 0.3);

/// Profesor NPC: patrulla entre `z_back` y `z_front` a lo largo de `cx`.
#[derive(Component)]
pub struct Teacher {
    /// Límite trasero (hacia la pizarra).
    pub z_back: f32,
    /// Límite delantero (hacia la puerta).
    pub z_front: f32,
    /// Dirección actual: +1 hacia la puerta, -1 hacia la pizarra.
    pub dir: f32,
    /// Tiempo restante de pausa en un extremo (s).
    pub pause: f32,
    /// Entidad del grupo visual (se espeja para mirar atrás).
    pub body: Entity,
    /// Asignatura que imparte (para el diálogo).
    pub subject: &'static str,
    /// Color de la asignatura (acento del diálogo y la corbata).
    pub accent: Color,
    /// Líneas de diálogo de este profesor (Fase 4).
    pub lines: &'static [&'static str],
}

/// Diálogo del profesor de Matemáticas.
const LINES_MATH: &[&str] = &[
    "¡Hola! Soy el profesor de Matemáticas.",
    "Para ganar la Estrellita azul, domina las tablas, los porcentajes y la geometría.",
    "En el tablero te preguntaré sumas, ecuaciones y algo de álgebra.",
    "¡Mucha suerte! Y recuerda: la práctica hace al maestro.",
];

/// Diálogo del profesor de Historia.
const LINES_HISTORY: &[&str] = &[
    "Bienvenido a clase de Historia.",
    "Fechas, batallas, imperios y personajes... ¡mi asignatura favorita!",
    "La Estrellita naranja espera a quien sepa de Roma, Egipto y Grecia.",
    "¡Estudia bien y nos vemos en el tablero!",
];

/// Diálogo del profesor de Informática.
const LINES_CS: &[&str] = &[
    "¡Hola! Yo llevo la clase de Informática.",
    "Mira los ordenadores de los pupitres: aquí se aprende haciendo.",
    "Si aciertas mis preguntas de hardware, redes y código, la Estrellita verde será tuya.",
    "¡Que no te pille desprevenido lo de los lenguajes de programación!",
];

/// Plugin de los profesores: los crea al arrancar y los mueve durante el juego.
pub struct TeacherPlugin;

impl Plugin for TeacherPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_teachers).add_systems(
            Update,
            update_teachers.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Datos de un profesor a crear: aula, colores, nombre y diálogo.
struct TeacherSpec {
    subject: &'static str,
    cx: f32,
    suit: Color,
    tie: Color,
    skin: Color,
    lines: &'static [&'static str],
}

/// Crea los tres profesores, uno por aula, junto a la mesa del profesor.
fn spawn_teachers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");

    let specs = [
        TeacherSpec {
            subject: "Matemáticas",
            cx: -8.0,
            suit: Color::srgb(0.32, 0.36, 0.42),
            tie: Color::srgb(0.28, 0.52, 0.90),
            skin: Color::srgb(0.90, 0.72, 0.55),
            lines: LINES_MATH,
        },
        TeacherSpec {
            subject: "Historia",
            cx: 0.0,
            suit: Color::srgb(0.55, 0.42, 0.28),
            tie: Color::srgb(0.85, 0.58, 0.28),
            skin: Color::srgb(0.72, 0.55, 0.42),
            lines: LINES_HISTORY,
        },
        TeacherSpec {
            subject: "Informática",
            cx: 8.0,
            suit: Color::srgb(0.30, 0.45, 0.58),
            tie: Color::srgb(0.38, 0.72, 0.45),
            skin: Color::srgb(0.95, 0.80, 0.68),
            lines: LINES_CS,
        },
    ];

    // Mallas compartidas por todos los profesores.
    let torso_mesh = meshes.add(Cuboid::new(0.5, 0.7, 0.3));
    let leg_mesh = meshes.add(Cylinder::new(0.09, 0.45));
    let arm_mesh = meshes.add(Cylinder::new(0.07, 0.55));
    let head_mesh = meshes.add(Sphere::new(0.18));
    let tie_mesh = meshes.add(Cuboid::new(0.09, 0.32, 0.04));
    let shoe_mesh = meshes.add(Cuboid::new(0.18, 0.1, 0.28));

    for spec in specs {
        let suit_mat = materials.add(StandardMaterial {
            base_color: spec.suit,
            perceptual_roughness: 0.7,
            ..default()
        });
        let tie_mat = materials.add(StandardMaterial {
            base_color: spec.tie,
            perceptual_roughness: 0.6,
            ..default()
        });
        let skin_mat = materials.add(StandardMaterial {
            base_color: spec.skin,
            perceptual_roughness: 0.5,
            ..default()
        });
        let shoe_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.11, 0.10),
            perceptual_roughness: 0.8,
            ..default()
        });

        let root = commands
            .spawn((
                Collider::new(TEACHER_HALF_EXTENTS),
                Transform::from_xyz(spec.cx, TEACHER_HALF_EXTENTS.y, -6.2),
                Visibility::default(),
            ))
            .id();

        // Grupo visual en los pies + etiqueta con la asignatura.
        let mut body_entity = Entity::PLACEHOLDER;
        commands.entity(root).with_children(|parent| {
            body_entity = parent
                .spawn((
                    Transform::from_xyz(0.0, -TEACHER_HALF_EXTENTS.y, 0.0),
                    Visibility::default(),
                ))
                .with_children(|body| {
                    // Piernas.
                    body.spawn((
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(suit_mat.clone()),
                        Transform::from_xyz(-0.14, 0.22, 0.0),
                    ));
                    body.spawn((
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(suit_mat.clone()),
                        Transform::from_xyz(0.14, 0.22, 0.0),
                    ));
                    // Torso.
                    body.spawn((
                        Mesh3d(torso_mesh.clone()),
                        MeshMaterial3d(suit_mat.clone()),
                        Transform::from_xyz(0.0, 0.65, 0.0),
                    ));
                    // Corbata del color de la asignatura.
                    body.spawn((
                        Mesh3d(tie_mesh.clone()),
                        MeshMaterial3d(tie_mat.clone()),
                        Transform::from_xyz(0.0, 0.72, 0.16),
                    ));
                    // Brazos.
                    body.spawn((
                        Mesh3d(arm_mesh.clone()),
                        MeshMaterial3d(suit_mat.clone()),
                        Transform::from_xyz(-0.38, 0.72, 0.0),
                    ));
                    body.spawn((
                        Mesh3d(arm_mesh.clone()),
                        MeshMaterial3d(suit_mat.clone()),
                        Transform::from_xyz(0.38, 0.72, 0.0),
                    ));
                    // Cabeza.
                    body.spawn((
                        Mesh3d(head_mesh.clone()),
                        MeshMaterial3d(skin_mat.clone()),
                        Transform::from_xyz(0.0, 1.1, 0.0),
                    ));
                    // Zapatos.
                    body.spawn((
                        Mesh3d(shoe_mesh.clone()),
                        MeshMaterial3d(shoe_mat.clone()),
                        Transform::from_xyz(-0.14, 0.06, 0.03),
                    ));
                    body.spawn((
                        Mesh3d(shoe_mesh.clone()),
                        MeshMaterial3d(shoe_mat.clone()),
                        Transform::from_xyz(0.14, 0.06, 0.03),
                    ));
                })
                .id();

            // Etiqueta con el nombre de la asignatura, siempre de frente.
            parent.spawn((
                Text2d::new(spec.subject),
                TextFont {
                    font: font.clone(),
                    font_size: 0.11,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.98, 0.9)),
                Transform::from_xyz(0.0, 1.62, 0.0),
            ));
        });

        commands.entity(root).insert(Teacher {
            // Entre la mesa del profesor (z ≈ -7.5) y la primera fila de
            // pupitres (z ≈ -5.2), sin chocar con ninguno.
            z_back: -6.9,
            z_front: -5.7,
            dir: 1.0,
            pause: 0.0,
            body: body_entity,
            subject: spec.subject,
            accent: spec.tie,
            lines: spec.lines,
        });
    }
}

/// Patrulla de los profesores y detección del jugador: si se acerca, se
/// detienen y se giran hacia él; si no, caminan de extremo a extremo.
fn update_teachers(
    time: Res<Time>,
    player_q: Query<&Transform, (With<Player>, Without<Teacher>)>,
    mut teachers: Query<(&mut Teacher, &mut Transform), Without<Player>>,
    mut bodies: Query<&mut Transform, (Without<Teacher>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    for (mut teacher, mut tf) in &mut teachers {
        let dx = player_tf.translation.x - tf.translation.x;
        let dz = player_tf.translation.z - tf.translation.z;
        let distance = Vec2::new(dx, dz).length();

        // ¿El jugador está cerca? Se detiene y le mira.
        if distance < NOTICE_DISTANCE {
            teacher.pause = PAUSE_AT_END;
            // Espeja el cuerpo hacia el lado del jugador (la etiqueta queda
            // siempre de frente).
            if let Ok(mut body) = bodies.get_mut(teacher.body) {
                body.scale.x = if player_tf.translation.z >= tf.translation.z {
                    1.0
                } else {
                    -1.0
                };
            }
            continue;
        }

        // Pausa al llegar a un extremo.
        if teacher.pause > 0.0 {
            teacher.pause -= dt;
            if teacher.pause <= 0.0 {
                teacher.dir = -teacher.dir;
            }
            continue;
        }

        // Avanza en la dirección actual hasta el límite del aula.
        let target_z = if teacher.dir > 0.0 {
            teacher.z_front
        } else {
            teacher.z_back
        };
        let mut next = tf.translation.z + teacher.dir * TEACHER_SPEED * dt;
        if (teacher.dir > 0.0 && next >= target_z) || (teacher.dir < 0.0 && next <= target_z) {
            next = target_z;
            teacher.pause = PAUSE_AT_END;
        }
        tf.translation.z = next;

        // Orientación: +z hacia la puerta, -z hacia la pizarra.
        if let Ok(mut body) = bodies.get_mut(teacher.body) {
            body.scale.x = teacher.dir;
        }
    }
}