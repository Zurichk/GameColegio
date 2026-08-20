//! Plugin del mundo: iluminación, suelo y edificio del colegio.

pub mod collision;
pub mod dialog;
pub mod quiz;
mod school;
mod teacher;
mod textures;

use bevy::prelude::*;

use crate::audio::{play_door, Sfx};
use crate::game::{GameState, RestartWorld, RestoreWorld};
use crate::i18n::tr;
use crate::player::Player;
use crate::save::Progress;

use self::dialog::{DialogPlugin, DialogSession};
use self::quiz::{QuizPlugin, QuizSession};
use self::school::SchoolPlugin;
use self::teacher::TeacherPlugin;

/// Re-export para que otros módulos (p. ej. `save`) puedan usar `Door`.
pub use self::school::Door;

/// Re-export para que otros módulos (p. ej. `fx`) puedan usar `Teacher`.
pub use self::teacher::Teacher;

/// Plugin encargado de construir y mantener el mundo del juego.
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SchoolPlugin, TeacherPlugin, DialogPlugin, QuizPlugin))
            .insert_resource(AmbientLight {
                color: Color::srgb(0.98, 0.97, 0.95),
                brightness: 220.0,
                ..default()
            })
            .add_systems(Startup, (setup_lighting, spawn_door_prompt))
            .add_systems(
                Update,
                (
                    toggle_doors,
                    update_door_prompt,
                    reset_world_state,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                restore_world_state.run_if(on_event::<RestoreWorld>),
            );
    }
}

/// Aviso en pantalla que indica cómo abrir/cerrar la puerta cercana.
#[derive(Component)]
pub struct DoorPrompt;

/// Crea el aviso de interacción (oculto hasta que el jugador se acerca a
/// una puerta). Usa la misma fuente con acentos que el resto de la UI.
fn spawn_door_prompt(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            DoorPrompt,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(48.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            ZIndex(10),
        ))
        .with_children(|prompt| {
            prompt.spawn((
                Text::new(tr("E — Abrir puerta")),
                TextFont {
                    font: font.clone(),
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.75)),
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.07, 0.16, 0.85)),
                BorderRadius::all(Val::Px(10.0)),
            ));
        });
}

/// Distancia máxima (metros) para interactuar con una puerta.
const DOOR_INTERACT_DISTANCE: f32 = 2.6;

/// Abre o cierra la puerta más cercana con la tecla E (Fase 2: puertas e
/// interacción). La puerta se desliza hacia el lado y su colisión la
/// sigue, por lo que bloquea el vano solo cuando está cerrada.
fn toggle_doors(
    keys: Res<ButtonInput<KeyCode>>,
    dialog: Option<Res<DialogSession>>,
    quiz: Option<Res<QuizSession>>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    player_q: Query<&Transform, With<Player>>,
    mut doors: Query<(Entity, &mut Door, &mut Transform), Without<Player>>,
) {
    // Mientras se habla con un profesor o se hace un cuestionario no se
    // interactúa con puertas.
    if dialog.is_some() || quiz.is_some() || !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(player) = player_q.single() else {
        return;
    };
    for (_entity, mut door, mut tf) in &mut doors {
        let dist =
            Vec2::new(tf.translation.x - player.translation.x, tf.translation.z - player.translation.z)
                .length();
        if dist < DOOR_INTERACT_DISTANCE {
            // Una puerta por pulsación.
            door.open = !door.open;
            tf.translation.x = if door.open { door.open_x } else { door.closed_x };
            play_door(&mut commands, &sfx);
            return;
        }
    }
}

/// Muestra u oculta el aviso "E — Abrir/Cerrar puerta" según la puerta más
/// cercana al jugador.
fn update_door_prompt(
    player_q: Query<&Transform, With<Player>>,
    doors: Query<(&Door, &Transform)>,
    mut prompt: Query<(&mut Visibility, &mut Text), With<DoorPrompt>>,
) {
    let Ok(player) = player_q.single() else {
        return;
    };
    let mut nearest: Option<(f32, bool)> = None;
    for (door, tf) in &doors {
        let dist = Vec2::new(tf.translation.x - player.translation.x, tf.translation.z - player.translation.z)
            .length();
        if dist < DOOR_INTERACT_DISTANCE && nearest.map_or(true, |(d, _)| dist < d) {
            nearest = Some((dist, door.open));
        }
    }
    let Ok((mut visibility, mut text)) = prompt.single_mut() else {
        return;
    };
    match nearest {
        Some((_, open)) => {
            *text = Text::new(tr(if open {
                "E — Cerrar puerta"
            } else {
                "E — Abrir puerta"
            }));
            *visibility = Visibility::Visible;
        }
        None => *visibility = Visibility::Hidden,
    }
}

/// Crea la iluminación principal de la escena.
fn setup_lighting(mut commands: Commands) {
    // Sol direccional con sombras.
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.5, 0.0)),
    ));

    // Luces cálidas de interior: una por aula, más pasillo y recepción.
    let interior_lights = [
        (-8.0, 2.8, -4.5), // Aula de Matemáticas.
        (0.0, 2.8, -4.5),  // Aula de Historia.
        (8.0, 2.8, -4.5),  // Aula de Informática.
        (0.0, 2.8, 3.0),   // Pasillo.
        (0.0, 2.8, 8.0),   // Recepción.
    ];
    for (x, y, z) in interior_lights {
        commands.spawn((
            PointLight {
                intensity: 2500.0,
                range: 16.0,
                color: Color::srgb(1.0, 0.95, 0.85),
                ..default()
            },
            Transform::from_xyz(x, y, z),
        ));
    }
}

/// Reinicia el mundo de exploración al llegar un evento `RestartWorld`
/// (botón "Reiniciar partida" de la pausa): deja todas las puertas abiertas
/// (deslizadas en la pared) y limpia diálogos y cuestionarios en curso.
fn reset_world_state(
    mut doors: Query<(&mut Door, &mut Transform), Without<Player>>,
    mut commands: Commands,
    mut restart: EventReader<RestartWorld>,
) {
    let mut triggered = false;
    for _ in restart.read() {
        triggered = true;
    }
    if !triggered {
        return;
    }
    for (mut door, mut tf) in &mut doors {
        door.open = true;
        tf.translation.x = door.open_x;
    }
    commands.remove_resource::<DialogSession>();
    commands.remove_resource::<QuizSession>();
}

/// Restaura el estado del mundo al llegar un evento `RestoreWorld` (botón
/// "Continuar" del menú principal): aplica a cada puerta su estado guardado
/// (por id) y limpia diálogos y cuestionarios en curso.
fn restore_world_state(
    progress: Option<Res<Progress>>,
    mut doors: Query<(&mut Door, &mut Transform), Without<Player>>,
    mut commands: Commands,
    mut restore: EventReader<RestoreWorld>,
) {
    let mut triggered = false;
    for _ in restore.read() {
        triggered = true;
    }
    if !triggered {
        return;
    }
    if let Some(progress) = progress {
        for (mut door, mut tf) in &mut doors {
            if let Some(saved) = progress.doors.iter().find(|saved| saved.id == door.id) {
                door.open = saved.open;
                tf.translation.x = saved.x;
            }
        }
    }
    commands.remove_resource::<DialogSession>();
    commands.remove_resource::<QuizSession>();
}