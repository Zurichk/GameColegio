//! Cuestionarios de ciencias naturales y de geografía de España (sección
//! Ciencias).
//!
//! Se elige el tipo en el menú de Ciencias y se genera una sesión de 10
//! preguntas al azar con 4 opciones cada una. Cada respuesta da un feedback
//! de 1,4 s y al final se muestra el marcador con la opción de volver.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Número de preguntas por sesión.
const ROUNDS: usize = 10;
/// Duración (s) del feedback.
const FEEDBACK_SECONDS: f32 = 1.4;

/// Tipo de trivia que se va a jugar.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaKind {
    Science,
    Geography,
}

impl TriviaKind {
    /// Título de la pantalla.
    pub fn title(self) -> &'static str {
        match self {
            TriviaKind::Science => "CIENCIAS NATURALES",
            TriviaKind::Geography => "GEOGRAFÍA DE ESPAÑA",
        }
    }
}

/// Una pregunta con sus 4 opciones y el índice de la correcta.
#[derive(Clone, Copy)]
struct TriviaQuestion {
    question: &'static str,
    options: [&'static str; 4],
    correct: usize,
}

/// Sesión de trivia activa.
#[derive(Resource)]
pub struct TriviaSession {
    kind: TriviaKind,
    rounds: Vec<TriviaQuestion>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla de trivia.
#[derive(Component)]
pub struct TriviaUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct TriviaText(TriviaField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum TriviaField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Texto de una opción (hijo de su botón).
#[derive(Component)]
pub struct TriviaOptionText(pub usize);

/// Botón de una opción (A/B/C/D).
#[derive(Component)]
pub struct TriviaOptionButton(pub usize);

/// Contenedor de resultados (oculto hasta terminar).
#[derive(Component)]
pub struct TriviaResultBox;

/// Botón de volver al menú de Ciencias.
#[derive(Component)]
pub struct TriviaBackButton;

/// Plugin de la trivia (ciencias naturales y geografía).
pub struct TriviaPlugin;

impl Plugin for TriviaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SciencePractice), spawn_trivia_ui)
            .add_systems(OnEnter(GameState::GeographyPractice), spawn_trivia_ui)
            .add_systems(
                OnExit(GameState::SciencePractice),
                cleanup_trivia,
            )
            .add_systems(
                OnExit(GameState::GeographyPractice),
                cleanup_trivia,
            )
            .add_systems(
                Update,
                update_trivia.run_if(
                    in_state(GameState::SciencePractice)
                        .or(in_state(GameState::GeographyPractice)),
                ),
            );
    }
}

/// Letras de las opciones.
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// Crea un texto del campo indicado.
fn trivia_text(
    parent: &mut ChildSpawnerCommands,
    field: TriviaField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        TriviaText(field),
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
            max_width: Val::Px(700.0),
            ..default()
        },
    ));
}

/// Crea el texto de una opción (hijo de su botón).
fn trivia_option_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        TriviaOptionText(index),
        Text::new(String::new()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Construye la pantalla de trivia.
fn spawn_trivia_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    kind: Option<Res<TriviaKind>>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    let kind = kind.map(|k| *k).unwrap_or(TriviaKind::Science);
    commands.insert_resource(TriviaSession {
        kind,
        rounds: build_rounds(kind),
        index: 0,
        correct: 0,
        wrong: 0,
        selected: None,
        feedback: false,
        feedback_timer: 0.0,
        done: false,
    });
    commands
        .spawn((
            TriviaUiRoot,
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
                        width: Val::Px(680.0),
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)),
                        row_gap: Val::Px(14.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)),
                    BorderRadius::all(Val::Px(16.0)),
                ))
                .with_children(|panel| {
                    trivia_text(panel, TriviaField::Title, "", 28.0, &font);
                    trivia_text(panel, TriviaField::Question, "", 26.0, &font);

                    // Opciones A/B/C/D.
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                TriviaOptionButton(index),
                                Node {
                                    width: Val::Px(600.0),
                                    height: Val::Px(46.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.15, 0.18, 0.28)),
                                BorderColor(Color::srgb(0.50, 0.55, 0.70)),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|option| {
                                trivia_option_text(option, index, 21.0, &font);
                            });
                    }

                    trivia_text(panel, TriviaField::Progress, "", 17.0, &font);
                    trivia_text(panel, TriviaField::Feedback, "", 22.0, &font);

                    // Resultados (ocultos hasta terminar).
                    panel
                        .spawn((
                            TriviaResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            trivia_text(results, TriviaField::ResultTitle, "", 26.0, &font);
                            trivia_text(results, TriviaField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Volver a Ciencias", TriviaBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_trivia(mut commands: Commands, roots: Query<Entity, With<TriviaUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<TriviaSession>();
}

/// Genera las 10 preguntas de la sesión (al azar, sin repetir), en el banco
/// del idioma activo.
fn build_rounds(kind: TriviaKind) -> Vec<TriviaQuestion> {
    let bank: &[TriviaQuestion] = match kind {
        TriviaKind::Science => science_bank(),
        TriviaKind::Geography => geography_bank(),
    };
    let mut rng = rand::thread_rng();
    let mut picked: Vec<TriviaQuestion> = bank.choose_multiple(&mut rng, ROUNDS).copied().collect();
    picked.shuffle(&mut rng);
    picked
}

/// Banco de ciencias naturales según el idioma activo.
fn science_bank() -> &'static [TriviaQuestion] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &SCIENCE_QUESTIONS_EN,
        crate::i18n::Language::Fr => &SCIENCE_QUESTIONS_FR,
        crate::i18n::Language::Es => &SCIENCE_QUESTIONS,
    }
}

