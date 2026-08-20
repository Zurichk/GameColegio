//! Menú de pausa de la exploración (Fase 6).
//!
//! Con la tecla **Escape** durante el modo `Playing` se abre el menú de
//! pausa: reanudar, reiniciar la partida, volver al menú principal o salir
//! del juego. Mientras el estado es `Paused` todos los sistemas de juego
//! (movimiento, puertas, profesores, diálogos, cuestionarios) están
//! inactivos porque solo corren en `GameState::Playing`.

use bevy::prelude::*;

use crate::game::{GameState, RestartWorld, SaveGameRequested};
use crate::i18n::tr;
use crate::settings::SettingsReturn;
use crate::world::dialog::DialogSession;
use crate::world::quiz::QuizSession;

/// Raíz del menú de pausa (para poder destruirlo al salir).
#[derive(Component)]
pub struct PauseMenuUi;

/// Botón de reanudar la partida.
#[derive(Component)]
pub struct ResumeButton;

/// Botón de reiniciar la partida (jugador a la salida, puertas abiertas).
#[derive(Component)]
pub struct RestartButton;

/// Botón de volver al menú principal.
#[derive(Component)]
pub struct MainMenuButton;

/// Botón de abrir la pantalla de ajustes.
#[derive(Component)]
pub struct AjustesButton;

/// Botón de guardar la partida manualmente.
#[derive(Component)]
pub struct SaveButton;

/// Texto de confirmación "Partida guardada".
#[derive(Component)]
pub struct SaveFeedback;

/// Botón de salir del juego.
#[derive(Component)]
pub struct QuitButton;

/// Modal de confirmación "¿Seguro que quieres salir?" (oculto por defecto).
#[derive(Component)]
pub struct PauseQuitConfirmUi;

/// Botón de confirmar la salida ("Sí, salir").
#[derive(Component)]
pub struct PauseQuitYesButton;

/// Botón de cancelar la salida ("Cancelar").
#[derive(Component)]
pub struct PauseQuitNoButton;

/// Plugin del menú de pausa.
pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, open_pause.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
            .add_systems(
                Update,
                pause_input.run_if(in_state(GameState::Paused)),
            );
    }
}

/// Abre la pausa con Escape. Si hay un diálogo o un cuestionario abierto,
/// Escape se deja para esas interacciones (avanzar/cerrar) y no pausa.
fn open_pause(
    keys: Res<ButtonInput<KeyCode>>,
    dialog: Option<Res<DialogSession>>,
    quiz: Option<Res<QuizSession>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if dialog.is_some() || quiz.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

/// Construye el menú de pausa: capa oscura + panel con título y botones.
fn spawn_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Fuente con acentos del español (el load es idempotente).
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            PauseMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.82)),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text("PAUSA", 60.0, Color::srgb(1.0, 0.90, 0.50), &font));
            root.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });
            spawn_button(root, "Reanudar", ResumeButton, &font);
            spawn_button(root, "Guardar partida", SaveButton, &font);
            spawn_button(root, "Reiniciar partida", RestartButton, &font);
            spawn_button(root, "Ajustes", AjustesButton, &font);
            spawn_button(root, "Menú principal", MainMenuButton, &font);
            spawn_button(root, "Salir del juego", QuitButton, &font);
            root.spawn((
                SaveFeedback,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.50, 0.90, 0.60)),
            ));

            // Modal de confirmación de salida (oculto hasta pulsar "Salir").
            root.spawn((
                PauseQuitConfirmUi,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                Visibility::Hidden,
                ZIndex(40),
            ))
            .with_children(|modal| {
                modal
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::axes(Val::Px(30.0), Val::Px(26.0)),
                            row_gap: Val::Px(18.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.12, 0.22, 0.98)),
                        BorderRadius::all(Val::Px(14.0)),
                    ))
                    .with_children(|panel| {
                        panel.spawn(ui_text(
                            "¿Seguro que quieres salir del juego?",
                            27.0,
                            Color::WHITE,
                            &font,
                        ));
                        panel
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(16.0),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_button(row, "Sí, salir", PauseQuitYesButton, &font);
                                spawn_button(row, "Cancelar", PauseQuitNoButton, &font);
                            });
                    });
            });
        });
}

