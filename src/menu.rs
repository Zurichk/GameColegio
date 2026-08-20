//! Menú principal del juego.

use bevy::prelude::*;

use crate::game::{GameState, RestoreWorld};
use crate::i18n::tr;
use crate::save::{save_exists, try_load_save};
use crate::settings::SettingsReturn;

/// Raíz del menú principal (para poder destruirlo al salir).
#[derive(Component)]
pub struct MainMenuUi;

/// Botón de explorar el colegio libremente.
#[derive(Component)]
pub struct ExploreButton;

/// Botón de abrir el modo tablero.
#[derive(Component)]
pub struct BoardModeButton;

/// Botón de continuar con la partida guardada (solo visible si hay guardado).
#[derive(Component)]
pub struct ContinueButton;

/// Botón de abrir la pantalla de ajustes.
#[derive(Component)]
pub struct AjustesButton;

/// Botón de abrir la zona de aprendizaje (Lengua, Matemáticas, Ciencias, Memoria).
#[derive(Component)]
pub struct LearningButton;

/// Botón de salir del juego.
#[derive(Component)]
pub struct QuitButton;

/// Modal de confirmación "¿Seguro que quieres salir?" (oculto por defecto).
#[derive(Component)]
pub struct QuitConfirmUi;

/// Botón de confirmar la salida ("Sí, salir").
#[derive(Component)]
pub struct QuitYesButton;

/// Botón de cancelar la salida ("Cancelar").
#[derive(Component)]
pub struct QuitNoButton;

/// Plugin del menú principal.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), despawn_main_menu)
            .add_systems(
                Update,
                menu_input.run_if(in_state(GameState::MainMenu)),
            );
    }
}

/// Construye el menú principal.
fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Fuente con acentos del español (el load es idempotente).
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            MainMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.80)),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(ui_text("GAMECOLEGIO", 78.0, Color::srgb(1.0, 0.90, 0.50), &font));
            root.spawn(ui_text(
                "Aprende jugando en tu colegio",
                26.0,
                Color::srgb(0.85, 0.90, 1.0),
                &font,
            ));
            root.spawn(Node {
                height: Val::Px(24.0),
                ..default()
            });
            spawn_button(root, "Explorar el colegio", ExploreButton, &font);
            // "Continuar" solo se muestra si existe una partida guardada.
            if save_exists() {
                spawn_button(root, "Continuar", ContinueButton, &font);
            }
            spawn_button(root, "Modo Tablero", BoardModeButton, &font);
            spawn_button(root, "Zona de aprendizaje", LearningButton, &font);
            spawn_button(root, "Ajustes", AjustesButton, &font);
            spawn_button(root, "Salir del juego", QuitButton, &font);

            // Modal de confirmación de salida (oculto hasta pulsar "Salir").
            root.spawn((
                QuitConfirmUi,
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
                                spawn_button(row, "Sí, salir", QuitYesButton, &font);
                                spawn_button(row, "Cancelar", QuitNoButton, &font);
                            });
                    });
            });
        });
}

/// Destruye el menú principal.
fn despawn_main_menu(mut commands: Commands, roots: Query<Entity, With<MainMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

/// Procesa los clics del menú principal.
fn menu_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: EventWriter<AppExit>,
    mut restore: EventWriter<RestoreWorld>,
    mut quit_modal: Query<&mut Visibility, With<QuitConfirmUi>>,
    interactions: Query<
        (
            &Interaction,
            Option<&ExploreButton>,
            Option<&BoardModeButton>,
            Option<&ContinueButton>,
            Option<&LearningButton>,
            Option<&AjustesButton>,
            Option<&QuitButton>,
            Option<&QuitYesButton>,
            Option<&QuitNoButton>,
        ),
        Changed<Interaction>,
    >,
) {
    // Esc con el modal abierto cancela la salida.
    if keys.just_pressed(KeyCode::Escape) {
        if let Ok(mut vis) = quit_modal.single_mut() {
            if *vis == Visibility::Visible {
                *vis = Visibility::Hidden;
                return;
            }
        }
    }

    let mut modal_open = quit_modal
        .single()
        .map_or(false, |vis| *vis == Visibility::Visible);
    for (
        interaction,
        explore,
        board,
        continue_btn,
        learning,
        ajustes,
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
        } else if explore.is_some() {
            next_state.set(GameState::Playing);
        } else if board.is_some() {
            next_state.set(GameState::BoardSetup);
        } else if continue_btn.is_some() {
            // Carga la partida guardada y restaura jugador/puertas.
            if let Some(progress) = try_load_save() {
                commands.insert_resource(progress);
                restore.write(RestoreWorld);
                next_state.set(GameState::Playing);
            }
        } else if learning.is_some() {
            next_state.set(GameState::LearningMenu);
        } else if ajustes.is_some() {
            commands.insert_resource(SettingsReturn(GameState::MainMenu));
            next_state.set(GameState::Settings);
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
                width: Val::Px(280.0),
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