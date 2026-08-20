//! HUD de la exploración (Fase 8): sala actual y asignaturas superadas.
//!
//! Se muestra arriba a la izquierda mientras el estado es `Playing`: una
//! etiqueta con la sala donde está el jugador y una fila de chips con las
//! asignaturas del colegio (✓ en verde las superadas).

use bevy::prelude::*;

use crate::game::GameState;
use crate::i18n::tr;
use crate::player::Player;
use crate::save::Progress;

/// Límite del edificio en el eje X (coincide con `school.rs`).
const BUILDING_HALF_X: f32 = 12.0;
/// Pared trasera de las aulas.
const BACK_WALL_Z: f32 = -9.0;
/// Pared frontal de las aulas (z = 0).
const CLASSROOM_FRONT_Z: f32 = 0.0;
/// Pared frontal de la recepción.
const RECEPTION_FRONT_Z: f32 = 10.0;
/// Grosor de las paredes.
const WALL_THICKNESS: f32 = 0.3;
/// A partir de esta z se considera "Recepción" (donde está el mostrador).
const RECEPTION_START_Z: f32 = 8.0;

/// Las tres asignaturas del colegio, en orden de los chips del HUD.
pub const SUBJECTS: [&str; 3] = ["Matemáticas", "Historia", "Informática"];

/// Raíz del HUD (para destruirlo al salir de la exploración).
#[derive(Component)]
pub struct HudUi;

/// Etiqueta con la sala actual.
#[derive(Component)]
pub struct RoomLabel;

/// Chip de una asignatura (índice 0..3 en `SUBJECTS`).
#[derive(Component)]
pub struct SubjectChip(usize);

/// Nombre de la sala según la posición del jugador.
pub fn room_name(x: f32, z: f32) -> &'static str {
    // Fuera del edificio: patio/jardín.
    if x.abs() > BUILDING_HALF_X
        || z < BACK_WALL_Z
        || z > RECEPTION_FRONT_Z + WALL_THICKNESS
    {
        return "Patio";
    }
    if z < CLASSROOM_FRONT_Z {
        // Dentro de las aulas (separadores en x = -4 y x = 4).
        if x <= -4.0 {
            "Aula de Matemáticas"
        } else if x >= 4.0 {
            "Aula de Informática"
        } else {
            "Aula de Historia"
        }
    } else if z >= RECEPTION_START_Z {
        "Recepción"
    } else {
        "Pasillo"
    }
}

/// Plugin del HUD.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(OnExit(GameState::Playing), despawn_hud)
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)));
    }
}

/// Construye el HUD: sala actual + chips de asignaturas.
fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            HudUi,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(8.0),
                ..default()
            },
            ZIndex(20),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn((
                RoomLabel,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 26.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.90, 0.70)),
                Node {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.07, 0.16, 0.85)),
                BorderRadius::all(Val::Px(8.0)),
            ));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|chips| {
                for (i, subject) in SUBJECTS.iter().enumerate() {
                    chips.spawn((
                        SubjectChip(i),
                        Text::new(tr(subject)),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.75, 0.78, 0.85)),
                        Node {
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.07, 0.16, 0.85)),
                        BorderRadius::all(Val::Px(6.0)),
                    ));
                }
            });
        });
}

/// Destruye el HUD.
fn despawn_hud(mut commands: Commands, roots: Query<Entity, With<HudUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Actualiza la sala actual y los chips según la posición y el progreso.
fn update_hud(
    player_q: Query<&Transform, With<Player>>,
    progress: Res<Progress>,
    mut room: Query<&mut Text, (With<RoomLabel>, Without<SubjectChip>)>,
    mut chips: Query<(&SubjectChip, &mut Text, &mut TextColor)>,
) {
    if let Ok(tf) = player_q.single() {
        if let Ok(mut room_text) = room.single_mut() {
            *room_text = Text::new(tr(room_name(tf.translation.x, tf.translation.z)));
        }
    }
    for (chip, mut text, mut color) in &mut chips {
        let subject = SUBJECTS[chip.0];
        if progress.has_passed(subject) {
            *text = Text::new(tr("✓ {subject}").replace("{subject}", subject));
            *color = TextColor(Color::srgb(0.50, 0.90, 0.55));
        } else {
            *text = Text::new(tr(subject));
            *color = TextColor(Color::srgb(0.75, 0.78, 0.85));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_detection() {
        // Aulas.
        assert_eq!(room_name(-8.0, -4.0), "Aula de Matemáticas");
        assert_eq!(room_name(0.0, -4.0), "Aula de Historia");
        assert_eq!(room_name(8.0, -4.0), "Aula de Informática");
        // Pasillo.
        assert_eq!(room_name(0.0, 3.0), "Pasillo");
        // Recepción.
        assert_eq!(room_name(0.0, 9.0), "Recepción");
        // Exterior (patio).
        assert_eq!(room_name(0.0, 16.0), "Patio");
        assert_eq!(room_name(20.0, 0.0), "Patio");
    }
}