/// Destruye el menú de pausa.
fn despawn_pause_menu(mut commands: Commands, roots: Query<Entity, With<PauseMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics y teclas del menú de pausa.
fn pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
    mut restart: EventWriter<RestartWorld>,
    mut save: EventWriter<SaveGameRequested>,
    mut commands: Commands,
    mut feedback: Query<&mut Text, With<SaveFeedback>>,
    mut quit_modal: Query<&mut Visibility, With<PauseQuitConfirmUi>>,
    interactions: Query<
        (
            &Interaction,
            Option<&ResumeButton>,
            Option<&SaveButton>,
            Option<&RestartButton>,
            Option<&AjustesButton>,
            Option<&MainMenuButton>,
            Option<&QuitButton>,
            Option<&PauseQuitYesButton>,
            Option<&PauseQuitNoButton>,
        ),
        Changed<Interaction>,
    >,
) {
    // Esc cierra primero el modal de salida; si no está abierto, reanuda.
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut vis) = quit_modal.single_mut() {
            if *vis == Visibility::Visible {
                *vis = Visibility::Hidden;
                return;
            }
        }
        next_state.set(GameState::Playing);
        return;
    }

    let mut modal_open = quit_modal
        .single()
        .map_or(false, |vis| *vis == Visibility::Visible);
    for (
        interaction,
        resume,
        save_btn,
        restart_btn,
        ajustes,
        main_menu,
        quit,
        quit_yes,
        quit_no,
    ) in &interactions
    {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if modal_open {
            // Solo se atienden los botones del modal de confirmación.
            if quit_yes.is_some() {
                // Último guardado automático antes de salir.
                save.write(SaveGameRequested);
                app_exit.write(AppExit::Success);
            } else if quit_no.is_some() {
                if let Ok(mut vis) = quit_modal.single_mut() {
                    *vis = Visibility::Hidden;
                }
                modal_open = false;
            }
            continue;
        }
        if quit.is_some() {
            if let Ok(mut vis) = quit_modal.single_mut() {
                *vis = Visibility::Visible;
            }
            modal_open = true;
        } else if resume.is_some() {
            next_state.set(GameState::Playing);
        } else if save_btn.is_some() {
            // Guardado manual: escribe `savegame.json` y lo confirma.
            save.write(SaveGameRequested);
            if let Ok(mut text) = feedback.single_mut() {
                *text = Text::new(tr("✓ Partida guardada"));
            }
        } else if restart_btn.is_some() {
            // Reinicia el mundo (jugador a la salida, puertas abiertas y
            // sesiones limpias) y vuelve a la exploración.
            restart.write(RestartWorld);
            next_state.set(GameState::Playing);
        } else if ajustes.is_some() {
            // Ajustes: al volver se reabre la pausa.
            commands.insert_resource(SettingsReturn(GameState::Paused));
            next_state.set(GameState::Settings);
        } else if main_menu.is_some() {
            // Limpia sesiones para no arrastrar un diálogo/cuestionario al
            // volver a entrar en el colegio. Al llegar al menú, el
            // guardado automático escribe el progreso.
            commands.remove_resource::<DialogSession>();
            commands.remove_resource::<QuizSession>();
            next_state.set(GameState::MainMenu);
        }
    }
}

/// Crea un botón grande con texto centrado dentro de `parent`.
fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(300.0),
                height: Val::Px(56.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.20, 0.38, 0.66)),
            BorderColor(Color::srgb(0.60, 0.80, 1.0)),
            BorderRadius::all(Val::Px(10.0)),
            // Inherited: en Bevy 0.16 `Visible` se muestra aunque el padre esté
            // oculto (rompería el modal de confirmación de salida).
            Visibility::Inherited,
            marker,
        ))
        .with_children(|button| {
            button.spawn(ui_text(label, 26.0, Color::WHITE, font));
        });
}

/// Crea un texto con la fuente de interfaz.
fn ui_text(label: &str, size: f32, color: Color, font: &Handle<Font>) -> (Text, TextFont, TextColor) {
    (
        Text::new(tr(label)),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}
