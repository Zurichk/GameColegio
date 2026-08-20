//! Memoria de secuencia (estilo Simón): una secuencia de colores se ilumina
//! y hay que repetirla con clics. Con cada ronda completada la secuencia crece
//! (hasta 8 colores); si se falla, la secuencia se vuelve a mostrar.
//!
//! Accesible desde el menú "Juegos de memoria" (estado `MemorySequence`).

use bevy::prelude::*;
use rand::Rng;

use super::{screen_background, spawn_button, ui_text};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Número máximo de colores de la secuencia para ganar.
const MAX_SEQUENCE: usize = 8;
/// Duración (s) de cada destello al mostrar la secuencia.
const FLASH_TIME: f32 = 0.55;
/// Duración (s) del aviso tras fallar.
const WRONG_TIME: f32 = 1.2;

/// Colores de los 4 botones del juego.
const BUTTON_COLORS: [Color; 4] = [
    Color::srgb(0.85, 0.25, 0.25), // rojo
    Color::srgb(0.25, 0.72, 0.30), // verde
    Color::srgb(0.25, 0.45, 0.90), // azul
    Color::srgb(0.90, 0.78, 0.20), // amarillo
];

/// Nombres de los colores (para los mensajes).
const COLOR_NAMES: [&str; 4] = ["rojo", "verde", "azul", "amarillo"];

/// Fases de la partida de secuencia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SequencePhase {
    /// La máquina está iluminando la secuencia.
    Show,
    /// El jugador debe repetir la secuencia.
    Input,
    /// Se ha fallado: aviso antes de volver a mostrar.
    Wrong,
    /// Partida ganada.
    Won,
}

/// Partida de memoria de secuencia activa.
#[derive(Resource)]
pub struct SequenceSession {
    sequence: Vec<usize>,
    phase: SequencePhase,
    /// Índice del elemento que se está mostrando.
    show_index: usize,
    /// Temporizador del destello actual.
    show_timer: f32,
    /// Botón que se ilumina ahora mismo.
    flash: Option<usize>,
    /// Posición de la secuencia que debe pulsar el jugador.
    input_index: usize,
    /// Rondas completadas (longitud de la secuencia actual).
    round: usize,
    /// Pulsaciones correctas de esta ronda.
    moves: u32,
    /// Segundos transcurridos de la partida.
    elapsed: f32,
    /// Temporizador del aviso de error.
    wrong_timer: f32,
}

impl Default for SequenceSession {
    fn default() -> Self {
        let mut session = SequenceSession {
            sequence: vec![rand::thread_rng().gen_range(0..4)],
            phase: SequencePhase::Show,
            show_index: 0,
            show_timer: 0.0,
            flash: None,
            input_index: 0,
            round: 1,
            moves: 0,
            elapsed: 0.0,
            wrong_timer: 0.0,
        };
        session.start_flash();
        session
    }
}

impl SequenceSession {
    /// Inicia el destello del siguiente elemento de la secuencia.
    fn start_flash(&mut self) {
        self.phase = SequencePhase::Show;
        self.show_index = 0;
        self.show_timer = FLASH_TIME;
        self.flash = Some(self.sequence[0]);
        self.input_index = 0;
    }

    /// Avanza el destello un paso (o pasa a la fase de entrada).
    fn advance_flash(&mut self) {
        self.show_index += 1;
        if self.show_index >= self.sequence.len() {
            self.phase = SequencePhase::Input;
            self.flash = None;
            self.input_index = 0;
        } else {
            self.show_timer = FLASH_TIME;
            self.flash = Some(self.sequence[self.show_index]);
        }
    }

    /// Añade un color nuevo a la secuencia (ronda superada).
    fn grow(&mut self) {
        self.sequence.push(rand::thread_rng().gen_range(0..4));
        self.round = self.sequence.len();
        self.moves = 0;
    }
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla de secuencia.
#[derive(Component)]
pub struct SequenceUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct SeqText(SeqField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum SeqField {
    Title,
    Stats,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Botón de color (índice 0..4).
#[derive(Component)]
pub struct SeqButton(pub usize);

/// Botón de volver al menú de Juegos de memoria.
#[derive(Component)]
pub struct SeqBackButton;

/// Botón de jugar otra vez.
#[derive(Component)]
pub struct SeqAgainButton;

/// Contenedor de resultados (oculto hasta ganar).
#[derive(Component)]
pub struct SeqResultBox;

/// Plugin del juego de memoria de secuencia.
pub struct SequencePlugin;

impl Plugin for SequencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MemorySequence), spawn_sequence_ui)
            .add_systems(OnExit(GameState::MemorySequence), cleanup_sequence)
            .add_systems(
                Update,
                update_sequence.run_if(in_state(GameState::MemorySequence)),
            );
    }
}

