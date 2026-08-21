//! Juego de Sinónimos (Lengua) — elige el sinónimo correcto.
//!
//! Sesión de 10 palabras al azar. Cada palabra muestra 4 opciones y una es
//! sinónimo. Feedback 1,4s y marcador final. Trilingüe ES/EN/FR.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

const ROUNDS: usize = 10;
const FEEDBACK_SECONDS: f32 = 1.4;

#[derive(Clone, Copy)]
struct SynonymQuestion {
    word: &'static str,
    options: [&'static str; 4],
    correct: usize,
}

#[derive(Resource)]
pub struct SynonymSession {
    rounds: Vec<SynonymQuestion>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

#[derive(Component)]
pub struct SynonymUiRoot;
#[derive(Component)]
pub struct SynonymText(SynonymField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum SynonymField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}
#[derive(Component)]
pub struct SynonymOptionText(pub usize);
#[derive(Component)]
pub struct SynonymOptionButton(pub usize);
#[derive(Component)]
pub struct SynonymResultBox;
#[derive(Component)]
pub struct SynonymBackButton;

pub struct SynonymPlugin;

impl Plugin for SynonymPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::SynonymsPractice), spawn_synonym_ui)
            .add_systems(OnExit(GameState::SynonymsPractice), cleanup_synonym)
            .add_systems(Update, update_synonym.run_if(in_state(GameState::SynonymsPractice)));
    }
}

const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];

fn synonym_text(parent: &mut ChildSpawnerCommands, field: SynonymField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((
        SynonymText(field),
        Text::new(text.to_string()),
        TextFont { font: font.clone(), font_size: size, ..default() },
        TextColor(Color::WHITE),
        TextLayout { linebreak: LineBreak::WordBoundary, ..default() },
        Node { max_width: Val::Px(700.0), ..default() },
    ));
}
fn synonym_option_text(parent: &mut ChildSpawnerCommands, index: usize, size: f32, font: &Handle<Font>) {
    parent.spawn((
        SynonymOptionText(index),
        Text::new(String::new()),
        TextFont { font: font.clone(), font_size: size, ..default() },
        TextColor(Color::WHITE),
    ));
}

fn spawn_synonym_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(SynonymSession {
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
            SynonymUiRoot,
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
                    synonym_text(panel, SynonymField::Title, "", 28.0, &font);
                    synonym_text(panel, SynonymField::Question, "", 26.0, &font);
                    for index in 0..4 {
                        panel
                            .spawn((
                                Button,
                                SynonymOptionButton(index),
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
                                synonym_option_text(option, index, 21.0, &font);
                            });
                    }
                    synonym_text(panel, SynonymField::Progress, "", 17.0, &font);
                    synonym_text(panel, SynonymField::Feedback, "", 22.0, &font);
                    panel
                        .spawn((
                            SynonymResultBox,
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.0),
                                ..default()
                            },
                            Visibility::Hidden,
                        ))
                        .with_children(|results| {
                            synonym_text(results, SynonymField::ResultTitle, "", 26.0, &font);
                            synonym_text(results, SynonymField::ResultDetail, "", 20.0, &font);
                            spawn_button(results, "Volver a Lengua", SynonymBackButton, &font);
                        });
                });
        });
}

fn cleanup_synonym(mut commands: Commands, roots: Query<Entity, With<SynonymUiRoot>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
    commands.remove_resource::<SynonymSession>();
}

fn build_rounds() -> Vec<SynonymQuestion> {
    let mut rng = rand::thread_rng();
    bank().choose_multiple(&mut rng, ROUNDS).copied().collect()
}
fn bank() -> &'static [SynonymQuestion] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &SYNONYMS_EN,
        crate::i18n::Language::Fr => &SYNONYMS_FR,
        crate::i18n::Language::Es => &SYNONYMS,
    }
}

const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);

