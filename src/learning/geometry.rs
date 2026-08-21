//! Geometría (Matemáticas) — perímetros, áreas, ángulos, figuras.
//! 10 preguntas, 4 opciones, trilingüe.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use super::{screen_background, spawn_button};
use crate::audio::{play_click, play_success, Sfx};
use crate::game::GameState;
use crate::i18n::tr;

const ROUNDS: usize = 10;
const FEEDBACK_SECONDS: f32 = 1.4;

#[derive(Clone, Copy)]
struct GeometryQuestion {
    question: &'static str,
    options: [&'static str; 4],
    correct: usize,
}

#[derive(Resource)]
pub struct GeometrySession {
    rounds: Vec<GeometryQuestion>,
    index: usize,
    correct: usize,
    wrong: usize,
    selected: Option<usize>,
    feedback: bool,
    feedback_timer: f32,
    done: bool,
}

#[derive(Component)]
pub struct GeometryUiRoot;
#[derive(Component)]
pub struct GeometryText(GeometryField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum GeometryField {
    Title,
    Question,
    Progress,
    Feedback,
    ResultTitle,
    ResultDetail,
}
#[derive(Component)]
pub struct GeometryOptionText(pub usize);
#[derive(Component)]
pub struct GeometryOptionButton(pub usize);
#[derive(Component)]
pub struct GeometryResultBox;
#[derive(Component)]
pub struct GeometryBackButton;

pub struct GeometryPlugin;
impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GeometryPractice), spawn_geometry_ui)
            .add_systems(OnExit(GameState::GeometryPractice), cleanup_geometry)
            .add_systems(Update, update_geometry.run_if(in_state(GameState::GeometryPractice)));
    }
}
const OPTION_LETTERS: [char; 4] = ['A', 'B', 'C', 'D'];
fn geometry_text(parent: &mut ChildSpawnerCommands, field: GeometryField, text: &str, size: f32, font: &Handle<Font>) {
    parent.spawn((GeometryText(field), Text::new(text.to_string()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(700.0), ..default() }));
}
fn geometry_option_text(parent: &mut ChildSpawnerCommands, index: usize, size: f32, font: &Handle<Font>) {
    parent.spawn((GeometryOptionText(index), Text::new(String::new()), TextFont { font: font.clone(), font_size: size, ..default() }, TextColor(Color::WHITE)));
}
fn spawn_geometry_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(GeometrySession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false });
    commands
        .spawn((GeometryUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(680.0), padding: UiRect::axes(Val::Px(28.0), Val::Px(24.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                geometry_text(panel, GeometryField::Title, "", 28.0, &font);
                geometry_text(panel, GeometryField::Question, "", 26.0, &font);
                for index in 0..4 { panel.spawn((Button, GeometryOptionButton(index), Node { width: Val::Px(600.0), height: Val::Px(46.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.18, 0.28)), BorderColor(Color::srgb(0.50, 0.55, 0.70)), BorderRadius::all(Val::Px(8.0)))).with_children(|o| geometry_option_text(o, index, 21.0, &font)); }
                geometry_text(panel, GeometryField::Progress, "", 17.0, &font);
                geometry_text(panel, GeometryField::Feedback, "", 22.0, &font);
                panel.spawn((GeometryResultBox, Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(10.0), ..default() }, Visibility::Hidden)).with_children(|r| { geometry_text(r, GeometryField::ResultTitle, "", 26.0, &font); geometry_text(r, GeometryField::ResultDetail, "", 20.0, &font); spawn_button(r, "Volver a Matemáticas", GeometryBackButton, &font); });
            });
        });
}
fn cleanup_geometry(mut commands: Commands, roots: Query<Entity, With<GeometryUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<GeometrySession>();
}
fn build_rounds() -> Vec<GeometryQuestion> {
    let mut rng = rand::thread_rng();
    bank().choose_multiple(&mut rng, ROUNDS).copied().collect()
}
fn bank() -> &'static [GeometryQuestion] {
    match crate::i18n::language() {
        crate::i18n::Language::En => &GEOMETRY_EN,
        crate::i18n::Language::Fr => &GEOMETRY_FR,
        _ => &GEOMETRY,
    }
}
const OPTION_NEUTRAL: Color = Color::srgb(0.15, 0.18, 0.28);
const OPTION_DIM: Color = Color::srgb(0.10, 0.12, 0.20);
const OPTION_CORRECT: Color = Color::srgb(0.15, 0.42, 0.25);
const OPTION_WRONG: Color = Color::srgb(0.50, 0.20, 0.20);
fn update_geometry(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<Sfx>,
    session: Option<ResMut<GeometrySession>>,
    mut texts: Query<(&GeometryText, &mut Text, &mut TextColor, &mut Visibility), (Without<GeometryOptionText>, Without<GeometryOptionButton>, Without<GeometryResultBox>)>,
    mut option_texts: Query<(&GeometryOptionText, &mut Text), (Without<GeometryText>, Without<GeometryOptionButton>, Without<GeometryResultBox>)>,
    mut option_colors: Query<(&GeometryOptionButton, &mut BackgroundColor), Without<GeometryText>>,
    option_clicks: Query<(&Interaction, &GeometryOptionButton), (Changed<Interaction>, Without<GeometryText>)>,
    mut result_box: Query<&mut Visibility, (With<GeometryResultBox>, Without<GeometryText>, Without<GeometryOptionButton>)>,
    close_clicks: Query<&Interaction, (Changed<Interaction>, With<GeometryBackButton>)>,
) {
    let dt = time.delta_secs();
    let mut session = match session { Some(s) => s, None => { commands.insert_resource(GeometrySession { rounds: build_rounds(), index: 0, correct: 0, wrong: 0, selected: None, feedback: false, feedback_timer: 0.0, done: false }); return; } };
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::MathMenu); return; }
    if session.done {
        let close = close_clicks.single().map_or(false, |i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyQ);
        if close { commands.set_state(GameState::MathMenu); return; }
        for (field, mut text, mut color, mut vis) in &mut texts {
            match field.0 {
                GeometryField::ResultTitle => { *text = Text::new(tr(if session.correct >= ROUNDS / 2 { "¡Muy bien!" } else { "¡Sigue practicando!" })); *color = TextColor(if session.correct >= ROUNDS / 2 { Color::srgb(0.40, 0.90, 0.50) } else { Color::srgb(0.95, 0.55, 0.30) }); *vis = Visibility::Visible; }
                GeometryField::ResultDetail => { *text = Text::new(tr("Aciertos: {} · Fallos: {}  de {} preguntas").replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string()).replace("{}", &ROUNDS.to_string())); *vis = Visibility::Visible; }
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
            GeometryField::Title => { *text = Text::new(tr("GEOMETRÍA")); *color = TextColor(Color::srgb(1.0, 0.90, 0.50)); *vis = Visibility::Visible; }
            GeometryField::Question => { *text = Text::new(question.question.to_string()); *vis = Visibility::Visible; }
            GeometryField::Progress => { *text = Text::new(tr("Pregunta {}/{}  ·  Aciertos: {}  ·  Fallos: {}").replace("{}", &(session.index + 1).to_string()).replace("{}", &ROUNDS.to_string()).replace("{}", &session.correct.to_string()).replace("{}", &session.wrong.to_string())); *vis = Visibility::Visible; }
            GeometryField::Feedback => { if session.feedback { let ok = session.selected == Some(question.correct); if ok { *text = Text::new(tr("¡Correcto!")); *color = TextColor(Color::srgb(0.40, 0.90, 0.50)); } else { *text = Text::new(tr("Incorrecto — era {}) {}").replace("{}", &OPTION_LETTERS[question.correct].to_string()).replace("{}", &question.options[question.correct])); *color = TextColor(Color::srgb(0.95, 0.40, 0.40)); } *vis = Visibility::Visible; } else { *vis = Visibility::Hidden; } }
            _ => {}
        }
    }
    for (field, mut text) in &mut option_texts { *text = Text::new(format!("{}) {}", OPTION_LETTERS[field.0], question.options[field.0])); }
    for (interaction, _button) in &option_clicks { if *interaction == Interaction::Pressed { play_click(&mut commands, &sfx); break; } }
}

