//! Problemas de texto (Matemáticas) — enunciado con historia y 4 opciones.
//! 10 problemas, dificultad creciente, trilingüe.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

const ROUNDS: usize = 10;
const FEEDBACK_SECONDS: f32 = 1.4;

#[derive(Clone, Copy)]
struct Problem {
    text: &'static str,
    options: [&'static str; 4],
    correct: usize,
}

#[derive(Resource)]
pub struct WordProblemsSession {
    rounds: Vec<Problem>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

#[derive(Component)]
pub struct WordProblemsUiRoot;
#[derive(Component)]
pub struct WordProblemsText(WordProblemsField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum WordProblemsField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}
#[derive(Component)]
pub struct WordProblemsOptionText(pub usize);
#[derive(Component)]
pub struct WordProblemsOptionButton(pub usize);
#[derive(Component)]
pub struct WordProblemsResultBox;
#[derive(Component)]
pub struct WordProblemsBackButton;

pub struct WordProblemsPlugin;
impl Plugin for WordProblemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::WordProblemsPractice), spawn_wp_ui)
            .add_systems(OnExit(GameState::WordProblemsPractice), cleanup_wp)
            .add_systems(Update, update_wp.run_if(in_state(GameState::WordProblemsPractice)));
    }
}
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];
fn wp_text(parent: &mut ChildSpawnerCommands, field: WordProblemsField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((WordProblemsText(field), Text::new(text.to_string()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(700.0), ..default() }));
}
fn wp_option_text(parent: &mut ChildSpawnerCommands, index: usize, size: f32, font: &Handle<Font>) {
    parent.spawn((WordProblemsOptionText(index), Text::new(String::new()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE)));
}
fn spawn_wp_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(WordProblemsSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false });
    commands
        .spawn((WordProblemsUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                wp_text(panel, WordProblemsField::Title, "", 28.0, &font);
                wp_text(panel, WordProblemsField::Question, "", 24.0, &font);
                for index in 0..4 { panel.spawn((Button, WordProblemsOptionButton(index), Node { width: Val::Px(600.0), height: Val::Px(46.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.18, 0.28)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(8.0)))).with_children(|o| wp_option_text(o, index, 21.0, &font)); }
                wp_text(panel, WordProblemsField::Progress, "", 17.0, &font);
                wp_text(panel, WordProblemsField::Feedback, "", 22.0, &font);
                panel.spawn((WordProblemsResultBox, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(10.0), ..default() }, Visibility::Hidden)).with_children(|r| { wp_text(r, WordProblemsField::ResultTitle, "", 26.0, &font); wp_text(r, WordProblemsField::ResultDetail, "", 20.0, &font); spawn_button(r, "Volver a Matemáticas", WordProblemsBackButton, &font); });
            });
        });
}
fn cleanup_wp(mut commands: Commands, roots: Query<Entity, With<WordProblemsUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<WordProblemsSession>();
}
fn build_rounds() -> Vec<Problem> {
    let mut rng = rand::thread_rng();
    bank().choose_multiple(&mut rng, ROUNDS).copied().collect()
}
fn bank() -> &'static [Problem] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &PROBLEMS_EN,
        crate::i18n::Language::Fr => &PROBLEMS_FR,
        _ => &PROBLEMS,
    }
}
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);
fn update_wp(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<WordProblemsSession>>,
    mut texts: Query<(&WordProblemsText, &mut Text, &mut TextColor, &mut Visibility), (Without<WordProblemsOptionText>, Without<WordProblemsOptionButton>, Without<WordProblemsResultBox>)>,
    mut option_texts: Query<(&WordProblemsOptionText, &mut Text), (Without<WordProblemsText>, Without<WordProblemsOptionButton>, Without<WordProblemsResultBox>)>,
    mut option_colors: Query<(&WordProblemsOptionButton, &mut BackgroundColor), Without<WordProblemsText>>,
    option_clicks: Query<(&Interaction, &WordProblemsOptionButton), (Changed<Interaction>, Without<WordProblemsText>)>,
    mut result_box: Query<&mut Visibility, (With<WordProblemsResultBox>, Without<WordProblemsText>, Without<WordProblemsOptionButton>)>,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<WordProblemsBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session { Some(s) => s, None => { commands.insert_resource(WordProblemsSession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false }); return; } };
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::MathMenu); return; }
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyQ);
        if close { commands.set_state(GameState::MathMenu); return; }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                WordProblemsField::ResultTitle => { *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" })); *color = TextColor(if session.correct >= ROUNDS / 2 { Color::srgb(0.40, 0.90, 0.50) } else { Color::srgb(0.95, 0.55, 0.30) }); *vis = Visibility::Visible; }
                WordProblemsField::ResultDetail => { *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} problemas").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string())); *vis = Visibility::Visible; }
                _ => {}
            }
        }
        if let Ok(mut vis) = result_box.single_mut() { *vis = Visibility::Visible; }
        return;
    }
    if session.feedback {
        session.feedback_timer -= dt;
        for (button, mut bg) in &mut option_colors { let q = &session.rounds[session.index]; *bg = BackgroundColor(if button.0 == q.correct { OPTION_CORRECT } else if Some(button.0) == session.selected { OPTION_WRONG } else { OPTION_DIM }); }
        if session.feedback_timer <= 0.0 { session.feedback = false; session.selected = None; session.index += 1; if session.index >= ROUNDS { session.done = true; return; } }
    } else {
        let mut chosen: Option<usize> = None;
        for (interaction, button) in &option_clicks { if *interaction == Interaction::Pressed { chosen = Some(button.0); break; } }
        if chosen.is_none() { for (index, code) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4].iter().enumerate() { if keys.just_pressed(*code) { chosen = Some(index); break; } } }
        if let Some(index) = chosen { let q = &session.rounds[session.index]; if index == q.correct { session.correct += 1; play_success(&mut commands, &sfx); } else { session.wrong += 1; } session.selected = Some(index); session.feedback = true; session.feedback_timer = FEEDBACK_SECONDS; }
        for (_button, mut bg) in &mut option_colors { *bg = BackgroundColor(OPTION_NEUTRAL); }
    }
    let question = &session.rounds[session.index];
    for (field, mut text, mut color, mut vis) in &mut texts {
        match field.0 {
            WordProblemsField::Title => { *text = Text::new(tr("PROBLEMAS")); *color = TextColor(Color::srgb(1.0, 0.90, 0.50)); *vis = Visibility::Visible; }
            WordProblemsField::Question => { *text = Text::new(question.text.to_string()); *vis = Visibility::Visible; }
            WordProblemsField::Progress => { *text = Text::new(tr("Problema {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string())); *vis = Visibility::Visible; }
            WordProblemsField::Feedback => { if session.feedback { let ok = session.selected == Some(question.correct); if ok { *text = Text::new(tr("¡Correcto!")); *color = TextColor(Color::srgb(0.40, 0.90, 0.50)); } else { *text = Text::new(tr("Incorrecto — era {}) {}").replace("{}", &OPTION_LETTERS[question.correct].to_string()).replace("{}", &question.options[question.correct])); *color = TextColor(Color::srgb(0.95, 0.40, 0.40)); } *vis = Visibility::Visible; } else { *vis = Visibility::Hidden; } }
            _ => {}
        }
    }
    for (field, mut text) in &mut option_texts { *text = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], question.options[field.0])); }
    for (interaction, _button) in &option_clicks { if *interaction == Interaction::Pressed { play_click(&mut commands, &sfx); break; } }
}

