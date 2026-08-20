//! Práctica de ortografía (sección Lengua).
//!
//! Se muestra una frase con una palabra en blanco y hay que elegir la forma
//! correcta entre 4 opciones (errores típicos: b/v, c/s/z, h, y/ll, tildes…).
//! Sesión de 10 frases con feedback y marcador final.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

/// Número de frases por sesión.
const ROUNDS: usize = 10;
/// Duración (s) del feedback.
const FEEDBACK_SECONDS: f32 = 1.4;

/// Una frase con hueco, sus 4 opciones y la correcta.
#[derive(Clone, Copy)]
struct SpellingQuestion {
    sentence: &'static str,
    options: [&'static str; 4],
    correct: usize,
}

/// Sesión de ortografía activa.
#[derive(Resource)]
pub struct SpellingSession {
    rounds: Vec<SpellingQuestion>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

// ---- Componentes de la UI --------------------------------------------------

/// Raíz de la pantalla de ortografía.
#[derive(Component)]
pub struct SpellingUiRoot;

/// Campo de texto etiquetado por su función.
#[derive(Component)]
pub struct SpellingText(SpellingField);

#[derive(Clone, Copy, PartialEq, Eq)]
enum SpellingField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}

/// Texto de una opción (hijo de su botón).
#[derive(Component)]
pub struct SpellingOptionText(pub usize);

/// Botón de una opción (A/B/C/D).
#[derive(Component)]
pub struct SpellingOptionButton(pub usize);

/// Contenedor de resultados (oculto hasta terminar).
#[derive(Component)]
pub struct SpellingResultBox;

/// Botón de volver al menú de Lengua.
#[derive(Component)]
pub struct SpellingBackButton;

/// Plugin de la práctica de ortografía.
pub struct SpellingPlugin;

impl Plugin for SpellingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SpellingPractice), spawn_spelling_ui)
            .add_systems(OnExit(GameState::SpellingPractice), cleanup_spelling)
            .add_systems(
                Update,
                update_spelling.run_if(in_state(GameState::SpellingPractice)),
            );
    }
}

/// Letras de las opciones.
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

/// Crea un texto del campo indicado.
fn spelling_text(
    parent: &mut ChildSpawnerCommands,
    field: SpellingField,
    text: &str,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        SpellingText(field),
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
fn spelling_option_text(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    size: f32,
    font: &Handle<Font>,
) {
    parent.spawn((
        SpellingOptionText(index),
        Text::new(String::new()),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Construye la pantalla de ortografía.
fn spawn_spelling_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(SpellingSession {
        rounds: build_rounds(),
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
            SpellingUiRoot,
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
                    spelling_text(panel, SpellingField::Title, "", 28.0, &font);
                    spelling_text(panel, SpellingField::Question, "", 30.0, &font);

                    // Opciones A/B/C/D.
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                SpellingOptionButton(index),
                                Node {
                                    width: Val::Px(560.0),
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
                                spelling_option_text(option, index, 22.0, &font);
                            });
                    }

                    spelling_text(panel, SpellingField::Progress, "", 17.0, &font);
                    spelling_text(panel, SpellingField::Feedback, "", 22.0, &font);

                    // Resultados (ocultos hasta terminar).
                    panel
                        .spawn((
                            SpellingResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            spelling_text(results, SpellingField::ResultTitle, "", 26.0, &font);
                            spelling_text(results, SpellingField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Volver a Lengua", SpellingBackButton, &font);
                        });
                });
        });
}

/// Destruye la pantalla y la sesión al salir.
fn cleanup_spelling(mut commands: Commands, roots: Query<Entity, With<SpellingUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<SpellingSession>();
}

/// Genera las 10 frases de la sesión (al azar, sin repetir), en el banco del
/// idioma activo.
fn build_rounds() -> Vec<SpellingQuestion> {
    let mut rng = rand::thread_rng();
    bank()
        .choose_multiple(&mut rng, ROUNDS)
        .copied()
        .collect()
}

/// Banco de frases según el idioma activo.
fn bank() -> &'static [SpellingQuestion] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &SPELLING_QUESTIONS_EN,
        crate::i18n::Language::Fr => &SPELLING_QUESTIONS_FR,
        crate::i18n::Language::Es => &SPELLING_QUESTIONS,
    }
}

