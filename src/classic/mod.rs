//! Sección Multijuegos / Juegos Clásicos — hub y juegos de reunión.
//!
//! - Hub (`ClassicMenu`) con 3 juegos: Tres en Raya, Conecta 4, Hundir la Flota.
//! - Cada juego es un overlay de UI a pantalla completa (como `learning`).

pub mod animal;
pub mod battleship;
pub mod bingo;
pub mod checkers;
pub mod cifras_letras;
pub mod connect4;
pub mod dice_anim;
pub mod differences;
pub mod labyrinth;
pub mod minesweeper;
pub mod oca;
pub mod painting;
pub mod parchis;
pub mod puzzle15;
pub mod roulette;
pub mod snake;
pub mod sudoku;
pub mod tangram;
pub mod tictactoe;
pub mod wordsearch;

use bevy::prelude::*;

use crate::game::GameState;
use crate::learning::{screen_background, spawn_button};
use crate::i18n::tr;

/// Plugin de la sección clásicos.
pub struct ClassicPlugin;

impl Plugin for ClassicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            tictactoe::TicTacToePlugin,
            connect4::Connect4Plugin,
            battleship::BattleshipPlugin,
            oca::OcaPlugin,
            parchis::ParchisPlugin,
            roulette::RoulettePlugin,
            cifras_letras::CountdownPlugin,
        ));
        app.add_plugins((
            minesweeper::MinesweeperPlugin,
            snake::SnakePlugin,
            checkers::CheckersPlugin,
            bingo::BingoPlugin,
            animal::AnimalPlugin,
            painting::PaintingPlugin,
            labyrinth::LabyrinthPlugin,
            sudoku::SudokuPlugin,
            wordsearch::WordSearchPlugin,
            tangram::TangramPlugin,
            puzzle15::Puzzle15Plugin,
            differences::DifferencesPlugin,
        ))
        .add_systems(
            Update,
            (
                dice_anim::animate_dice_system,
                dice_anim::dice_text_update_system,
                dice_anim::animate_bingo_system,
                dice_anim::update_bingo_history_system,
            ),
        )
        .add_systems(OnEnter(GameState::ClassicMenu), spawn_classic_menu)
        .add_systems(OnExit(GameState::ClassicMenu), despawn_classic_menu)
        .add_systems(Update, classic_menu_input.run_if(in_state(GameState::ClassicMenu)));
    }
}

#[derive(Component)]
pub struct ClassicMenuUi;
#[derive(Component)]
pub struct TicTacToeButton;
#[derive(Component)]
pub struct Connect4Button;
#[derive(Component)]
pub struct BattleshipButton;
#[derive(Component)]
pub struct OcaButton;
#[derive(Component)]
pub struct ParchisButton;
#[derive(Component)]
pub struct RouletteButton;
#[derive(Component)]
pub struct CountdownButton;
#[derive(Component)]
pub struct MinesweeperButton;
#[derive(Component)]
pub struct SnakeButton;
#[derive(Component)]
pub struct CheckersButton;
#[derive(Component)]
pub struct BingoButton;
#[derive(Component)]
pub struct AnimalButton;
#[derive(Component)]
pub struct PaintingButton;
#[derive(Component)]
pub struct LabyrinthButton;
#[derive(Component)]
pub struct SudokuButton;
#[derive(Component)]
pub struct WordSearchButton;
#[allow(dead_code)]
#[derive(Component)]
pub struct TangramButton;
#[allow(dead_code)]
#[derive(Component)]
pub struct Puzzle15Button;
#[allow(dead_code)]
#[derive(Component)]
pub struct DifferencesButton;
#[derive(Component)]
pub struct ClassicBackButton;

#[derive(Component)]
pub struct ClassicMenuChoice(pub GameState);

fn spawn_classic_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands
        .spawn((
            ClassicMenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            screen_background(),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new(tr("JUEGOS CLÁSICOS")),
                TextFont { font: font.clone(), font_size: 48.0, ..default() },
                TextColor(Color::srgb(1.0, 0.85, 0.40)),
            ));
            root.spawn((
                Text::new(tr("Reúnete y juega — 2 jugadores o contra la CPU")),
                TextFont { font: font.clone(), font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.85, 0.90, 1.0)),
            ));
            root.spawn(Node { height: Val::Px(8.0), ..default() });
            // Grid responsive: 2 columnas en escritorio, 1 en móvil (via flex_wrap)
            root.spawn(Node {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::px(240.0), GridTrack::px(240.0)],
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|grid| {
            grid.spawn((Button, TicTacToeButton, ClassicMenuChoice(GameState::TicTacToeGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Tres en Raya")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, Connect4Button, ClassicMenuChoice(GameState::Connect4Game), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Conecta 4")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, BattleshipButton, ClassicMenuChoice(GameState::BattleshipGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Hundir la Flota (10×10)")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, OcaButton, ClassicMenuChoice(GameState::OcaGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("La Oca")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, ParchisButton, ClassicMenuChoice(GameState::ParchisGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Parchís")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, RouletteButton, ClassicMenuChoice(GameState::RouletteGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Ruleta de la Fortuna")), TextFont { font: font.clone(), font_size: 12.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, CountdownButton, ClassicMenuChoice(GameState::CountdownGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Cifras y Letras")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, MinesweeperButton, ClassicMenuChoice(GameState::MinesweeperGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Buscaminas")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, SnakeButton, ClassicMenuChoice(GameState::SnakeGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Snake")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, CheckersButton, ClassicMenuChoice(GameState::CheckersGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Damas")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, BingoButton, ClassicMenuChoice(GameState::BingoGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Bingo")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, AnimalButton, ClassicMenuChoice(GameState::AnimalGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Adivina el Animal")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, PaintingButton, ClassicMenuChoice(GameState::PaintingGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Pintar punteado")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, LabyrinthButton, ClassicMenuChoice(GameState::LabyrinthGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Laberinto")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, SudokuButton, ClassicMenuChoice(GameState::SudokuGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Sudoku")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, WordSearchButton, ClassicMenuChoice(GameState::WordSearchGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Sopa de Letras")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, TangramButton, ClassicMenuChoice(GameState::TangramGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Tangram")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, Puzzle15Button, ClassicMenuChoice(GameState::Puzzle15Game), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Puzzle 15")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            grid.spawn((Button, DifferencesButton, ClassicMenuChoice(GameState::DifferencesGame), Node { width: Val::Px(240.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20,0.38,0.66)), BorderColor(Color::srgb(0.60,0.80,1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b|{ b.spawn((Text::new(tr("Busca Diferencias")), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE))); });
            });
            root.spawn(Node { height: Val::Px(8.0), ..default() });
            spawn_button(root, "Volver al menú principal", ClassicBackButton, &font);
            root.spawn((
                Text::new(tr("Los dados laterales se animan al tirar — ¡mira los laterales del tablero!")),
                TextFont { font: font.clone(), font_size: 11.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.55)),
            ));
        });
}

fn despawn_classic_menu(mut commands: Commands, roots: Query<Entity, With<ClassicMenuUi>>) {
    for root in &roots {
        commands.entity(root).despawn();
    }
}

fn classic_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    choices: Query<(&Interaction, &ClassicMenuChoice), Changed<Interaction>>,
    back: Query<&Interaction, (Changed<Interaction>, With<ClassicBackButton>)>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
        return;
    }
    if back.single().map_or(false, |i| *i == Interaction::Pressed) { next_state.set(GameState::MainMenu); return; }
    for (interaction, choice) in &choices {
        if *interaction == Interaction::Pressed {
            next_state.set(choice.0);
            return;
        }
    }
}
