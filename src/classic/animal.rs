//! Adivina el Animal — 5 pistas por nivel, a ver en qué nivel lo aciertas.
//!
//! El juego elige un animal al azar y te da pistas de menos a más obvias.
//! Cuanto antes lo aciertes, más puntos. ¡Pon a prueba tu cultura animal!

use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use rand::seq::SliceRandom;

use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

#[derive(Clone, Debug)]
struct Animal {
    name: &'static str,
    clues: [&'static str; 5],
    emoji: &'static str,
}

const ANIMALS: &[Animal] = &[
    Animal { name: "elefante", clues: ["Vivo en la sabana y la selva", "Soy el mamífero terrestre más grande", "Tengo una trompa muy larga", "Mis orejas son como abanicos gigantes", "Soy gris, peso toneladas y dicen que nunca olvido"], emoji: "🐘" },
    Animal { name: "jirafa", clues: ["Vivo en África", "Soy el animal más alto del mundo", "Tengo el cuello larguísimo", "Mi lengua es azul-negra y mide 50cm", "Tengo manchas marrones como piezas de puzzle"], emoji: "🦒" },
    Animal { name: "leon", clues: ["Soy el rey de la selva", "Vivo en manada", "El macho tiene una gran melena", "Rujo tan fuerte que se oye a 8km", "Mi nombre empieza por L"], emoji: "🦁" },
    Animal { name: "delfin", clues: ["Vivo en el mar pero soy mamífero", "Soy muy inteligente y juguetón", "Me comunico con chasquidos y silbidos", "Salto fuera del agua haciendo acrobacias", "Tengo hocico alargado como sonrisa"], emoji: "🐬" },
    Animal { name: "pinguino", clues: ["Vivo donde hace mucho frío", "Soy un ave pero no vuelo", "Nado como un torpedo", "Voy siempre de esmoquin blanco y negro", "Camino balanceándome y me deslizo por el hielo"], emoji: "🐧" },
    Animal { name: "canguro", clues: ["Vivo en Australia", "Llevo a mi cría en una bolsa", "Salto enormes distancias", "Tengo cola muy fuerte para equilibrarme", "Boxeo con mis patas delanteras"], emoji: "🦘" },
    Animal { name: "cocodrilo", clues: ["Vivo en ríos y pantanos", "Soy un reptil muy antiguo", "Tengo mandíbulas potentísimas", "Mis ojos y nariz sobresalen del agua", "Mi piel es acorazada y escamosa"], emoji: "🐊" },
    Animal { name: "oso", clues: ["Vivo en bosques y montañas", "Me gusta la miel", "En invierno hiberno", "Tengo garras grandes y soy muy fuerte", "Mi cría se llama osezno"], emoji: "🐻" },
    Animal { name: "mono", clues: ["Vivo en la selva", "Me encantan los plátanos", "Tengo cola prensil para colgarme", "Soy muy travieso y me balanceo de rama en rama", "Mi grito se oye muy lejos: ¡u-u-a-a!"], emoji: "🐒" },
    Animal { name: "cebra", clues: ["Vivo en la sabana africana", "Parezo un caballo", "Tengo rayas blancas y negras únicas", "Cada ejemplar tiene un patrón distinto", "Nunca dos somos iguales"], emoji: "🦓" },
    Animal { name: "tigre", clues: ["Soy el felino más grande", "Tengo rayas naranjas y negras", "Vivo en la jungla asiática", "Cazo solo y soy muy sigiloso", "Mi rugido impone respeto"], emoji: "🐯" },
    Animal { name: "serpiente", clues: ["No tengo patas", "Me deslizo por el suelo", "Algunas somos venenosas", "Cambio de piel cada cierto tiempo", "Mi lengua es bífida"], emoji: "🐍" },
];

fn normalize(s: &str) -> String {
    s.to_lowercase().trim().chars().filter(|c| c.is_alphanumeric()).collect()
}

#[derive(Resource, Clone)]
struct AnimalSession {
    animals: Vec<Animal>,
    current: usize,
    clue_level: usize, // 0..4 -> 5 niveles, 0 es pista 1
    score: u32,
    total: u32,
    revealed: bool,
    message: String,
    input: String,
    options: Vec<String>,
}

impl AnimalSession {
    fn new_random() -> Self {
        let mut animals = ANIMALS.to_vec();
        animals.shuffle(&mut rand::thread_rng());
        let current = 0;
        let mut session = Self {
            animals,
            current,
            clue_level: 0,
            score: 0,
            total: 0,
            revealed: false,
            message: "¡Adivina con la primera pista!".to_string(),
            input: String::new(),
            options: Vec::new(),
        };
        session.gen_options();
        session
    }
    fn current_animal(&self) -> &Animal { &self.animals[self.current] }
    fn gen_options(&mut self) {
        let correct = self.current_animal().name.to_string();
        let mut pool: Vec<String> = ANIMALS.iter().map(|a| a.name.to_string()).filter(|n| n != &correct).collect();
        pool.shuffle(&mut rand::thread_rng());
        let mut opts = vec![correct.clone()];
        opts.extend(pool.into_iter().take(3));
        opts.shuffle(&mut rand::thread_rng());
        self.options = opts;
    }
    fn guess(&mut self, guess: &str) {
        if self.revealed { return; }
        let g = normalize(guess);
        let target = normalize(self.current_animal().name);
        // también aceptar sin tilde: leon == león
        let target_no_accent = target.replace('ó', "o").replace('á',"a").replace('é',"e").replace('í',"i").replace('ú',"u");
        let g_no_accent = g.replace('ó', "o").replace('á',"a").replace('é',"e").replace('í',"i").replace('ú',"u");
        let correct = g == target || g_no_accent == target_no_accent || g == target_no_accent || g_no_accent == target;
        if correct {
            let points = (5 - self.clue_level as u32).max(1);
            self.score += points;
            self.total += 1;
            self.revealed = true;
            self.message = format!("¡Correcto! Era {} {} — Nivel {}/5 (+{} pts)", self.current_animal().name, self.current_animal().emoji, self.clue_level+1, points);
        } else {
            if self.clue_level < 4 {
                self.clue_level += 1;
                self.message = format!("No es '{}' — Te doy otra pista (nivel {}/5)", guess, self.clue_level+1);
                self.input.clear();
            } else {
                self.revealed = true;
                self.message = format!("¡Era {} {}! No lo adivinaste", self.current_animal().name, self.current_animal().emoji);
            }
        }
    }
    fn next_animal(&mut self) {
        self.current = (self.current + 1) % self.animals.len();
        if self.current == 0 {
            let mut rng = rand::thread_rng();
            self.animals.shuffle(&mut rng);
        }
        self.clue_level = 0;
        self.revealed = false;
        self.input.clear();
        self.message = "¡Nueva ronda! Adivina con la pista 1".to_string();
        self.gen_options();
    }
    fn next_clue(&mut self) {
        if self.revealed { return; }
        if self.clue_level < 4 {
            self.clue_level += 1;
            self.message = format!("Pista {}/5 mostrada", self.clue_level+1);
        }
    }
}

#[derive(Component)]
struct AnimalUiRoot;
#[derive(Component)]
struct AnimalText(AnimalField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum AnimalField { Title, Clue, Counter, Score, Message, Input, Emoji }
#[derive(Component)]
struct AnimalOptionButton(usize);
#[derive(Component)]
struct AnimalGuessButton;
#[derive(Component)]
struct AnimalNextClueButton;
#[derive(Component)]
struct AnimalNextButton;
#[derive(Component)]
struct AnimalBackButton;
#[derive(Component)]
struct AnimalInputField;

pub struct AnimalPlugin;
impl Plugin for AnimalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::AnimalGame), spawn_animal)
            .add_systems(OnExit(GameState::AnimalGame), cleanup_animal)
            .add_systems(Update, (update_animal, animal_typing).chain().run_if(in_state(GameState::AnimalGame)));
    }
}