const GEOMETRY: [GeometryQuestion; 30] = [
    GeometryQuestion { question: "¿Cuántos lados tiene un pentágono?", options: ["4", "5", "6", "7"], correct: 1 },
    GeometryQuestion { question: "¿Cuántos lados tiene un hexágono?", options: ["5", "6", "7", "8"], correct: 1 },
    GeometryQuestion { question: "¿Cuántos lados tiene un octógono?", options: ["6", "7", "8", "9"], correct: 2 },
    GeometryQuestion { question: "Perímetro de un cuadrado de lado 6", options: ["12", "18", "24", "36"], correct: 2 },
    GeometryQuestion { question: "Área de un rectángulo 5×8", options: ["13", "26", "40", "45"], correct: 2 },
    GeometryQuestion { question: "¿Cuántos vértices tiene un cubo?", options: ["6", "8", "12", "4"], correct: 1 },
    GeometryQuestion { question: "¿Cuántas caras tiene un cubo?", options: ["4", "6", "8", "12"], correct: 1 },
    GeometryQuestion { question: "Ángulo recto mide", options: ["45°", "90°", "180°", "360°"], correct: 1 },
    GeometryQuestion { question: "Suma de ángulos de un triángulo", options: ["90°", "180°", "270°", "360°"], correct: 1 },
    GeometryQuestion { question: "Suma de ángulos de un cuadrado", options: ["180°", "270°", "360°", "90°"], correct: 2 },
    GeometryQuestion { question: "Perímetro de un triángulo equilátero lado 7", options: ["14", "21", "28", "49"], correct: 1 },
    GeometryQuestion { question: "Área de un triángulo base 10 altura 6", options: ["60", "30", "15", "45"], correct: 1 },
    GeometryQuestion { question: "¿Cuántos grados tiene un ángulo llano?", options: ["90°", "180°", "270°", "360°"], correct: 1 },
    GeometryQuestion { question: "¿Cuántos lados tiene un decágono?", options: ["8", "9", "10", "12"], correct: 2 },
    GeometryQuestion { question: "Perímetro de un rectángulo 7×3", options: ["10", "20", "21", "14"], correct: 1 },
    GeometryQuestion { question: "Área de un cuadrado lado 9", options: ["36", "81", "18", "72"], correct: 1 },
    GeometryQuestion { question: "¿Cuántas aristas tiene un cubo?", options: ["6", "8", "12", "10"], correct: 2 },
    GeometryQuestion { question: "¿Cuántos vértices tiene un triángulo?", options: ["2", "3", "4", "6"], correct: 1 },
    GeometryQuestion { question: "Ángulo agudo es", options: ["<90°", "90°", ">90°", "180°"], correct: 0 },
    GeometryQuestion { question: "Ángulo obtuso es", options: ["<90°", "90°", ">90° y <180°", "180°"], correct: 2 },
    GeometryQuestion { question: "¿Cuántos lados tiene un heptágono?", options: ["6", "7", "8", "9"], correct: 1 },
    GeometryQuestion { question: "Perímetro de un pentágono regular lado 4", options: ["16", "20", "12", "24"], correct: 1 },
    GeometryQuestion { question: "Área de un círculo radio 3 (π≈3,14)", options: ["28,26", "18,84", "9,42", "31,4"], correct: 0 },
    GeometryQuestion { question: "¿Cuántas diagonales tiene un cuadrado?", options: ["1", "2", "4", "0"], correct: 1 },
    GeometryQuestion { question: "¿Cuántas caras tiene una pirámide cuadrangular?", options: ["4", "5", "6", "3"], correct: 1 },
    GeometryQuestion { question: "¿Cuántos vértices tiene un tetraedro?", options: ["3", "4", "6", "8"], correct: 1 },
    GeometryQuestion { question: "Hipotenusa con catetos 3 y 4", options: ["5", "6", "7", "12"], correct: 0 },
    GeometryQuestion { question: "Perímetro de un hexágono regular lado 5", options: ["25", "30", "20", "15"], correct: 1 },
    GeometryQuestion { question: "Área de un triángulo base 8 altura 5", options: ["40", "20", "13", "26"], correct: 1 },
    GeometryQuestion { question: "¿Cuántos ejes de simetría tiene un cuadrado?", options: ["2", "4", "6", "8"], correct: 1 },
];
const GEOMETRY_EN: [GeometryQuestion; 30] = [
    GeometryQuestion { question: "How many sides does a pentagon have?", options: ["4", "5", "6", "7"], correct: 1 },
    GeometryQuestion { question: "How many sides does a hexagon have?", options: ["5", "6", "7", "8"], correct: 1 },
    GeometryQuestion { question: "How many sides does an octagon have?", options: ["6", "7", "8", "9"], correct: 2 },
    GeometryQuestion { question: "Perimeter of a square side 6", options: ["12", "18", "24", "36"], correct: 2 },
    GeometryQuestion { question: "Area of a 5×8 rectangle", options: ["13", "26", "40", "45"], correct: 2 },
    GeometryQuestion { question: "How many vertices does a cube have?", options: ["6", "8", "12", "4"], correct: 1 },
    GeometryQuestion { question: "How many faces does a cube have?", options: ["4", "6", "8", "12"], correct: 1 },
    GeometryQuestion { question: "A right angle is", options: ["45°", "90°", "180°", "360°"], correct: 1 },
    GeometryQuestion { question: "Sum of angles in a triangle", options: ["90°", "180°", "270°", "360°"], correct: 1 },
    GeometryQuestion { question: "Sum of angles in a square", options: ["180°", "270°", "360°", "90°"], correct: 2 },
    GeometryQuestion { question: "Perimeter of an equilateral triangle side 7", options: ["14", "21", "28", "49"], correct: 1 },
    GeometryQuestion { question: "Area of a triangle base 10 height 6", options: ["60", "30", "15", "45"], correct: 1 },
    GeometryQuestion { question: "How many degrees in a straight angle?", options: ["90°", "180°", "270°", "360°"], correct: 1 },
    GeometryQuestion { question: "How many sides does a decagon have?", options: ["8", "9", "10", "12"], correct: 2 },
    GeometryQuestion { question: "Perimeter of a 7×3 rectangle", options: ["10", "20", "21", "14"], correct: 1 },
    GeometryQuestion { question: "Area of a square side 9", options: ["36", "81", "18", "72"], correct: 1 },
    GeometryQuestion { question: "How many edges does a cube have?", options: ["6", "8", "12", "10"], correct: 2 },
    GeometryQuestion { question: "How many vertices does a triangle have?", options: ["2", "3", "4", "6"], correct: 1 },
    GeometryQuestion { question: "An acute angle is", options: ["<90°", "90°", ">90°", "180°"], correct: 0 },
    GeometryQuestion { question: "An obtuse angle is", options: ["<90°", "90°", ">90° and <180°", "180°"], correct: 2 },
    GeometryQuestion { question: "How many sides does a heptagon have?", options: ["6", "7", "8", "9"], correct: 1 },
    GeometryQuestion { question: "Perimeter of a regular pentagon side 4", options: ["16", "20", "12", "24"], correct: 1 },
    GeometryQuestion { question: "Area of a circle radius 3 (π≈3.14)", options: ["28.26", "18.84", "9.42", "31.4"], correct: 0 },
    GeometryQuestion { question: "How many diagonals does a square have?", options: ["1", "2", "4", "0"], correct: 1 },
    GeometryQuestion { question: "How many faces does a square pyramid have?", options: ["4", "5", "6", "3"], correct: 1 },
    GeometryQuestion { question: "How many vertices does a tetrahedron have?", options: ["3", "4", "6", "8"], correct: 1 },
    GeometryQuestion { question: "Hypotenuse with legs 3 and 4", options: ["5", "6", "7", "12"], correct: 0 },
    GeometryQuestion { question: "Perimeter of a regular hexagon side 5", options: ["25", "30", "20", "15"], correct: 1 },
    GeometryQuestion { question: "Area of a triangle base 8 height 5", options: ["40", "20", "13", "26"], correct: 1 },
    GeometryQuestion { question: "How many axes of symmetry does a square have?", options: ["2", "4", "6", "8"], correct: 1 },
];
const GEOMETRY_FR: [GeometryQuestion; 30] = [
    GeometryQuestion { question: "Combien de côtés a un pentagone ?", options: ["4", "5", "6", "7"], correct: 1 },
    GeometryQuestion { question: "Combien de côtés a un hexagone ?", options: ["5", "6", "7", "8"], correct: 1 },
    GeometryQuestion { question: "Combien de côtés a un octogone ?", options: ["6", "7", "8", "9"], correct: 2 },
    GeometryQuestion { question: "Périmètre d'un carré côté 6", options: ["12", "18", "24", "36"], correct: 2 },
    GeometryQuestion { question: "Aire d'un rectangle 5×8", options: ["13", "26", "40", "45"], correct: 2 },
    GeometryQuestion { question: "Combien de sommets a un cube ?", options: ["6", "8", "12", "4"], correct: 1 },
    GeometryQuestion { question: "Combien de faces a un cube ?", options: ["4", "6", "8", "12"], correct: 1 },
    GeometryQuestion { question: "Un angle droit mesure", options: ["45°", "90°", "180°", "360°"], correct: 1 },
    GeometryQuestion { question: "Somme des angles d'un triangle", options: ["90°", "180°", "270°", "360°"], correct: 1 },
    GeometryQuestion { question: "Somme des angles d'un carré", options: ["180°", "270°", "360°", "90°"], correct: 2 },
    GeometryQuestion { question: "Périmètre d'un triangle équilatéral côté 7", options: ["14", "21", "28", "49"], correct: 1 },
    GeometryQuestion { question: "Aire d'un triangle base 10 hauteur 6", options: ["60", "30", "15", "45"], correct: 1 },
    GeometryQuestion { question: "Combien de degrés dans un angle plat ?", options: ["90°", "180°", "270°", "360°"], correct: 1 },
    GeometryQuestion { question: "Combien de côtés a un décagone ?", options: ["8", "9", "10", "12"], correct: 2 },
    GeometryQuestion { question: "Périmètre d'un rectangle 7×3", options: ["10", "20", "21", "14"], correct: 1 },
    GeometryQuestion { question: "Aire d'un carré côté 9", options: ["36", "81", "18", "72"], correct: 1 },
    GeometryQuestion { question: "Combien d'arêtes a un cube ?", options: ["6", "8", "12", "10"], correct: 2 },
    GeometryQuestion { question: "Combien de sommets a un triangle ?", options: ["2", "3", "4", "6"], correct: 1 },
    GeometryQuestion { question: "Un angle aigu est", options: ["<90°", "90°", ">90°", "180°"], correct: 0 },
    GeometryQuestion { question: "Un angle obtus est", options: ["<90°", "90°", ">90° et <180°", "180°"], correct: 2 },
    GeometryQuestion { question: "Combien de côtés a un heptagone ?", options: ["6", "7", "8", "9"], correct: 1 },
    GeometryQuestion { question: "Périmètre d'un pentagone régulier côté 4", options: ["16", "20", "12", "24"], correct: 1 },
    GeometryQuestion { question: "Aire d'un cercle rayon 3 (π≈3,14)", options: ["28,26", "18,84", "9,42", "31,4"], correct: 0 },
    GeometryQuestion { question: "Combien de diagonales a un carré ?", options: ["1", "2", "4", "0"], correct: 1 },
    GeometryQuestion { question: "Combien de faces a une pyramide carrée ?", options: ["4", "5", "6", "3"], correct: 1 },
    GeometryQuestion { question: "Combien de sommets a un tétraèdre ?", options: ["3", "4", "6", "8"], correct: 1 },
    GeometryQuestion { question: "Hypoténuse avec côtés 3 et 4", options: ["5", "6", "7", "12"], correct: 0 },
    GeometryQuestion { question: "Périmètre d'un hexagone régulier côté 5", options: ["25", "30", "20", "15"], correct: 1 },
    GeometryQuestion { question: "Aire d'un triangle base 8 hauteur 5", options: ["40", "20", "13", "26"], correct: 1 },
    GeometryQuestion { question: "Combien d'axes de symétrie a un carré ?", options: ["2", "4", "6", "8"], correct: 1 },
];