/// Crea un texto del campo indicado.
fn seq_text(
    parent: &mut ChildSpawnerCommands,
    field: SeqField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        SeqText(field),
        Text::new(text.to_string()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout {
            linebreak: LineBreak::WordBoundary,
            ..default()
        },
        Node {
            max_width: Val::Px(620.0),
            ..default()
        },
    ));
}

/// Construye la pantalla del juego de secuencia.
fn spawn_sequence_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(SequenceSession::default());

    commands
        .spawn((
            SequenceUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            screen_background(),
            Visibility::Visible,
            ZIndex(30),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(520.0),
                        padding: UiRect::axes(Val::Px(26.0), Val::Px(20.0)),
                        row_gap: Val::Px(14.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    seq_text(panel, SeqField::Title, "MEMORIA DE SECUENCIA", 26.0, &font);
                    seq_text(panel, SeqField::Stats, "", 18.0, &font);
                    seq_text(panel, SeqField::Feedback, "", 22.0, &font);

                    // Tablero 2x2 de botones de color.
                    panel
                        .spawn((
                            Node {
                                width: Val::Px(440.0),
                                flex_wrap: FlexWrap::Wrap,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(16.0),
                                row_gap: Val::Px(16.0),
                                ..default()
                            },
                        ))
                        .with_children(|board| {
                            for index in 0..4 {
                                board
                                    .spawn((
                                        Button,
                                        SeqButton(index),
                                        Node {
                                            width: Val::Px(200.0),
                                            height: Val::Px(150.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(3.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_COLORS[index]),
                                        BorderColor(Color::srgb(0.95, 0.95, 0.95)),
                                        BorderRadius::all(Val::Px(14.0)),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn(ui_text(COLOR_NAMES[index], 24.0, Color::WHITE, &font));
                                    });
                            }
                        });

                    // Resultados (ocultos hasta ganar).
                    panel
                        .spawn((
                            SeqResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            seq_text(results, SeqField::ResultTitle, "", 26.0, &font);
                            seq_text(results, SeqField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Jugar otra vez", SeqAgainButton, &font);
                            spawn_button(results, "Volver a Juegos de memoria", SeqBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_sequence(mut commands: Commands, roots: Query<Entity, With<SequenceUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<SequenceSession>();
}

/// Color de un botón según si se está iluminando o está apagado.
fn button_color(session: &SequenceSession, index: usize) -> Color {
    let base = BUTTON_COLORS[index].to_srgba();
    if session.flash == Some(index) {
        // Destello: aclarar hacia blanco.
        Color::srgb(
            (base.red + 0.45).min(1.0),
            (base.green + 0.45).min(1.0),
            (base.blue + 0.45).min(1.0),
        )
    } else {
        // Apagado: oscurecer un poco.
        Color::srgb(base.red * 0.55, base.green * 0.55, base.blue * 0.55)
    }
}

/// Gestiona la partida: mostrar secuencia, entrada del jugador y victoria.
fn update_sequence(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    mut session: ResMut<SequenceSession>,
    mut texts: Query<(&SeqText, &mut Text, &mut TextColor, &mut Visibility)>,
    mut button_colors: Query<
        (&SeqButton, &mut BackgroundColor),
        (Without<SeqText>, Without<SeqResultBox>),
    >,
    button_clicks: Query<
        (&Interaction, &SeqButton),
        (Changed<Interaction>, Without<SeqText>, Without<SeqResultBox>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (With<SeqResultBox>, Without<SeqText>, Without<SeqButton>),
    >,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<SeqBackButton>)>,
    again_clicks: Query<&Interaction, (Changed<Interaction>, With<SeqAgainButton>)>,
) {
    // Escape: volver al menú de Juegos de memoria.
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::MemoryMenu);
        return;
    }

    let dt = time.delta_secs();
    if session.phase != SequencePhase::Won {
        session.elapsed += dt;
    }

    // 1) Resultados (partida ganada).
    if session.phase == SequencePhase::Won {
        let back = back_clicks.single().map_or(false, |i| *i == Interaction::Pressed)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::KeyQ);
        let again = again_clicks
            .single()
            .map_or(false, |i| *i == Interaction::Pressed);
        if back {
            commands.set_state(GameState::MemoryMenu);
            return;
        }
        if again {
            *session = SequenceSession::default();
            if let Ok(mut vis) = result_box.single_mut() {
                *vis = Visibility::Hidden;
            }
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                SeqField::ResultTitle => {
                    *text = Text::new(tr("¡Memoria increíble!"));
                    *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                    *vis = Visibility::Visible;
                }
                SeqField::ResultDetail => {
                    *text = Text::new(tr("Secuencia de {} colores · Tiempo: {} s · Pulsaciones: {}").replace("{}", &MAX_SEQUENCE.to_string()).replace("{}", &format!("{:.0}", session.elapsed)).replace("{}", &session.moves.to_string()));
                    *vis = Visibility::Visible;
                }
                _ => {}
            }
        }
        if let Ok(mut vis) = result_box.single_mut() {
            *vis = Visibility::Visible;
        }
        return;
    }

    // 2) Mostrar la secuencia.
    if session.phase == SequencePhase::Show {
        session.show_timer -= dt;
        if session.show_timer <= 0.0 {
            session.advance_flash();
        }
    }

    // 3) Entrada del jugador.
    if session.phase == SequencePhase::Input {
        let mut chosen: Option<usize> = None;
        for (interaction, button) in &button_clicks {
            if *interaction == Interaction::Pressed {
                chosen = Some(button.0);
                break;
            }
        }
        if let Some(index) = chosen {
            session.moves += 1;
            play_click(&mut commands, &sfx);
            if index == session.sequence[session.input_index] {
                session.input_index += 1;
                if session.input_index >= session.sequence.len() {
                    // Ronda completada.
                    if session.sequence.len() >= MAX_SEQUENCE {
                        session.phase = SequencePhase::Won;
                        play_success(&mut commands, &sfx);
                    } else {
                        session.grow();
                        session.start_flash();
                        play_success(&mut commands, &sfx);
                    }
                }
            } else {
                // Fallo: aviso y se vuelve a mostrar la misma secuencia.
                session.phase = SequencePhase::Wrong;
                session.wrong_timer = WRONG_TIME;
                session.input_index = 0;
            }
        }
    }

    // 4) Aviso de error.
    if session.phase == SequencePhase::Wrong {
        session.wrong_timer -= dt;
        if session.wrong_timer <= 0.0 {
            session.start_flash();
        }
    }

    // 5) Textos.
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            SeqField::Stats => {
                *text = Text::new(tr("Ronda {}/{} · Secuencia: {} · Pulsaciones: {}").replace("{}", &session.sequence.len().min(MAX_SEQUENCE).to_string()).replace("{}", &MAX_SEQUENCE.to_string()).replace("{}", &session.sequence.iter().map(|i| tr(COLOR_NAMES[*i])).collect::<Vec<_>>().join(" → ")).replace("{}", &session.moves.to_string()));
                *vis = Visibility::Visible;
            }
            SeqField::Feedback => {
                match session.phase {
                    SequencePhase::Show => {
                        *text = Text::new(tr("¡Mira con atención..."));
                        *color = TextColor(Color::srgb(0.85, 0.90, 1.0));
                        *vis = Visibility::Visible;
                    }
                    SequencePhase::Input => {
                        *text = Text::new(tr("¡Ahora repite la secuencia!"));
                        *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                        *vis = Visibility::Visible;
                    }
                    SequencePhase::Wrong => {
                        *text = Text::new(tr("¡Cuidado! Vuelve a mirar..."));
                        *color = TextColor(Color::srgb(0.95, 0.40, 0.40));
                        *vis = Visibility::Visible;
                    }
                    SequencePhase::Won => {
                        *vis = Visibility::Hidden;
                    }
                }
            }
            _ => {}
        }
    }

    // 6) Colores de los botones.
    for (button, mut bg) in &mut button_colors {
        *bg = BackgroundColor(button_color(&session, button.0));
    }
}