fn spawn_animal(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(AnimalSession::new_random());
    commands
        .spawn((AnimalUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(12.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30)))
        .with_children(|overlay| {
            overlay.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(720.0), max_width: Val::Percent(96.0), padding: UiRect::axes(Val::Px(20.0), Val::Px(16.0)), row_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((AnimalText(AnimalField::Title), Text::new("ADIVINA EL ANIMAL"), TextFont { font: font.clone(), font_size: 28.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn((AnimalText(AnimalField::Emoji), Text::new("❓"), TextFont { font: font.clone(), font_size: 64.0, ..default() }, TextColor(Color::WHITE)));
                panel.spawn((AnimalText(AnimalField::Score), Text::new("Puntos: 0"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
                panel.spawn((AnimalText(AnimalField::Counter), Text::new("Pista 1/5"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::srgb(0.70, 0.80, 0.95))));
                panel.spawn((Node { width: Val::Px(680.0), min_height: Val::Px(72.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(12.0)), ..default() }, BackgroundColor(Color::srgba(0.12, 0.18, 0.30, 0.9)), BorderRadius::all(Val::Px(12.0)))).with_children(|boxp| {
                    boxp.spawn((AnimalText(AnimalField::Clue), Text::new(""), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(640.0), ..default() }));
                });
                // Opciones (4 botones)
                panel.spawn(Node { display: Display::Grid, grid_template_columns: vec![GridTrack::px(160.0), GridTrack::px(160.0)], grid_template_rows: vec![GridTrack::px(44.0), GridTrack::px(44.0)], column_gap: Val::Px(10.0), row_gap: Val::Px(10.0), ..default() }).with_children(|grid| {
                    for i in 0..4 {
                        grid.spawn((Button, AnimalOptionButton(i), Node { width: Val::Px(160.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.22, 0.27, 0.38)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("".to_string()), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    }
                });
                // Input de texto libre
                panel.spawn((AnimalInputField, Node { width: Val::Px(420.0), height: Val::Px(42.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)), ..default() }, BackgroundColor(Color::srgb(0.12, 0.14, 0.24)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(8.0)))).with_children(|input| {
                    input.spawn((AnimalText(AnimalField::Input), Text::new("Escribe aquí..."), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::srgba(1.0,1.0,1.0,0.55))));
                });
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), flex_wrap: FlexWrap::Wrap, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
                    row.spawn((Button, AnimalGuessButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Adivinar")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, AnimalNextClueButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.30, 0.35, 0.55)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("Otra pista"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, AnimalNextButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("Siguiente animal"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
                });
                panel.spawn((AnimalText(AnimalField::Message), Text::new(""), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::srgb(1.0, 0.90, 0.50)), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(680.0), ..default() }));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                    row.spawn((Button, AnimalBackButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
}

fn cleanup_animal(mut commands: Commands, roots: Query<Entity, With<AnimalUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<AnimalSession>();
}

fn animal_typing(
    mut events: EventReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<AnimalSession>,
) {
    if session.revealed { return; }
    let mut typed = String::new();
    for ev in events.read() {
        if ev.state != ButtonState::Pressed { continue; }
        if let Some(t) = &ev.text {
            for ch in t.chars() { if !ch.is_control() { typed.push(ch); } }
        }
    }
    if keys.just_pressed(KeyCode::Backspace) { session.input.pop(); }
    if !typed.is_empty() { session.input.push_str(&typed); }
    if keys.just_pressed(KeyCode::Enter) && !session.input.trim().is_empty() {
        let g = session.input.clone();
        session.input.clear();
        session.guess(&g);
    }
}

fn update_animal(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<AnimalSession>,
    option_clicks: Query<(&Interaction, &AnimalOptionButton), Changed<Interaction>>,
    guess_click: Query<&Interaction, (Changed<Interaction>, With<AnimalGuessButton>)>,
    next_clue_click: Query<&Interaction, (Changed<Interaction>, With<AnimalNextClueButton>)>,
    next_click: Query<&Interaction, (Changed<Interaction>, With<AnimalNextButton>)>,
    back_click: Query<&Interaction, (Changed<Interaction>, With<AnimalBackButton>)>,
    mut texts: Query<(&AnimalText, &mut Text, &mut TextColor)>,
    mut option_buttons: Query<(&AnimalOptionButton, &Children)>,
    mut option_texts: Query<&mut Text, Without<AnimalText>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_click.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    for (interaction, btn) in &option_clicks {
        if *interaction == Interaction::Pressed {
            let guess = session.options.get(btn.0).cloned().unwrap_or_default();
            session.guess(&guess);
        }
    }
    for interaction in &guess_click { if *interaction == Interaction::Pressed && !session.input.trim().is_empty() { let g = session.input.clone(); session.input.clear(); session.guess(&g); } }
    for interaction in &next_clue_click { if *interaction == Interaction::Pressed { session.next_clue(); } }
    for interaction in &next_click { if *interaction == Interaction::Pressed { session.next_animal(); } }
    if keys.just_pressed(KeyCode::KeyN) { session.next_animal(); }

    for (field, mut text, mut color) in &mut texts {
        match field.0 {
            AnimalField::Clue => {
                let clue = session.current_animal().clues[session.clue_level];
                *text = Text::new(format!("Pista {}/5: {}", session.clue_level+1, clue));
            },
            AnimalField::Counter => {
                *text = Text::new(format!("Animal {}/{} — Nivel {}/5", session.total+1, ANIMALS.len(), session.clue_level+1));
            },
            AnimalField::Score => {
                *text = Text::new(format!("Puntos: {} — Acertados: {}/{} — Nivel medio: {:.1}", session.score, session.total, ANIMALS.len(), if session.total==0 {0.0} else { session.score as f32 / session.total as f32 * 1.0 }));
                // Mostrar nivel medio invertido: puntos 5..1, nivel 1..5, puntos = 6 - nivel
            },
            AnimalField::Message => {
                *text = Text::new(session.message.clone());
                *color = TextColor(if session.revealed && session.message.contains("Correcto") { Color::srgb(0.40, 0.90, 0.50) } else if session.revealed { Color::srgb(0.95, 0.40, 0.40) } else { Color::srgb(1.0, 0.90, 0.50) });
            },
            AnimalField::Input => {
                let shown = if session.input.is_empty() { "Escribe aquí...".to_string() } else { format!("{}▌", session.input) };
                *text = Text::new(shown);
                *color = TextColor(if session.input.is_empty() { Color::srgba(1.0,1.0,1.0,0.5) } else { Color::WHITE });
            },
            AnimalField::Emoji => {
                *text = Text::new(if session.revealed { session.current_animal().emoji } else { "❓" }.to_string());
            },
            _ => {}
        }
    }
    for (btn, children) in &mut option_buttons {
        let label = session.options.get(btn.0).cloned().unwrap_or_default();
        for child in children.iter() {
            if let Ok(mut text) = option_texts.get_mut(child) {
                *text = Text::new(label.clone());
            }
        }
    }
}