fn update_synonym(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<SynonymSession>>,
    mut texts: Query<(&SynonymText, &mut Text, &mut TextColor, &mut Visibility), (Without<SynonymOptionText>, Without<SynonymOptionButton>, Without<SynonymResultBox>)>,
    mut option_texts: Query<(&SynonymOptionText, &mut Text), (Without<SynonymText>, Without<SynonymOptionButton>, Without<SynonymResultBox>)>,
    mut option_colors: Query<(&SynonymOptionButton, &mut BackgroundColor), Without<SynonymText>>,
    option_clicks: Query<(&Interaction, &SynonymOptionButton), (Changed<Interaction>, Without<SynonymText>)>,
    mut result_box: Query<&mut Visibility, (With<SynonymResultBox>, Without<SynonymText>, Without<SynonymOptionButton>)>,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<SynonymBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session {
        Some(s) => s,
        None => {
            commands.insert_resource(SynonymSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false });
            return;
        }
    };
    if keys.just_pressed(KeyCode::Escape) {
        commands.set_state(GameState::LanguageMenu);
        return;
    }
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyQ);
        if close {
            commands.set_state(GameState::LanguageMenu);
            return;
        }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                SynonymField::ResultTitle => {
                    *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" }));
                    *color = TextColor(if session.correct >= ROUNDS / 2 { Color::srgb(0.40, 0.90, 0.50) } else { Color::srgb(0.95, 0.55, 0.30) });
                    *vis = Visibility::Visible;
                }
                SynonymField::ResultDetail => {
                    *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} palabras").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string()));
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
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors {
            let q = &session.rounds[session.index];
            *bg = BackgroundColor(if button.0 == q.correct { OPTION_CORRECT } else if Some(button.0) == session.selected { OPTION_WRONG } else { OPTION_DIM });
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
        let mut chosen: Option<usize> = None;
        for (interaction, button) in &option_clicks {
            if *interaction == Interaction::Pressed {
                chosen = Some(button.0);
                break;
            }
        }
        if chosen.is_none() {
            for (index, code) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() {
                if keys.just_pressed(*code) {
                    chosen = Some(index);
                    break;
                }
            }
        }
        if let Some(index) = chosen {
            let q = &session.rounds[session.index];
            if index == q.correct {
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
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            SynonymField::Title => {
                *text = Text::new(tr("SINÓNIMOS"));
                *color = TextColor(Color::srgb(0.80, 0.75, 1.0));
                *vis = Visibility::Visible;
            }
            SynonymField::Question => {
                *text = Text::new(tr("Sinónimo de \"{}\"").replace("{}", question.word));
                *vis = Visibility::Visible;
            }
            SynonymField::Progress => {
                *text = Text::new(tr("Palabra {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()));
                *vis = Visibility::Visible;
            }
            SynonymField::Feedback => {
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
        *text = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], question.options[field.0]));
    }
    for (interaction, _button) in &option_clicks {
        if *interaction == Interaction::Pressed {
            play_click(&mut commands, &sfx);
            break;
        }
    }
}

// ---- Banco ES ----
const SYNONYMS: [SynonymQuestion; 30] = [
    SynonymQuestion { word: "alegre", options: ["contento", "triste", "enfadado", "serio"], correct: 0 },
    SynonymQuestion { word: "rápido", options: ["lento", "veloz", "pequeño", "alto"], correct: 1 },
    SynonymQuestion { word: "grande", options: ["pequeño", "enorme", "corto", "rojo"], correct: 1 },
    SynonymQuestion { word: "bonito", options: ["feo", "hermoso", "viejo", "duro"], correct: 1 },
    SynonymQuestion { word: "listo", options: ["tonto", "inteligente", "alto", "gordo"], correct: 1 },
    SynonymQuestion { word: "valiente", options: ["cobarde", "miedoso", "audaz", "débil"], correct: 2 },
    SynonymQuestion { word: "enfadado", options: ["contento", "enojado", "tranquilo", "feliz"], correct: 1 },
    SynonymQuestion { word: "difícil", options: ["fácil", "complicado", "corto", "claro"], correct: 1 },
    SynonymQuestion { word: "tranquilo", options: ["nervioso", "sereno", "rápido", "ruidoso"], correct: 1 },
    SynonymQuestion { word: "fuerte", options: ["débil", "robusto", "pequeño", "blando"], correct: 1 },
    SynonymQuestion { word: "pequeño", options: ["grande", "diminuto", "largo", "alto"], correct: 1 },
    SynonymQuestion { word: "hablar", options: ["callar", "charlar", "escuchar", "mirar"], correct: 1 },
    SynonymQuestion { word: "casa", options: ["hogar", "calle", "coche", "árbol"], correct: 0 },
    SynonymQuestion { word: "amigo", options: ["enemigo", "compañero", "jefe", "vecino"], correct: 1 },
    SynonymQuestion { word: "comenzar", options: ["terminar", "empezar", "parar", "seguir"], correct: 1 },
    SynonymQuestion { word: "observar", options: ["mirar", "olvidar", "tocar", "oír"], correct: 0 },
    SynonymQuestion { word: "cansado", options: ["activo", "agotado", "despierto", "fresco"], correct: 1 },
    SynonymQuestion { word: "regalo", options: ["castigo", "obsequio", "compra", "deuda"], correct: 1 },
    SynonymQuestion { word: "veloz", options: ["lento", "rápido", "pesado", "corto"], correct: 1 },
    SynonymQuestion { word: "sencillo", options: ["difícil", "fácil", "caro", "largo"], correct: 1 },
    SynonymQuestion { word: "oscuridad", options: ["luz", "tiniebla", "día", "sol"], correct: 1 },
    SynonymQuestion { word: "alumno", options: ["profesor", "estudiante", "director", "padre"], correct: 1 },
    SynonymQuestion { word: "contar", options: ["narrar", "callar", "borrar", "romper"], correct: 0 },
    SynonymQuestion { word: "brillante", options: ["opaco", "luminoso", "oscuro", "mate"], correct: 1 },
    SynonymQuestion { word: "pregunta", options: ["respuesta", "interrogante", "afirmación", "orden"], correct: 1 },
    SynonymQuestion { word: "ayuda", options: ["estorbo", "apoyo", "carga", "problema"], correct: 1 },
    SynonymQuestion { word: "terminar", options: ["empezar", "finalizar", "continuar", "abrir"], correct: 1 },
    SynonymQuestion { word: "elegir", options: ["rechazar", "escoger", "perder", "olvidar"], correct: 1 },
    SynonymQuestion { word: "feliz", options: ["triste", "dichoso", "enfadado", "aburrido"], correct: 1 },
    SynonymQuestion { word: "unir", options: ["separar", "juntar", "cortar", "romper"], correct: 1 },
];

const SYNONYMS_EN: [SynonymQuestion; 30] = [
    SynonymQuestion { word: "happy", options: ["joyful", "sad", "angry", "serious"], correct: 0 },
    SynonymQuestion { word: "fast", options: ["slow", "quick", "small", "tall"], correct: 1 },
    SynonymQuestion { word: "big", options: ["small", "huge", "short", "red"], correct: 1 },
    SynonymQuestion { word: "pretty", options: ["ugly", "beautiful", "old", "hard"], correct: 1 },
    SynonymQuestion { word: "smart", options: ["silly", "clever", "tall", "fat"], correct: 1 },
    SynonymQuestion { word: "brave", options: ["cowardly", "fearful", "bold", "weak"], correct: 2 },
    SynonymQuestion { word: "angry", options: ["happy", "mad", "calm", "cheerful"], correct: 1 },
    SynonymQuestion { word: "difficult", options: ["easy", "hard", "short", "clear"], correct: 1 },
    SynonymQuestion { word: "calm", options: ["nervous", "serene", "fast", "noisy"], correct: 1 },
    SynonymQuestion { word: "strong", options: ["weak", "sturdy", "small", "soft"], correct: 1 },
    SynonymQuestion { word: "small", options: ["big", "tiny", "long", "tall"], correct: 1 },
    SynonymQuestion { word: "talk", options: ["be silent", "chat", "listen", "look"], correct: 1 },
    SynonymQuestion { word: "house", options: ["home", "street", "car", "tree"], correct: 0 },
    SynonymQuestion { word: "friend", options: ["enemy", "companion", "boss", "neighbour"], correct: 1 },
    SynonymQuestion { word: "begin", options: ["end", "start", "stop", "continue"], correct: 1 },
    SynonymQuestion { word: "observe", options: ["watch", "forget", "touch", "hear"], correct: 0 },
    SynonymQuestion { word: "tired", options: ["active", "exhausted", "awake", "fresh"], correct: 1 },
    SynonymQuestion { word: "gift", options: ["punishment", "present", "purchase", "debt"], correct: 1 },
    SynonymQuestion { word: "quick", options: ["slow", "fast", "heavy", "short"], correct: 1 },
    SynonymQuestion { word: "simple", options: ["hard", "easy", "expensive", "long"], correct: 1 },
    SynonymQuestion { word: "darkness", options: ["light", "gloom", "day", "sun"], correct: 1 },
    SynonymQuestion { word: "pupil", options: ["teacher", "student", "principal", "parent"], correct: 1 },
    SynonymQuestion { word: "tell", options: ["narrate", "be silent", "erase", "break"], correct: 0 },
    SynonymQuestion { word: "bright", options: ["dull", "shining", "dark", "matt"], correct: 1 },
    SynonymQuestion { word: "question", options: ["answer", "query", "statement", "order"], correct: 1 },
    SynonymQuestion { word: "help", options: ["hindrance", "support", "load", "problem"], correct: 1 },
    SynonymQuestion { word: "finish", options: ["start", "complete", "continue", "open"], correct: 1 },
    SynonymQuestion { word: "choose", options: ["reject", "pick", "lose", "forget"], correct: 1 },
    SynonymQuestion { word: "joyful", options: ["sad", "happy", "angry", "bored"], correct: 1 },
    SynonymQuestion { word: "join", options: ["separate", "unite", "cut", "break"], correct: 1 },
];

const SYNONYMS_FR: [SynonymQuestion; 30] = [
    SynonymQuestion { word: "joyeux", options: ["content", "triste", "fâché", "sérieux"], correct: 0 },
    SynonymQuestion { word: "rapide", options: ["lent", "vite", "petit", "grand"], correct: 1 },
    SynonymQuestion { word: "grand", options: ["petit", "énorme", "court", "rouge"], correct: 1 },
    SynonymQuestion { word: "joli", options: ["laid", "beau", "vieux", "dur"], correct: 1 },
    SynonymQuestion { word: "intelligent", options: ["bête", "malin", "grand", "gros"], correct: 1 },
    SynonymQuestion { word: "courageux", options: ["peureux", "craintif", "audacieux", "faible"], correct: 2 },
    SynonymQuestion { word: "fâché", options: ["content", "en colère", "calme", "heureux"], correct: 1 },
    SynonymQuestion { word: "difficile", options: ["facile", "compliqué", "court", "clair"], correct: 1 },
    SynonymQuestion { word: "calme", options: ["nerveux", "serein", "rapide", "bruyant"], correct: 1 },
    SynonymQuestion { word: "fort", options: ["faible", "robuste", "petit", "mou"], correct: 1 },
    SynonymQuestion { word: "petit", options: ["grand", "minuscule", "long", "haut"], correct: 1 },
    SynonymQuestion { word: "parler", options: ["se taire", "bavarder", "écouter", "regarder"], correct: 1 },
    SynonymQuestion { word: "maison", options: ["foyer", "rue", "voiture", "arbre"], correct: 0 },
    SynonymQuestion { word: "ami", options: ["ennemi", "copain", "chef", "voisin"], correct: 1 },
    SynonymQuestion { word: "commencer", options: ["finir", "débuter", "arrêter", "continuer"], correct: 1 },
    SynonymQuestion { word: "observer", options: ["regarder", "oublier", "toucher", "entendre"], correct: 0 },
    SynonymQuestion { word: "fatigué", options: ["actif", "épuisé", "éveillé", "frais"], correct: 1 },
    SynonymQuestion { word: "cadeau", options: ["punition", "présent", "achat", "dette"], correct: 1 },
    SynonymQuestion { word: "vite", options: ["lent", "rapide", "lourd", "court"], correct: 1 },
    SynonymQuestion { word: "simple", options: ["difficile", "facile", "cher", "long"], correct: 1 },
    SynonymQuestion { word: "obscurité", options: ["lumière", "ténèbres", "jour", "soleil"], correct: 1 },
    SynonymQuestion { word: "élève", options: ["professeur", "étudiant", "directeur", "parent"], correct: 1 },
    SynonymQuestion { word: "raconter", options: ["narrer", "se taire", "effacer", "casser"], correct: 0 },
    SynonymQuestion { word: "brillant", options: ["terne", "lumineux", "sombre", "mat"], correct: 1 },
    SynonymQuestion { word: "question", options: ["réponse", "interrogation", "affirmation", "ordre"], correct: 1 },
    SynonymQuestion { word: "aide", options: ["obstacle", "soutien", "charge", "problème"], correct: 1 },
    SynonymQuestion { word: "finir", options: ["commencer", "terminer", "continuer", "ouvrir"], correct: 1 },
    SynonymQuestion { word: "choisir", options: ["rejeter", "sélectionner", "perdre", "oublier"], correct: 1 },
    SynonymQuestion { word: "heureux", options: ["triste", "joyeux", "fâché", "ennuyé"], correct: 1 },
    SynonymQuestion { word: "unir", options: ["séparer", "joindre", "couper", "casser"], correct: 1 },
];