/// Banco de geografía de España según el idioma activo.
fn geography_bank() -> &'static [TriviaQuestion] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &GEOGRAPHY_QUESTIONS_EN,
        crate::i18n::Language::Fr => &GEOGRAPHY_QUESTIONS_FR,
        crate::i18n::Language::Es => &GEOGRAPHY_QUESTIONS,
    }
}

/// Colores de los botones de opción.
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

/// Gestiona la sesión de trivia: respuesta, feedback y resultados.
fn update_trivia(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<TriviaSession>>,
    mut texts: Query<
        (&TriviaText, &mut Text, &mut TextColor, &mut Visibility),
        (
            Without<TriviaOptionText>,
            Without<TriviaOptionButton>,
            Without<TriviaResultBox>,
        ),
    >,
    mut option_texts: Query<
        (&TriviaOptionText, &mut Text),
        (
            Without<TriviaText>,
            Without<TriviaOptionButton>,
            Without<TriviaResultBox>,
        ),
    >,
    mut option_colors: Query<(&TriviaOptionButton, &mut BackgroundColor), Without<TriviaText>>,
    option_clicks: Query<
        (&Interaction, &TriviaOptionButton),
        (Changed<Interaction>, Without<TriviaText>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (
            With<TriviaResultBox>,
            Without<TriviaText>,
            Without<TriviaOptionButton>,
        ),
    >,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<TriviaBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(session) => session,
        None => {
            commands.insert_resource(TriviaSession {
                kind: TriviaKind::Science,
                rounds: build_rounds(TriviaKind::Science),
                index: 0,
                correct: 0,
                wrong: 0,
                selected: None,
                feedback: false,
                feedback_timer: 0.0,
                done: false,
            });
            return;
        }
    };

    // Escape: volver al menú de Ciencias.
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::ScienceMenu);
        return;
    }

    // 1) Resultados.
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::KeyQ);
        if close {
            commands.set_state(GameState::ScienceMenu);
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                TriviaField::ResultTitle => {
                    *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" }));
                    *color = TextColor(if session.correct >= ROUNDS / 2 {
                        Color::srgb(0.40, 0.90, 0.50)
                    } else {
                        Color::srgb(0.95, 0.55, 0.30)
                    });
                    *vis = Visibility::Visible;
                }
                TriviaField::ResultDetail => {
                    *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} preguntas").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string()));
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

    // 2) Feedback.
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors {
            let question = &session.rounds[session.index];
            *bg = BackgroundColor(if button.0 == question.correct {
                OPTION_CORRECT
            } else if Some(button.0) == session.selected {
                OPTION_WRONG
            } else {
                OPTION_DIM
            });
        }
        if session.feedback_timer <= 0.0 {
            session.feedback = false;
            session.selected = None;
            session.index += 1;
            if session.index >= ROUNDS {
                session.done = true;
                return;
            }
        }
    } else {
        // 3) Responder: clic en una opción o teclas 1-4.
        let mut chosen: Option<usize> = None;
        for (interaction, button) in &option_clicks {
            if *interaction == Interaction::Pressed {
                chosen = Some(button.0);
                break;
            }
        }
        if chosen.is_none() {
            for (index, code) in [
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
            ]
            .iter()
            .enumerate()
            {
                if keys.just_pressed(*code) {
                    chosen = Some(index);
                    break;
                }
            }
        }
        if let Some(index) = chosen {
            let question = &session.rounds[session.index];
            if index == question.correct {
                session.correct += 1;
                play_success(&mut commands, &sfx);
            } else {
                session.wrong += 1;
            }
            session.selected = Some(index);
            session.feedback = true;
            session.feedback_timer = FEEDBACK_SECONDS;
        }
        for (_button, mut bg) in &mut option_colors {
            *bg = BackgroundColor(OPTION_NEUTRAL);
        }
    }

    // 4) Textos de la pregunta actual.
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            TriviaField::Title => {
                *text = Text::new(tr(session.kind.title()));
                *color = TextColor(Color::srgb(0.85, 0.95, 1.0));
                *vis = Visibility::Visible;
            }
            TriviaField::Question => {
                *text = Text::new(question.question.to_string());
                *vis = Visibility::Visible;
            }
            TriviaField::Progress => {
                *text = Text::new(tr("Pregunta {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()));
                *vis = Visibility::Visible;
            }
            TriviaField::Feedback => {
                if session.feedback {
                    let ok = session.selected == Some(question.correct);
                    if ok {
                        *text = Text::new(tr("¡Correcto!"));
                        *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                    } else {
                        *text = Text::new(tr("Incorrecto — era {}) {}").replace("{}", &OPTION_LETTERS[question.correct].to_string()).replace("{}", &question.options[question.correct]));
                        *color = TextColor(Color::srgb(0.95, 0.40, 0.40));
                    }
                    *vis = Visibility::Visible;
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            _ => {}
        }
    }
    for (field, mut text) in &mut option_texts {
        *text = Text::new(format!(
            "{}) {}",
            OPTION_LETTERS[field.0],
            question.options[field.0]
        ));
    }
    // Clic sonoro al pulsar una opción.
    for (interaction, _button) in &option_clicks {
        if *interaction == Interaction::Pressed {
            play_click(&mut commands, &sfx);
            break;
        }
    }
}

// ---- Banco de preguntas ----------------------------------------------------

/// Preguntas de ciencias naturales.
const SCIENCE_QUESTIONS: [TriviaQuestion; 30] = [
    TriviaQuestion { question: "¿Cuál es el planeta más cercano al Sol?", options: ["Venus", "Mercurio", "Marte", "La Tierra"], correct: 1 },
    TriviaQuestion { question: "¿Cuántas patas tiene un insecto?", options: ["8", "4", "10", "6"], correct: 3 },
    TriviaQuestion { question: "¿Qué órgano bombea la sangre por todo el cuerpo?", options: ["El cerebro", "El pulmón", "El corazón", "El estómago"], correct: 2 },
    TriviaQuestion { question: "¿Cuál es el animal más grande del mundo?", options: ["La ballena azul", "El elefante", "La jirafa", "El tiburón blanco"], correct: 0 },
    TriviaQuestion { question: "¿Qué gas respiramos para vivir?", options: ["El helio", "El oxígeno", "El hidrógeno", "El dióxido de carbono"], correct: 1 },
    TriviaQuestion { question: "¿Cómo se llama la capa de aire que rodea la Tierra?", options: ["El suelo", "La atmósfera", "El núcleo", "El mar"], correct: 1 },
    TriviaQuestion { question: "¿Qué parte de la planta hace la fotosíntesis?", options: ["Las raíces", "Las flores", "El tallo", "Las hojas"], correct: 3 },
    TriviaQuestion { question: "¿Cuál es el hueso más largo del cuerpo humano?", options: ["El fémur", "La tibia", "El húmero", "El cráneo"], correct: 0 },
    TriviaQuestion { question: "¿Qué necesita una planta para hacer la fotosíntesis?", options: ["La sombra", "El viento", "La luz del sol", "El frío"], correct: 2 },
    TriviaQuestion { question: "¿Cuál de estos animales es un mamífero?", options: ["El tiburón", "El pulpo", "El salmón", "El delfín"], correct: 3 },
    TriviaQuestion { question: "¿En qué estado está el agua a 0 grados?", options: ["Líquido", "Vapor", "Sólido (hielo)", "Gaseoso"], correct: 2 },
    TriviaQuestion { question: "¿Qué planeta es conocido como el planeta rojo?", options: ["Júpiter", "Marte", "Venus", "Saturno"], correct: 1 },
    TriviaQuestion { question: "¿Cómo se llama el cambio de la oruga a mariposa?", options: ["Fotosíntesis", "Reproducción", "Respiración", "Metamorfosis"], correct: 3 },
    TriviaQuestion { question: "¿Cuál es el satélite natural de la Tierra?", options: ["El Sol", "La Estación Espacial", "La Luna", "Marte"], correct: 2 },
    TriviaQuestion { question: "¿Qué animal hiberna durante el invierno?", options: ["El delfín", "El gorrión", "El oso", "La gacela"], correct: 2 },
    TriviaQuestion { question: "¿Cuántos continentes tiene la Tierra?", options: ["5", "7", "8", "6"], correct: 3 },
    TriviaQuestion { question: "¿Con qué órgano vemos?", options: ["Los oídos", "La nariz", "La piel", "Los ojos"], correct: 3 },
    TriviaQuestion { question: "¿Cuál de estos animales es un reptil?", options: ["La rana", "La salamandra", "La tortuga", "El tritón"], correct: 2 },
    TriviaQuestion { question: "¿Cuántos planetas hay en el sistema solar?", options: ["7", "8", "9", "10"], correct: 1 },
    TriviaQuestion { question: "¿Qué le ocurre al agua cuando hierve?", options: ["Se convierte en hielo", "Se convierte en vapor", "Desaparece", "Se vuelve azul"], correct: 1 },
    TriviaQuestion { question: "¿Cuál es el animal terrestre más rápido?", options: ["El león", "El caballo", "El guepardo", "El ciervo"], correct: 2 },
    TriviaQuestion { question: "¿Cómo llamamos al planeta azul?", options: ["Marte", "Venus", "Neptuno", "La Tierra"], correct: 3 },
    TriviaQuestion { question: "¿Qué hueso protege el cerebro?", options: ["La pelvis", "El cráneo", "La columna", "Las costillas"], correct: 1 },
    TriviaQuestion { question: "¿Cómo se llama la cría de la vaca?", options: ["El potro", "El cordero", "El ternero", "El cachorro"], correct: 2 },
    TriviaQuestion { question: "¿Qué fruto sale de la vid?", options: ["La pera", "La uva", "La manzana", "La ciruela"], correct: 1 },
    TriviaQuestion { question: "¿Dónde se digieren los alimentos?", options: ["En el cerebro", "En los pulmones", "En el estómago", "En el corazón"], correct: 2 },
    TriviaQuestion { question: "¿Cuál de estos animales pone huevos?", options: ["La vaca", "El delfín", "La gallina", "El perro"], correct: 2 },
    TriviaQuestion { question: "¿Qué necesitan los seres vivos para vivir?", options: ["Solo agua", "Agua, comida y aire", "Solo luz", "Solo calor"], correct: 1 },
    TriviaQuestion { question: "¿Cómo se llama la estrella más cercana a la Tierra?", options: ["La Luna", "Sirio", "El Sol", "Polaris"], correct: 2 },
    TriviaQuestion { question: "¿Qué órgano usamos para respirar?", options: ["El hígado", "Los pulmones", "El riñón", "El estómago"], correct: 1 },
];

/// Preguntas de geografía de España.
const GEOGRAPHY_QUESTIONS: [TriviaQuestion; 32] = [
    TriviaQuestion { question: "¿Cuál es la capital de España?", options: ["Barcelona", "Madrid", "Sevilla", "Valencia"], correct: 1 },
    TriviaQuestion { question: "¿Qué río pasa por Sevilla?", options: ["El Ebro", "El Duero", "El Guadalquivir", "El Tajo"], correct: 2 },
    TriviaQuestion { question: "¿Cuál es el río más largo de España?", options: ["El Ebro", "El Guadalquivir", "El Duero", "El Tajo"], correct: 3 },
    TriviaQuestion { question: "¿Qué mar baña la costa este de España?", options: ["El mar Cantábrico", "El océano Atlántico", "El mar Mediterráneo", "El mar Muerto"], correct: 2 },
    TriviaQuestion { question: "¿Cuál es la montaña más alta de España?", options: ["El Teide", "El Mulhacén", "El Aneto", "El Naranjo"], correct: 0 },
    TriviaQuestion { question: "¿Cuántas comunidades autónomas tiene España?", options: ["16", "17", "18", "19"], correct: 1 },
    TriviaQuestion { question: "¿Qué comunidad autónoma tiene capital en Barcelona?", options: ["Andalucía", "Cataluña", "Aragón", "Valencia"], correct: 1 },
    TriviaQuestion { question: "¿Qué océano baña el oeste de España?", options: ["El océano Pacífico", "El océano Índico", "El océano Ártico", "El océano Atlántico"], correct: 3 },
    TriviaQuestion { question: "¿Cuál es la capital de Andalucía?", options: ["Málaga", "Granada", "Sevilla", "Córdoba"], correct: 2 },
    TriviaQuestion { question: "¿Qué río atraviesa Madrid?", options: ["El Manzanares", "El Tajo", "El Ebro", "El Segura"], correct: 0 },
    TriviaQuestion { question: "¿En qué isla está la ciudad de Palma?", options: ["Tenerife", "Mallorca", "Lanzarote", "Ibiza"], correct: 1 },
    TriviaQuestion { question: "¿Cuál es la capital de la Comunidad Valenciana?", options: ["Alicante", "Castellón", "Valencia", "Elche"], correct: 2 },
    TriviaQuestion { question: "¿Qué cordillera separa España de Francia?", options: ["Los Pirineos", "El Sistema Central", "Sierra Morena", "Los Picos de Europa"], correct: 0 },
    TriviaQuestion { question: "¿Cuál es la capital de Galicia?", options: ["La Coruña", "Vigo", "Orense", "Santiago de Compostela"], correct: 3 },
    TriviaQuestion { question: "¿Qué comunidad autónoma es un archipiélago?", options: ["Extremadura", "Castilla y León", "Canarias", "Aragón"], correct: 2 },
    TriviaQuestion { question: "¿Cuál es la capital de Aragón?", options: ["Huesca", "Teruel", "Zaragoza", "Lérida"], correct: 2 },
    TriviaQuestion { question: "¿Qué estrecho separa España de África?", options: ["El estrecho de Gibraltar", "El estrecho de Mesina", "El estrecho de Bósforo", "El estrecho de Ormuz"], correct: 0 },
    TriviaQuestion { question: "¿Cuál es la capital de Castilla y León?", options: ["León", "Valladolid", "Burgos", "Salamanca"], correct: 1 },
    TriviaQuestion { question: "¿Qué comunidad autónoma está junto al mar Cantábrico?", options: ["Extremadura", "Andalucía", "País Vasco", "Murcia"], correct: 2 },
    TriviaQuestion { question: "¿Cuál es la capital de Extremadura?", options: ["Badajoz", "Cáceres", "Mérida", "Plasencia"], correct: 2 },
    TriviaQuestion { question: "¿Qué río pasa por Zaragoza?", options: ["El Duero", "El Ebro", "El Guadalquivir", "El Tajo"], correct: 1 },
    TriviaQuestion { question: "¿Cuál es la capital de la Región de Murcia?", options: ["Cartagena", "Lorca", "Murcia", "Molina"], correct: 2 },
    TriviaQuestion { question: "¿En qué comunidad autónoma está Sevilla?", options: ["Castilla-La Mancha", "Extremadura", "Andalucía", "Murcia"], correct: 2 },
    TriviaQuestion { question: "¿Cuál es el pico más alto de la península ibérica?", options: ["El Aneto", "El Teide", "El Mulhacén", "El Moncayo"], correct: 2 },
    TriviaQuestion { question: "¿En qué isla está Santa Cruz de Tenerife?", options: ["Gran Canaria", "Tenerife", "La Palma", "Fuerteventura"], correct: 1 },
    TriviaQuestion { question: "¿Cuál es la capital de Cantabria?", options: ["Santander", "Bilbao", "Gijón", "Oviedo"], correct: 0 },
    TriviaQuestion { question: "¿Cuál es la capital de Castilla-La Mancha?", options: ["Cuenca", "Albacete", "Ciudad Real", "Toledo"], correct: 3 },
    TriviaQuestion { question: "¿Cuál es la capital de Asturias?", options: ["Gijón", "Avilés", "Oviedo", "Mieres"], correct: 2 },
    TriviaQuestion { question: "¿Qué comunidad autónoma tiene capital en Logroño?", options: ["Navarra", "La Rioja", "Cantabria", "Aragón"], correct: 1 },
    TriviaQuestion { question: "¿Cuál es la capital de Navarra?", options: ["San Sebastián", "Vitoria", "Logroño", "Pamplona"], correct: 3 },
    TriviaQuestion { question: "¿Qué islas están en el mar Mediterráneo?", options: ["Las Canarias", "Las Baleares", "Las Azores", "Las Cíes"], correct: 1 },
    TriviaQuestion { question: "¿Cuál es la capital de Euskadi?", options: ["Bilbao", "San Sebastián", "Vitoria-Gasteiz", "Logroño"], correct: 2 },
];

/// Preguntas de ciencias naturales en inglés.
const SCIENCE_QUESTIONS_EN: [TriviaQuestion; 30] = [
    TriviaQuestion { question: "Which planet is closest to the Sun?", options: ["Venus", "Mercury", "Mars", "Earth"], correct: 1 },
    TriviaQuestion { question: "How many legs does an insect have?", options: ["8", "4", "10", "6"], correct: 3 },
    TriviaQuestion { question: "Which organ pumps blood around the body?", options: ["The brain", "The lung", "The heart", "The stomach"], correct: 2 },
    TriviaQuestion { question: "What is the biggest animal in the world?", options: ["The blue whale", "The elephant", "The giraffe", "The great white shark"], correct: 0 },
    TriviaQuestion { question: "Which gas do we breathe to live?", options: ["Helium", "Oxygen", "Hydrogen", "Carbon dioxide"], correct: 1 },
    TriviaQuestion { question: "What is the layer of air around the Earth called?", options: ["The soil", "The atmosphere", "The core", "The sea"], correct: 1 },
    TriviaQuestion { question: "Which part of the plant does photosynthesis?", options: ["The roots", "The flowers", "The stem", "The leaves"], correct: 3 },
    TriviaQuestion { question: "What is the longest bone in the human body?", options: ["The femur", "The tibia", "The humerus", "The skull"], correct: 0 },
    TriviaQuestion { question: "What does a plant need for photosynthesis?", options: ["Shade", "Wind", "Sunlight", "Cold"], correct: 2 },
    TriviaQuestion { question: "Which of these animals is a mammal?", options: ["The shark", "The octopus", "The salmon", "The dolphin"], correct: 3 },
    TriviaQuestion { question: "What state is water in at 0 degrees?", options: ["Liquid", "Vapour", "Solid (ice)", "Gas"], correct: 2 },
    TriviaQuestion { question: "Which planet is known as the red planet?", options: ["Jupiter", "Mars", "Venus", "Saturn"], correct: 1 },
    TriviaQuestion { question: "What is the change from caterpillar to butterfly called?", options: ["Photosynthesis", "Reproduction", "Respiration", "Metamorphosis"], correct: 3 },
    TriviaQuestion { question: "What is the natural satellite of the Earth?", options: ["The Sun", "The Space Station", "The Moon", "Mars"], correct: 2 },
    TriviaQuestion { question: "Which animal hibernates in winter?", options: ["The dolphin", "The sparrow", "The bear", "The gazelle"], correct: 2 },
    TriviaQuestion { question: "How many continents does the Earth have?", options: ["5", "7", "8", "6"], correct: 3 },
    TriviaQuestion { question: "Which organ do we see with?", options: ["The ears", "The nose", "The skin", "The eyes"], correct: 3 },
    TriviaQuestion { question: "Which of these animals is a reptile?", options: ["The frog", "The salamander", "The turtle", "The newt"], correct: 2 },
    TriviaQuestion { question: "How many planets are there in the solar system?", options: ["7", "8", "9", "10"], correct: 1 },
    TriviaQuestion { question: "What happens to water when it boils?", options: ["It turns into ice", "It turns into steam", "It disappears", "It turns blue"], correct: 1 },
    TriviaQuestion { question: "What is the fastest land animal?", options: ["The lion", "The horse", "The cheetah", "The deer"], correct: 2 },
    TriviaQuestion { question: "What do we call the blue planet?", options: ["Mars", "Venus", "Neptune", "Earth"], correct: 3 },
    TriviaQuestion { question: "Which bone protects the brain?", options: ["The pelvis", "The skull", "The spine", "The ribs"], correct: 1 },
    TriviaQuestion { question: "What is the baby of a cow called?", options: ["The foal", "The lamb", "The calf", "The puppy"], correct: 2 },
    TriviaQuestion { question: "Which fruit comes from the vine?", options: ["The pear", "The grape", "The apple", "The plum"], correct: 1 },
    TriviaQuestion { question: "Where is food digested?", options: ["In the brain", "In the lungs", "In the stomach", "In the heart"], correct: 2 },
    TriviaQuestion { question: "Which of these animals lays eggs?", options: ["The cow", "The dolphin", "The hen", "The dog"], correct: 2 },
    TriviaQuestion { question: "What do living things need to survive?", options: ["Only water", "Water, food and air", "Only light", "Only heat"], correct: 1 },
    TriviaQuestion { question: "What is the closest star to the Earth?", options: ["The Moon", "Sirius", "The Sun", "Polaris"], correct: 2 },
    TriviaQuestion { question: "Which organ do we use to breathe?", options: ["The liver", "The lungs", "The kidney", "The stomach"], correct: 1 },
];

/// Preguntas de geografía de España en inglés.
const GEOGRAPHY_QUESTIONS_EN: [TriviaQuestion; 32] = [
    TriviaQuestion { question: "What is the capital of Spain?", options: ["Barcelona", "Madrid", "Seville", "Valencia"], correct: 1 },
    TriviaQuestion { question: "Which river flows through Seville?", options: ["The Ebro", "The Duero", "The Guadalquivir", "The Tagus"], correct: 2 },
    TriviaQuestion { question: "What is the longest river in Spain?", options: ["The Ebro", "The Guadalquivir", "The Duero", "The Tagus"], correct: 3 },
    TriviaQuestion { question: "Which sea bathes the east coast of Spain?", options: ["The Cantabrian Sea", "The Atlantic Ocean", "The Mediterranean Sea", "The Dead Sea"], correct: 2 },
    TriviaQuestion { question: "What is the highest mountain in Spain?", options: ["Teide", "Mulhacén", "Aneto", "Naranjo"], correct: 0 },
    TriviaQuestion { question: "How many autonomous communities does Spain have?", options: ["16", "17", "18", "19"], correct: 1 },
    TriviaQuestion { question: "Which autonomous community has its capital in Barcelona?", options: ["Andalusia", "Catalonia", "Aragon", "Valencia"], correct: 1 },
    TriviaQuestion { question: "Which ocean bathes the west of Spain?", options: ["The Pacific Ocean", "The Indian Ocean", "The Arctic Ocean", "The Atlantic Ocean"], correct: 3 },
    TriviaQuestion { question: "What is the capital of Andalusia?", options: ["Malaga", "Granada", "Seville", "Cordoba"], correct: 2 },
    TriviaQuestion { question: "Which river runs through Madrid?", options: ["The Manzanares", "The Tagus", "The Ebro", "The Segura"], correct: 0 },
    TriviaQuestion { question: "On which island is the city of Palma?", options: ["Tenerife", "Mallorca", "Lanzarote", "Ibiza"], correct: 1 },
    TriviaQuestion { question: "What is the capital of the Valencian Community?", options: ["Alicante", "Castellón", "Valencia", "Elche"], correct: 2 },
    TriviaQuestion { question: "Which mountain range separates Spain from France?", options: ["The Pyrenees", "The Central System", "Sierra Morena", "The Picos de Europa"], correct: 0 },
    TriviaQuestion { question: "What is the capital of Galicia?", options: ["La Coruña", "Vigo", "Orense", "Santiago de Compostela"], correct: 3 },
    TriviaQuestion { question: "Which autonomous community is an archipelago?", options: ["Extremadura", "Castile and León", "The Canary Islands", "Aragon"], correct: 2 },
    TriviaQuestion { question: "What is the capital of Aragon?", options: ["Huesca", "Teruel", "Zaragoza", "Lérida"], correct: 2 },
    TriviaQuestion { question: "Which strait separates Spain from Africa?", options: ["The Strait of Gibraltar", "The Strait of Messina", "The Bosphorus", "The Strait of Hormuz"], correct: 0 },
    TriviaQuestion { question: "What is the capital of Castile and León?", options: ["León", "Valladolid", "Burgos", "Salamanca"], correct: 1 },
    TriviaQuestion { question: "Which autonomous community is next to the Cantabrian Sea?", options: ["Extremadura", "Andalusia", "The Basque Country", "Murcia"], correct: 2 },
    TriviaQuestion { question: "What is the capital of Extremadura?", options: ["Badajoz", "Cáceres", "Mérida", "Plasencia"], correct: 2 },
    TriviaQuestion { question: "Which river flows through Zaragoza?", options: ["The Duero", "The Ebro", "The Guadalquivir", "The Tagus"], correct: 1 },
    TriviaQuestion { question: "What is the capital of the Region of Murcia?", options: ["Cartagena", "Lorca", "Murcia", "Molina"], correct: 2 },
    TriviaQuestion { question: "In which autonomous community is Seville?", options: ["Castile-La Mancha", "Extremadura", "Andalusia", "Murcia"], correct: 2 },
    TriviaQuestion { question: "What is the highest peak on the Iberian peninsula?", options: ["Aneto", "Teide", "Mulhacén", "Moncayo"], correct: 2 },
    TriviaQuestion { question: "On which island is Santa Cruz de Tenerife?", options: ["Gran Canaria", "Tenerife", "La Palma", "Fuerteventura"], correct: 1 },
    TriviaQuestion { question: "What is the capital of Cantabria?", options: ["Santander", "Bilbao", "Gijón", "Oviedo"], correct: 0 },
    TriviaQuestion { question: "What is the capital of Castile-La Mancha?", options: ["Cuenca", "Albacete", "Ciudad Real", "Toledo"], correct: 3 },
    TriviaQuestion { question: "What is the capital of Asturias?", options: ["Gijón", "Avilés", "Oviedo", "Mieres"], correct: 2 },
    TriviaQuestion { question: "Which autonomous community has its capital in Logroño?", options: ["Navarre", "La Rioja", "Cantabria", "Aragon"], correct: 1 },
    TriviaQuestion { question: "What is the capital of Navarre?", options: ["San Sebastián", "Vitoria", "Logroño", "Pamplona"], correct: 3 },
    TriviaQuestion { question: "Which islands are in the Mediterranean Sea?", options: ["The Canary Islands", "The Balearic Islands", "The Azores", "The Cíes Islands"], correct: 1 },
    TriviaQuestion { question: "What is the capital of the Basque Country?", options: ["Bilbao", "San Sebastián", "Vitoria-Gasteiz", "Logroño"], correct: 2 },
];

/// Preguntas de ciencias naturales en francés.
const SCIENCE_QUESTIONS_FR: [TriviaQuestion; 30] = [
    TriviaQuestion { question: "Quelle planète est la plus proche du Soleil ?", options: ["Vénus", "Mercure", "Mars", "La Terre"], correct: 1 },
    TriviaQuestion { question: "Combien de pattes a un insecte ?", options: ["8", "4", "10", "6"], correct: 3 },
    TriviaQuestion { question: "Quel organe pompe le sang dans tout le corps ?", options: ["Le cerveau", "Le poumon", "Le cœur", "L'estomac"], correct: 2 },
    TriviaQuestion { question: "Quel est le plus grand animal du monde ?", options: ["La baleine bleue", "L'éléphant", "La girafe", "Le grand requin blanc"], correct: 0 },
    TriviaQuestion { question: "Quel gaz respirons-nous pour vivre ?", options: ["L'hélium", "L'oxygène", "L'hydrogène", "Le dioxyde de carbone"], correct: 1 },
    TriviaQuestion { question: "Comment s'appelle la couche d'air autour de la Terre ?", options: ["Le sol", "L'atmosphère", "Le noyau", "La mer"], correct: 1 },
    TriviaQuestion { question: "Quelle partie de la plante fait la photosynthèse ?", options: ["Les racines", "Les fleurs", "La tige", "Les feuilles"], correct: 3 },
    TriviaQuestion { question: "Quel est le plus long os du corps humain ?", options: ["Le fémur", "Le tibia", "L'humérus", "Le crâne"], correct: 0 },
    TriviaQuestion { question: "De quoi une plante a-t-elle besoin pour la photosynthèse ?", options: ["De l'ombre", "Du vent", "De la lumière du soleil", "Du froid"], correct: 2 },
    TriviaQuestion { question: "Lequel de ces animaux est un mammifère ?", options: ["Le requin", "Le poulpe", "Le saumon", "Le dauphin"], correct: 3 },
    TriviaQuestion { question: "Dans quel état est l'eau à 0 degré ?", options: ["Liquide", "Vapeur", "Solide (glace)", "Gazeux"], correct: 2 },
    TriviaQuestion { question: "Quelle planète est connue comme la planète rouge ?", options: ["Jupiter", "Mars", "Vénus", "Saturne"], correct: 1 },
    TriviaQuestion { question: "Comment s'appelle le changement de la chenille en papillon ?", options: ["Photosynthèse", "Reproduction", "Respiration", "Métamorphose"], correct: 3 },
    TriviaQuestion { question: "Quel est le satellite naturel de la Terre ?", options: ["Le Soleil", "La Station spatiale", "La Lune", "Mars"], correct: 2 },
    TriviaQuestion { question: "Quel animal hiberne pendant l'hiver ?", options: ["Le dauphin", "Le moineau", "L'ours", "La gazelle"], correct: 2 },
    TriviaQuestion { question: "Combien de continents compte la Terre ?", options: ["5", "7", "8", "6"], correct: 3 },
    TriviaQuestion { question: "Avec quel organe voyons-nous ?", options: ["Les oreilles", "Le nez", "La peau", "Les yeux"], correct: 3 },
    TriviaQuestion { question: "Lequel de ces animaux est un reptile ?", options: ["La grenouille", "La salamandre", "La tortue", "Le triton"], correct: 2 },
    TriviaQuestion { question: "Combien y a-t-il de planètes dans le système solaire ?", options: ["7", "8", "9", "10"], correct: 1 },
    TriviaQuestion { question: "Que devient l'eau quand elle bout ?", options: ["Elle se transforme en glace", "Elle se transforme en vapeur", "Elle disparaît", "Elle devient bleue"], correct: 1 },
    TriviaQuestion { question: "Quel est l'animal terrestre le plus rapide ?", options: ["Le lion", "Le cheval", "Le guépard", "Le cerf"], correct: 2 },
    TriviaQuestion { question: "Comment appelle-t-on la planète bleue ?", options: ["Mars", "Vénus", "Neptune", "La Terre"], correct: 3 },
    TriviaQuestion { question: "Quel os protège le cerveau ?", options: ["Le bassin", "Le crâne", "La colonne", "Les côtes"], correct: 1 },
    TriviaQuestion { question: "Comment s'appelle le petit de la vache ?", options: ["Le poulain", "L'agneau", "Le veau", "Le chiot"], correct: 2 },
    TriviaQuestion { question: "Quel fruit vient de la vigne ?", options: ["La poire", "Le raisin", "La pomme", "La prune"], correct: 1 },
    TriviaQuestion { question: "Où les aliments sont-ils digérés ?", options: ["Dans le cerveau", "Dans les poumons", "Dans l'estomac", "Dans le cœur"], correct: 2 },
    TriviaQuestion { question: "Lequel de ces animaux pond des œufs ?", options: ["La vache", "Le dauphin", "La poule", "Le chien"], correct: 2 },
    TriviaQuestion { question: "De quoi les êtres vivants ont-ils besoin pour vivre ?", options: ["Seulement d'eau", "D'eau, de nourriture et d'air", "Seulement de lumière", "Seulement de chaleur"], correct: 1 },
    TriviaQuestion { question: "Comment s'appelle l'étoile la plus proche de la Terre ?", options: ["La Lune", "Sirius", "Le Soleil", "Polaris"], correct: 2 },
    TriviaQuestion { question: "Quel organe utilisons-nous pour respirer ?", options: ["Le foie", "Les poumons", "Le rein", "L'estomac"], correct: 1 },
];

/// Preguntas de geografía de España en francés.
const GEOGRAPHY_QUESTIONS_FR: [TriviaQuestion; 32] = [
    TriviaQuestion { question: "Quelle est la capitale de l'Espagne ?", options: ["Barcelone", "Madrid", "Séville", "Valence"], correct: 1 },
    TriviaQuestion { question: "Quel fleuve traverse Séville ?", options: ["L'Èbre", "Le Duero", "Le Guadalquivir", "Le Tage"], correct: 2 },
    TriviaQuestion { question: "Quel est le plus long fleuve d'Espagne ?", options: ["L'Èbre", "Le Guadalquivir", "Le Duero", "Le Tage"], correct: 3 },
    TriviaQuestion { question: "Quelle mer baigne la côte est de l'Espagne ?", options: ["La mer Cantabrique", "L'océan Atlantique", "La mer Méditerranée", "La mer Morte"], correct: 2 },
    TriviaQuestion { question: "Quelle est la plus haute montagne d'Espagne ?", options: ["Le Teide", "Le Mulhacén", "L'Aneto", "Le Naranjo"], correct: 0 },
    TriviaQuestion { question: "Combien de communautés autonomes compte l'Espagne ?", options: ["16", "17", "18", "19"], correct: 1 },
    TriviaQuestion { question: "Quelle communauté autonome a sa capitale à Barcelone ?", options: ["L'Andalousie", "La Catalogne", "L'Aragon", "Valence"], correct: 1 },
    TriviaQuestion { question: "Quel océan baigne l'ouest de l'Espagne ?", options: ["L'océan Pacifique", "L'océan Indien", "L'océan Arctique", "L'océan Atlantique"], correct: 3 },
    TriviaQuestion { question: "Quelle est la capitale de l'Andalousie ?", options: ["Malaga", "Grenade", "Séville", "Cordoue"], correct: 2 },
    TriviaQuestion { question: "Quel fleuve traverse Madrid ?", options: ["Le Manzanares", "Le Tage", "L'Èbre", "Le Segura"], correct: 0 },
    TriviaQuestion { question: "Sur quelle île se trouve la ville de Palma ?", options: ["Tenerife", "Majorque", "Lanzarote", "Ibiza"], correct: 1 },
    TriviaQuestion { question: "Quelle est la capitale de la Communauté valencienne ?", options: ["Alicante", "Castellón", "Valence", "Elche"], correct: 2 },
    TriviaQuestion { question: "Quelle chaîne de montagnes sépare l'Espagne de la France ?", options: ["Les Pyrénées", "Le Système central", "Sierra Morena", "Les Picos de Europa"], correct: 0 },
    TriviaQuestion { question: "Quelle est la capitale de la Galice ?", options: ["La Corogne", "Vigo", "Orense", "Saint-Jacques-de-Compostelle"], correct: 3 },
    TriviaQuestion { question: "Quelle communauté autonome est un archipel ?", options: ["L'Estrémadure", "Castille-et-León", "Les Canaries", "L'Aragon"], correct: 2 },
    TriviaQuestion { question: "Quelle est la capitale de l'Aragon ?", options: ["Huesca", "Teruel", "Saragosse", "Lérida"], correct: 2 },
    TriviaQuestion { question: "Quel détroit sépare l'Espagne de l'Afrique ?", options: ["Le détroit de Gibraltar", "Le détroit de Messine", "Le Bosphore", "Le détroit d'Ormuz"], correct: 0 },
    TriviaQuestion { question: "Quelle est la capitale de Castille-et-León ?", options: ["León", "Valladolid", "Burgos", "Salamanque"], correct: 1 },
    TriviaQuestion { question: "Quelle communauté autonome est au bord de la mer Cantabrique ?", options: ["L'Estrémadure", "L'Andalousie", "Le Pays basque", "Murcie"], correct: 2 },
    TriviaQuestion { question: "Quelle est la capitale de l'Estrémadure ?", options: ["Badajoz", "Cáceres", "Mérida", "Plasencia"], correct: 2 },
    TriviaQuestion { question: "Quel fleuve traverse Saragosse ?", options: ["Le Duero", "L'Èbre", "Le Guadalquivir", "Le Tage"], correct: 1 },
    TriviaQuestion { question: "Quelle est la capitale de la Région de Murcie ?", options: ["Cartagène", "Lorca", "Murcie", "Molina"], correct: 2 },
    TriviaQuestion { question: "Dans quelle communauté autonome se trouve Séville ?", options: ["Castille-La Manche", "L'Estrémadure", "L'Andalousie", "Murcie"], correct: 2 },
    TriviaQuestion { question: "Quel est le plus haut sommet de la péninsule ibérique ?", options: ["L'Aneto", "Le Teide", "Le Mulhacén", "Le Moncayo"], correct: 2 },
    TriviaQuestion { question: "Sur quelle île se trouve Santa Cruz de Tenerife ?", options: ["Grande Canarie", "Tenerife", "La Palma", "Fuerteventura"], correct: 1 },
    TriviaQuestion { question: "Quelle est la capitale de la Cantabrie ?", options: ["Santander", "Bilbao", "Gijón", "Oviedo"], correct: 0 },
    TriviaQuestion { question: "Quelle est la capitale de Castille-La Manche ?", options: ["Cuenca", "Albacete", "Ciudad Real", "Tolède"], correct: 3 },
    TriviaQuestion { question: "Quelle est la capitale des Asturies ?", options: ["Gijón", "Avilés", "Oviedo", "Mieres"], correct: 2 },
    TriviaQuestion { question: "Quelle communauté autonome a sa capitale à Logroño ?", options: ["La Navarre", "La Rioja", "La Cantabrie", "L'Aragon"], correct: 1 },
    TriviaQuestion { question: "Quelle est la capitale de la Navarre ?", options: ["Saint-Sébastien", "Vitoria", "Logroño", "Pampelune"], correct: 3 },
    TriviaQuestion { question: "Quelles îles se trouvent en mer Méditerranée ?", options: ["Les Canaries", "Les Baléares", "Les Açores", "Les îles Cíes"], correct: 1 },
    TriviaQuestion { question: "Quelle est la capitale du Pays basque ?", options: ["Bilbao", "Saint-Sébastien", "Vitoria-Gasteiz", "Logroño"], correct: 2 },
];