/// Colores de los botones de opción.
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

/// Gestiona la sesión de ortografía: respuesta, feedback y resultados.
fn update_spelling(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<SpellingSession>>,
    mut texts: Query<
        (&SpellingText, &mut Text, &mut TextColor, &mut Visibility),
        (
            Without<SpellingOptionText>,
            Without<SpellingOptionButton>,
            Without<SpellingResultBox>,
        ),
    >,
    mut option_texts: Query<
        (&SpellingOptionText, &mut Text),
        (
            Without<SpellingText>,
            Without<SpellingOptionButton>,
            Without<SpellingResultBox>,
        ),
    >,
    mut option_colors: Query<(&SpellingOptionButton, &mut BackgroundColor), Without<SpellingText>>,
    option_clicks: Query<
        (&Interaction, &SpellingOptionButton),
        (Changed<Interaction>, Without<SpellingText>),
    >,
    mut result_box: Query<
        &mut Visibility,
        (
            With<SpellingResultBox>,
            Without<SpellingText>,
            Without<SpellingOptionButton>,
        ),
    >,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<SpellingBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(session) => session,
        None => {
            commands.insert_resource(SpellingSession {
                rounds: build_rounds(),
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

    // Escape: volver al menú de Lengua.
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::LanguageMenu);
        return;
    }

    // 1) Resultados.
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::KeyQ);
        if close {
            commands.set_state(GameState::LanguageMenu);
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                SpellingField::ResultTitle => {
                    *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" }));
                    *color = TextColor(if session.correct >= ROUNDS / 2 {
                        Color::srgb(0.40, 0.90, 0.50)
                    } else {
                        Color::srgb(0.95, 0.55, 0.30)
                    });
                    *vis = Visibility::Visible;
                }
                SpellingField::ResultDetail => {
                    *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} frases").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string()));
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

    // 4) Textos de la frase actual.
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            SpellingField::Title => {
                *text = Text::new("ORTOGRAFÍA");
                *color = TextColor(Color::srgb(0.85, 0.95, 1.0));
                *vis = Visibility::Visible;
            }
            SpellingField::Question => {
                *text = Text::new(question.sentence.to_string());
                *vis = Visibility::Visible;
            }
            SpellingField::Progress => {
                *text = Text::new(tr("Frase {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()));
                *vis = Visibility::Visible;
            }
            SpellingField::Feedback => {
                if session.feedback {
                    let ok = session.selected == Some(question.correct);
                    if ok {
                        *text = Text::new(tr("¡Correcto!"));
                        *color = TextColor(Color::srgb(0.40, 0.90, 0.50));
                    } else {
                        *text = Text::new(tr("Incorrecto — la correcta es: {}").replace("{}", &question.options[question.correct]));
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

// ---- Banco de frases -------------------------------------------------------

/// Frases con errores ortográficos típicos del español.
const SPELLING_QUESTIONS: [SpellingQuestion; 30] = [
    SpellingQuestion { sentence: "La ______ hace miel.", options: ["habeja", "abeja", "aveja", "abeha"], correct: 1 },
    SpellingQuestion { sentence: "El perro ______ todo el día.", options: ["ladraba", "labraba", "ladrava", "labrava"], correct: 0 },
    SpellingQuestion { sentence: "Me gusta ______ en el parque.", options: ["hugar", "juhar", "jugar", "huguar"], correct: 2 },
    SpellingQuestion { sentence: "La ______ tiene manzanas.", options: ["uerta", "güerta", "huherta", "huerta"], correct: 3 },
    SpellingQuestion { sentence: "Hoy hace mucho ______.", options: ["calór", "kalor", "calor", "calorr"], correct: 2 },
    SpellingQuestion { sentence: "El ______ de la ventana está roto.", options: ["bidrio", "vidrrio", "vidrio", "bidrrio"], correct: 2 },
    SpellingQuestion { sentence: "En el ______ hay muchas estrellas.", options: ["zielo", "cielo", "scielo", "ceilo"], correct: 1 },
    SpellingQuestion { sentence: "El ______ del árbol es muy grueso.", options: ["tronko", "troncco", "tronco", "trronco"], correct: 2 },
    SpellingQuestion { sentence: "Mi abuela me ______ cuentos.", options: ["cunta", "kuenta", "cuentta", "cuenta"], correct: 3 },
    SpellingQuestion { sentence: "El ______ vuela muy alto.", options: ["habión", "abión", "avion", "avión"], correct: 3 },
    SpellingQuestion { sentence: "Compré ______ en la frutería.", options: ["ubas", "uvas", "huvas", "ubvas"], correct: 1 },
    SpellingQuestion { sentence: "El gato ______ sobre el sofá.", options: ["duerhme", "duerme", "duerne", "duereme"], correct: 1 },
    SpellingQuestion { sentence: "La ______ es mi asignatura favorita.", options: ["musica", "música", "musika", "musiga"], correct: 1 },
    SpellingQuestion { sentence: "¿Qué hora ______?", options: ["es", "ez", "hes", "ehs"], correct: 0 },
    SpellingQuestion { sentence: "______ ayer al cine con mi primo.", options: ["Fuí", "Hufi", "Fui", "Fuy"], correct: 2 },
    SpellingQuestion { sentence: "El ______ del pueblo es muy antiguo.", options: ["castiyo", "kastillo", "castillo", "castilo"], correct: 2 },
    SpellingQuestion { sentence: "Me duele la ______.", options: ["caveza", "cabesa", "cabeça", "cabeza"], correct: 3 },
    SpellingQuestion { sentence: "El ______ está muy limpio.", options: ["ospital", "hospitar", "hospitall", "hospital"], correct: 3 },
    SpellingQuestion { sentence: "Las ______ del jardín son rojas.", options: ["rozas", "rosas", "rosass", "rrosas"], correct: 1 },
    SpellingQuestion { sentence: "______ el lápiz, por favor.", options: ["Coge", "Coje", "Cóge", "Koge"], correct: 0 },
    SpellingQuestion { sentence: "En invierno cae la ______.", options: ["niebe", "nieve", "nyebe", "nievve"], correct: 1 },
    SpellingQuestion { sentence: "El ______ tiene muchas páginas.", options: ["livro", "libbro", "liblo", "libro"], correct: 3 },
    SpellingQuestion { sentence: "El ______ del partido fue emocionante.", options: ["finnal", "final", "finl", "finall"], correct: 1 },
    SpellingQuestion { sentence: "¿______ años tienes?", options: ["¿Cuantos", "¿Quantos", "¿Cuántos", "¿Kuántos"], correct: 2 },
    SpellingQuestion { sentence: "En la ______ hay muchos libros.", options: ["bibloteca", "viblioteca", "biblioteca", "biblioeca"], correct: 2 },
    SpellingQuestion { sentence: "Mi amigo ______ muy bien la guitarra.", options: ["toka", "tocga", "ttoha", "toca"], correct: 3 },
    SpellingQuestion { sentence: "El plátano es ______.", options: ["amarilo", "amariyo", "hamarillo", "amarillo"], correct: 3 },
    SpellingQuestion { sentence: "¿______ está el baño?", options: ["Donde", "Dondé", "Dónde", "Dóndé"], correct: 2 },
    SpellingQuestion { sentence: "El ______ del barco es muy alto.", options: ["mastil", "mástil", "masstil", "mástill"], correct: 1 },
    SpellingQuestion { sentence: "El ______ es un animal muy rápido.", options: ["conejo", "coneho", "honejo", "conego"], correct: 0 },
];

/// Frases de ortografía típicas del inglés.
const SPELLING_QUESTIONS_EN: [SpellingQuestion; 30] = [
    SpellingQuestion { sentence: "The ______ makes honey.", options: ["bea", "bee", "be", "beh"], correct: 1 },
    SpellingQuestion { sentence: "I ______ my homework every day.", options: ["do", "du", "dho", "doo"], correct: 0 },
    SpellingQuestion { sentence: "She has a ______ new bike.", options: ["blue", "blu", "blew", "bloo"], correct: 0 },
    SpellingQuestion { sentence: "We ______ to the park yesterday.", options: ["went", "whnt", "weent", "whent"], correct: 0 },
    SpellingQuestion { sentence: "The cat is ______ the table.", options: ["under", "undr", "underr", "undar"], correct: 0 },
    SpellingQuestion { sentence: "I like to ______ books.", options: ["red", "read", "reed", "rade"], correct: 1 },
    SpellingQuestion { sentence: "My ______ is very big.", options: ["house", "hous", "howse", "hause"], correct: 0 },
    SpellingQuestion { sentence: "The ______ is shining today.", options: ["sun", "sunn", "sone", "sonn"], correct: 0 },
    SpellingQuestion { sentence: "Please ______ the door.", options: ["close", "cloze", "clos", "closse"], correct: 0 },
    SpellingQuestion { sentence: "I can ______ very fast.", options: ["run", "runn", "rune", "runne"], correct: 0 },
    SpellingQuestion { sentence: "There are seven days in a ______.", options: ["week", "weak", "wek", "wekk"], correct: 0 },
    SpellingQuestion { sentence: "The baby is ______.", options: ["sleeping", "sleping", "sleepin", "sleping"], correct: 0 },
    SpellingQuestion { sentence: "I have a ______ and a sister.", options: ["brother", "brotherr", "brothr", "brohter"], correct: 0 },
    SpellingQuestion { sentence: "My favorite ______ is pizza.", options: ["food", "fud", "foood", "foad"], correct: 0 },
    SpellingQuestion { sentence: "We go to ______ on Monday.", options: ["school", "scool", "skool", "school"], correct: 3 },
    SpellingQuestion { sentence: "The ______ is very tall.", options: ["tree", "tre", "trie", "tres"], correct: 0 },
    SpellingQuestion { sentence: "She ______ her teeth twice a day.", options: ["brushes", "brushs", "brushess", "brushes"], correct: 0 },
    SpellingQuestion { sentence: "The dog is ______ in the garden.", options: ["playing", "playng", "plaing", "playin"], correct: 0 },
    SpellingQuestion { sentence: "I eat ______ for breakfast.", options: ["bread", "bred", "braed", "breed"], correct: 0 },
    SpellingQuestion { sentence: "The ______ is cold today.", options: ["weather", "wheather", "wether", "weather"], correct: 3 },
    SpellingQuestion { sentence: "My mother is a ______.", options: ["teacher", "techer", "teacher", "teachr"], correct: 0 },
    SpellingQuestion { sentence: "The baby ______ every night.", options: ["cries", "crys", "cryes", "criez"], correct: 0 },
    SpellingQuestion { sentence: "I have a ______ in my pocket.", options: ["coin", "coyn", "koin", "coine"], correct: 0 },
    SpellingQuestion { sentence: "The ______ is on the wall.", options: ["picture", "pictur", "pichure", "picture"], correct: 3 },
    SpellingQuestion { sentence: "We ______ the ball.", options: ["throw", "thro", "throu", "thraw"], correct: 0 },
    SpellingQuestion { sentence: "The sky is ______.", options: ["blue", "blu", "blew", "blou"], correct: 0 },
    SpellingQuestion { sentence: "I need a ______ for the test.", options: ["pencil", "pensil", "pencill", "pencil"], correct: 3 },
    SpellingQuestion { sentence: "The ______ rings at noon.", options: ["bell", "bel", "belle", "bhel"], correct: 0 },
    SpellingQuestion { sentence: "My ______ is from Spain.", options: ["friend", "frend", "friind", "friend"], correct: 3 },
    SpellingQuestion { sentence: "The ______ has four legs.", options: ["chair", "cheir", "chare", "chair"], correct: 3 },
];

/// Frases de ortografía típicas del francés.
const SPELLING_QUESTIONS_FR: [SpellingQuestion; 30] = [
    SpellingQuestion { sentence: "L'______ fait du miel.", options: ["abeille", "abeile", "abeille", "abele"], correct: 0 },
    SpellingQuestion { sentence: "Le chien ______ toute la journée.", options: ["aboie", "aboye", "aboit", "aboi"], correct: 0 },
    SpellingQuestion { sentence: "J'aime ______ au parc.", options: ["jouer", "jover", "joué", "joue"], correct: 0 },
    SpellingQuestion { sentence: "La ______ a des pommes.", options: ["pomme", "pome", "pom", "pommes"], correct: 0 },
    SpellingQuestion { sentence: "Aujourd'hui il fait très ______.", options: ["chaud", "chot", "chaux", "chaud"], correct: 3 },
    SpellingQuestion { sentence: "La ______ de la fenêtre est cassée.", options: ["vitre", "vitre", "vite", "vitr"], correct: 0 },
    SpellingQuestion { sentence: "Dans le ______ il y a beaucoup d'étoiles.", options: ["ciel", "cielle", "cieu", "siel"], correct: 0 },
    SpellingQuestion { sentence: "Le ______ de l'arbre est très épais.", options: ["tronc", "tron", "tronc", "tronque"], correct: 0 },
    SpellingQuestion { sentence: "Ma grand-mère me ______ des histoires.", options: ["raconte", "racontt", "raconte", "raconce"], correct: 0 },
    SpellingQuestion { sentence: "L'______ vole très haut.", options: ["avion", "avions", "avion", "havion"], correct: 0 },
    SpellingQuestion { sentence: "J'ai acheté des ______ au marché.", options: ["raisins", "raisain", "ressins", "raisins"], correct: 3 },
    SpellingQuestion { sentence: "Le chat ______ sur le canapé.", options: ["dort", "dortt", "dore", "dor"], correct: 0 },
    SpellingQuestion { sentence: "La ______ est ma matière préférée.", options: ["musique", "musiqe", "musik", "musique"], correct: 3 },
    SpellingQuestion { sentence: "Quelle ______ est-il ?", options: ["heure", "heurre", "heures", "heure"], correct: 3 },
    SpellingQuestion { sentence: "______ hier au cinéma avec mon cousin.", options: ["Suis allé", "Suis allé", "Suis aller", "Suis allée"], correct: 0 },
    SpellingQuestion { sentence: "Le ______ du village est très ancien.", options: ["château", "chateau", "château", "chateaux"], correct: 0 },
    SpellingQuestion { sentence: "J'ai mal à la ______.", options: ["tête", "tête", "tette", "tête"], correct: 3 },
    SpellingQuestion { sentence: "L'______ est très propre.", options: ["hôpital", "hopital", "hôpital", "hospitale"], correct: 0 },
    SpellingQuestion { sentence: "Les ______ du jardin sont rouges.", options: ["roses", "roses", "rosse", "rosses"], correct: 0 },
    SpellingQuestion { sentence: "______ le crayon, s'il te plaît.", options: ["Prends", "Prend", "Pran", "Prends"], correct: 3 },
    SpellingQuestion { sentence: "En hiver il tombe de la ______.", options: ["neige", "neige", "neje", "neiges"], correct: 0 },
    SpellingQuestion { sentence: "Le ______ a beaucoup de pages.", options: ["livre", "livre", "livvre", "livre"], correct: 0 },
    SpellingQuestion { sentence: "La ______ du match était émouvante.", options: ["finale", "finale", "finall", "finalles"], correct: 0 },
    SpellingQuestion { sentence: "______ ans as-tu ?", options: ["Quel", "Quels", "Quelle", "Combien d'"], correct: 3 },
    SpellingQuestion { sentence: "Dans la ______ il y a beaucoup de livres.", options: ["bibliothèque", "bibliotèque", "bibliothèque", "biblioteque"], correct: 0 },
    SpellingQuestion { sentence: "Mon ami ______ très bien de la guitare.", options: ["joue", "jou", "joues", "joie"], correct: 0 },
    SpellingQuestion { sentence: "La banane est ______.", options: ["jaune", "jaune", "jaunes", "jaun"], correct: 0 },
    SpellingQuestion { sentence: "Où ______ les toilettes ?", options: ["sont", "son", "sont", "sonds"], correct: 0 },
    SpellingQuestion { sentence: "Le ______ du bateau est très haut.", options: ["mât", "mat", "mât", "maste"], correct: 0 },
    SpellingQuestion { sentence: "Le ______ est un animal très rapide.", options: ["lapin", "lapin", "lappin", "lapins"], correct: 0 },
];