const PROBLEMS: [Problem; 20] = [
    Problem { text: "Ana tiene 12 caramelos y da 5. ¿Cuántos le quedan?", options: ["5", "7", "12", "17"], correct: 1 },
    Problem { text: "Un autobús lleva 24 niños y suben 8 más. ¿Cuántos hay?", options: ["32", "30", "24", "16"], correct: 0 },
    Problem { text: "Luis tiene 36 cromos y los reparte entre 4 amigos. ¿Cuántos por amigo?", options: ["6", "8", "9", "12"], correct: 2 },
    Problem { text: "Una caja tiene 6 huevos. ¿Cuántos huevos en 5 cajas?", options: ["30", "25", "11", "35"], correct: 0 },
    Problem { text: "María compra 3 cuadernos a 2€ cada uno. ¿Cuánto paga?", options: ["5€", "6€", "9€", "3€"], correct: 1 },
    Problem { text: "Un tren va a 60 km/h. ¿Cuánto recorre en 2 horas?", options: ["60 km", "100 km", "120 km", "30 km"], correct: 2 },
    Problem { text: "Tienes 20€ y gastas 7€. ¿Cuánto te queda?", options: ["13€", "12€", "27€", "7€"], correct: 0 },
    Problem { text: "Un rectángulo mide 5×4. ¿Cuál es su área?", options: ["9", "10", "20", "16"], correct: 2 },
    Problem { text: "Si 1 kg de manzanas cuesta 3€, ¿cuánto cuestan 4 kg?", options: ["7€", "12€", "8€", "9€"], correct: 1 },
    Problem { text: "Javier lee 15 páginas al día. ¿Cuántas en 7 días?", options: ["105", "85", "75", "95"], correct: 0 },
    Problem { text: "Un paquete tiene 12 galletas. ¿Cuántas en 3 paquetes?", options: ["36", "24", "15", "30"], correct: 0 },
    Problem { text: "De 50 alumnos, 20 son chicas. ¿Cuántos son chicos?", options: ["20", "30", "25", "35"], correct: 1 },
    Problem { text: "Un coche consume 8L cada 100km. ¿Cuánto en 200km?", options: ["8L", "12L", "16L", "20L"], correct: 2 },
    Problem { text: "Tienes 100 canicas y pierdes 1/4. ¿Cuántas pierdes?", options: ["20", "25", "30", "50"], correct: 1 },
    Problem { text: "Un reloj marca las 3:15. ¿Cuántos minutos han pasado desde las 3:00?", options: ["10", "15", "30", "45"], correct: 1 },
    Problem { text: "Mamá hace 24 magdalenas y las pone en bandejas de 6. ¿Cuántas bandejas?", options: ["4", "6", "3", "8"], correct: 0 },
    Problem { text: "Un campo es cuadrado de lado 9m. ¿Perímetro?", options: ["18m", "27m", "36m", "81m"], correct: 2 },
    Problem { text: "Compras 2 zumos a 1,50€ cada uno. ¿Total?", options: ["2€", "3€", "1,50€", "4€"], correct: 1 },
    Problem { text: "Un libro tiene 80 páginas y has leído 1/2. ¿Cuántas leídas?", options: ["20", "40", "60", "80"], correct: 1 },
    Problem { text: "Naciste en 2015. ¿Cuántos años tienes en 2026?", options: ["10", "11", "12", "9"], correct: 1 },
];
const PROBLEMS_EN: [Problem; 20] = [
    Problem { text: "Ana has 12 sweets and gives 5 away. How many left?", options: ["5", "7", "12", "17"], correct: 1 },
    Problem { text: "A bus has 24 children and 8 more get on. How many now?", options: ["32", "30", "24", "16"], correct: 0 },
    Problem { text: "Luis has 36 stickers shared among 4 friends. How many each?", options: ["6", "8", "9", "12"], correct: 2 },
    Problem { text: "A box has 6 eggs. How many eggs in 5 boxes?", options: ["30", "25", "11", "35"], correct: 0 },
    Problem { text: "Maria buys 3 notebooks at 2€ each. How much?", options: ["5€", "6€", "9€", "3€"], correct: 1 },
    Problem { text: "A train goes 60 km/h. How far in 2 hours?", options: ["60 km", "100 km", "120 km", "30 km"], correct: 2 },
    Problem { text: "You have 20€ and spend 7€. How much left?", options: ["13€", "12€", "27€", "7€"], correct: 0 },
    Problem { text: "A rectangle is 5×4. What is its area?", options: ["9", "10", "20", "16"], correct: 2 },
    Problem { text: "If 1 kg apples costs 3€, how much for 4 kg?", options: ["7€", "12€", "8€", "9€"], correct: 1 },
    Problem { text: "Javier reads 15 pages a day. How many in 7 days?", options: ["105", "85", "75", "95"], correct: 0 },
    Problem { text: "A pack has 12 cookies. How many in 3 packs?", options: ["36", "24", "15", "30"], correct: 0 },
    Problem { text: "Of 50 pupils, 20 are girls. How many boys?", options: ["20", "30", "25", "35"], correct: 1 },
    Problem { text: "A car uses 8L per 100km. How much in 200km?", options: ["8L", "12L", "16L", "20L"], correct: 2 },
    Problem { text: "You have 100 marbles and lose 1/4. How many lost?", options: ["20", "25", "30", "50"], correct: 1 },
    Problem { text: "A clock shows 3:15. Minutes past 3:00?", options: ["10", "15", "30", "45"], correct: 1 },
    Problem { text: "Mom bakes 24 muffins on trays of 6. How many trays?", options: ["4", "6", "3", "8"], correct: 0 },
    Problem { text: "A square field side 9m. Perimeter?", options: ["18m", "27m", "36m", "81m"], correct: 2 },
    Problem { text: "You buy 2 juices at 1.50€ each. Total?", options: ["2€", "3€", "1.50€", "4€"], correct: 1 },
    Problem { text: "A book has 80 pages and you read 1/2. How many read?", options: ["20", "40", "60", "80"], correct: 1 },
    Problem { text: "Born in 2015. How old in 2026?", options: ["10", "11", "12", "9"], correct: 1 },
];
const PROBLEMS_FR: [Problem; 20] = [
    Problem { text: "Ana a 12 bonbons et en donne 5. Combien reste-t-il ?", options: ["5", "7", "12", "17"], correct: 1 },
    Problem { text: "Un bus a 24 enfants et 8 montent. Combien ?", options: ["32", "30", "24", "16"], correct: 0 },
    Problem { text: "Luis a 36 images pour 4 amis. Combien chacun ?", options: ["6", "8", "9", "12"], correct: 2 },
    Problem { text: "Une boîte a 6 œufs. Combien dans 5 boîtes ?", options: ["30", "25", "11", "35"], correct: 0 },
    Problem { text: "Maria achète 3 cahiers à 2€ chacun. Combien ?", options: ["5€", "6€", "9€", "3€"], correct: 1 },
    Problem { text: "Un train va à 60 km/h. Distance en 2h ?", options: ["60 km", "100 km", "120 km", "30 km"], correct: 2 },
    Problem { text: "Tu as 20€ et dépenses 7€. Reste ?", options: ["13€", "12€", "27€", "7€"], correct: 0 },
    Problem { text: "Un rectangle 5×4. Aire ?", options: ["9", "10", "20", "16"], correct: 2 },
    Problem { text: "1 kg pommes 3€, combien pour 4 kg ?", options: ["7€", "12€", "8€", "9€"], correct: 1 },
    Problem { text: "Javier lit 15 pages par jour. En 7 jours ?", options: ["105", "85", "75", "95"], correct: 0 },
    Problem { text: "Un paquet a 12 gâteaux. Combien dans 3 paquets ?", options: ["36", "24", "15", "30"], correct: 0 },
    Problem { text: "Sur 50 élèves, 20 filles. Combien de garçons ?", options: ["20", "30", "25", "35"], correct: 1 },
    Problem { text: "Voiture 8L/100km. Combien pour 200km ?", options: ["8L", "12L", "16L", "20L"], correct: 2 },
    Problem { text: "Tu as 100 billes et perds 1/4. Combien perdues ?", options: ["20", "25", "30", "50"], correct: 1 },
    Problem { text: "Horloge 3h15. Minutes depuis 3h00 ?", options: ["10", "15", "30", "45"], correct: 1 },
    Problem { text: "Maman fait 24 muffins par plateaux de 6. Combien ?", options: ["4", "6", "3", "8"], correct: 0 },
    Problem { text: "Champ carré côté 9m. Périmètre ?", options: ["18m", "27m", "36m", "81m"], correct: 2 },
    Problem { text: "2 jus à 1,50€ chacun. Total ?", options: ["2€", "3€", "1,50€", "4€"], correct: 1 },
    Problem { text: "Livre 80 pages, lu 1/2. Combien lues ?", options: ["20", "40", "60", "80"], correct: 1 },
    Problem { text: "Né en 2015. Âge en 2026 ?", options: ["10", "11", "12", "9"], correct: 1 },
];
