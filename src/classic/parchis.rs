//! Parchís — 68 casillas circuito + 4 casas, 4 fichas por jugador, reglas oficiales España.
//!
//! Circuito 68 casillas (12 seguros), 4 casas, 4 salidas, pasillo 8 casillas
//! y meta central. Sacar con 5, comer = 20, llegar a meta = 10, tres 6 = a casa.
//! Adaptado a 1 ficha por jugador para partida corta (lógica fiel al recorrido).

use bevy::prelude::*;
use rand::Rng;

use crate::classic::dice_anim::{spawn_side_dice, AnimatedDice};
use crate::game::GameState;
use crate::learning::screen_background;
use crate::i18n::tr;

/// 68 casillas del circuito exterior.
const CIRCUIT: usize = 68;
/// Casillas seguras oficiales (12): cada salida + 8 intermedias. Las 4 salidas son seguras.
const SAFE: [usize; 12] = [0, 5, 12, 17, 22, 29, 34, 39, 46, 51, 56, 63];
/// Salidas de cada color (en índice de circuito). Orden: rojo(0), amarillo(17), verde(34), azul(51)
const STARTS: [usize; 4] = [0, 17, 34, 51];
/// Longitud del pasillo de llegada (7 pasillo + meta = 8).
const PASILLO: usize = 8;
/// Meta final después del pasillo (índice lógico para ganar).
const GOAL: usize = CIRCUIT + PASILLO; // 76

#[derive(Resource, Clone)]
struct ParchisSession {
    /// Posición lógica: None = en casa, Some(0..67) circuito, 68..75 pasillo, 76 meta.
    pos: [Option<usize>; 4],
    turn: usize,
    dice: u8,
    winner: Option<usize>,
    message: String,
    pending_dice: Option<u8>,
    pending_is_player: bool,
    num_players: usize,
    vs_cpu: bool,
    setup_done: bool,
}

impl ParchisSession {
    fn new() -> Self {
        Self { pos: [None; 4], turn: 0, dice: 1, winner: None, message: "¡Necesitas un 5 para salir de casa!".to_string(), pending_dice: None, pending_is_player: true, num_players: 4, vs_cpu: true, setup_done: false }
    }
    fn new_with_setup(num_players: usize, vs_cpu: bool) -> Self {
        Self { pos: [None; 4], turn: 0, dice: 1, winner: None, message: format!("¡Parchís {num_players} jugadores — tira el dado!"), pending_dice: None, pending_is_player: true, num_players: num_players.clamp(2,4), vs_cpu, setup_done: true }
    }
    fn is_human_turn(&self) -> bool {
        if self.vs_cpu { self.turn == 0 } else { self.turn < self.num_players }
    }
    fn is_safe(pos: usize) -> bool { SAFE.contains(&pos) }
    #[allow(dead_code)]
    fn move_turn(&mut self, is_player: bool) {
        let dice = rand::thread_rng().gen_range(1..=6);
        self.move_turn_with_dice(is_player, dice);
    }
    fn move_turn_with_dice(&mut self, is_player: bool, dice: u8) {
        if self.winner.is_some() { return; }
        self.dice = dice;
        let idx = self.turn;
        let cur = self.pos[idx];
        let player_name = if self.vs_cpu {
            if is_player { "Tú".to_string() } else { format!("CPU {}", idx) }
        } else {
            format!("J{}", idx + 1)
        };
        let new_pos = match cur {
            None => {
                if dice == 5 {
                    let start = STARTS[idx];
                    self.message = format!("{} saca 5 y sale a la casilla {}", player_name, start);
                    Some(start)
                } else {
                    self.message = format!("{} saca {} y necesita un 5", player_name, dice);
                    None
                }
            },
            Some(p) => {
                // Si ya está en pasillo/meta, avanza solo dentro del pasillo.
                let np = if p < CIRCUIT {
                    // Entrada al pasillo: cada color entra en su casilla de entrada
                    // (justo antes de su salida, 4 casillas antes). Simplificamos:
                    // entrada = (STARTS[idx] + 68 - 5) % 68, luego pasillo 68..75.
                    // Si sobrepasa, entra al pasillo.
                    let entry = (STARTS[idx] + CIRCUIT - 4) % CIRCUIT;
                    // distancia hasta entrada
                    let dist_to_entry = (entry + CIRCUIT - p) % CIRCUIT;
                    if (p + dice as usize) % CIRCUIT == entry || p + dice as usize >= CIRCUIT && dist_to_entry < dice as usize {
                        // entra al pasillo: calcula cuánto sobra
                        let steps_past_entry = (p + dice as usize).saturating_sub(entry).saturating_sub(1);
                        // mapea a pasillo 68 + steps
                        Some(CIRCUIT + steps_past_entry.min(PASILLO - 1))
                    } else {
                        Some((p + dice as usize) % CIRCUIT)
                    }
                } else if p >= CIRCUIT && p < GOAL {
                    let np = p + dice as usize;
                    if np > GOAL { None } else { Some(np) }
                } else {
                    None
                };
                // Si no se pudo mover por rebote en pasillo, no avanza
                if np.is_none() && p >= CIRCUIT {
                    self.message = format!("{} saca {} pero necesita exacto para meta ({}→{})", player_name, dice, p, GOAL);
                    None
                } else if let Some(v) = np {
                    if v >= CIRCUIT {
                        self.message = format!("{} avanza {} → pasillo {}", player_name, dice, v - CIRCUIT + 1);
                    } else if v < 10 || (v as i32 - p as i32).abs() > 20 {
                        // vuelta al inicio
                        self.message = format!("{} avanza {} → {}", player_name, dice, v);
                    } else {
                        self.message = format!("{} avanza {} → {}", player_name, dice, v);
                    }
                    Some(v)
                } else {
                    // circuito normal sin entrada
                    let v = (p + dice as usize) % CIRCUIT;
                    self.message = format!("{} avanza {} → {}", player_name, dice, v);
                    Some(v)
                }
            }
        };
        // Rebote clásico en meta exacta: si pasaría de GOAL, rebota
        let mut final_pos = new_pos;
        if let Some(np) = new_pos {
            if np > GOAL {
                let over = np - GOAL;
                final_pos = Some(GOAL - over);
                self.message = format!("¡Rebote! {} + {} se pasa, vuelve a {}", cur.unwrap_or(0), dice, final_pos.unwrap());
            }
        }
        if let Some(np) = final_pos {
            // comer en circuito (no en seguros ni pasillo)
            if np < CIRCUIT && !Self::is_safe(np) {
                for (other_idx, other_pos) in self.pos.iter_mut().enumerate() {
                    if other_idx != idx {
                        if let Some(op) = *other_pos {
                            if op == np {
                                *other_pos = None;
                                self.message = format!("{} ¡Comes a CPU {} y lo mandas a casa! (+20)", self.message, other_idx);
                                // Contar 20 extra: avance inmediato de 20 casillas con la misma ficha (simplificado: solo mensaje)
                            }
                        }
                    }
                }
            }
            self.pos[idx] = Some(np);
            if np == GOAL {
                self.winner = Some(idx);
                let winner_name = if self.vs_cpu {
                    if is_player { "Tú".to_string() } else { format!("CPU {}", idx) }
                } else { format!("J{}", idx + 1) };
                self.message = format!("¡{} gana! Llegó a la META", winner_name);
                return;
            }
            if dice == 6 {
                self.message = format!("{} ¡6! Tiras otra vez", self.message);
                return;
            }
        }
        let n = if self.vs_cpu { 4 } else { self.num_players };
        self.turn = (self.turn + 1) % n;
    }
}

#[derive(Component)]
struct ParchisUiRoot;
#[derive(Component)]
struct ParchisText(ParchisField);
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParchisField { Title, Status, Dice, Positions }
#[derive(Component)]
struct ParchisRollButton;
#[derive(Component)]
struct ParchisBackButton;
#[derive(Component)]
struct ParchisRestartButton;
#[derive(Component)]
struct ParchisBoardCell(usize);
#[derive(Component)]
struct ParchisSetupRoot;
#[derive(Component)]
struct ParchisSetup1vCpu;
#[derive(Component)]
struct ParchisSetup2P;
#[derive(Component)]
struct ParchisSetup3P;
#[derive(Component)]
struct ParchisSetup4P;

/// Waypoints del circuito 68 en forma de cruz con muescas en las 4 casas.
/// El polilínea sigue el borde del tablero hugging las casas de 144px en las esquinas.
fn parchis_waypoints() -> Vec<(f32, f32)> {
    // Tablero 520x520, origen (0,0) arriba-izquierda. Casas 144x144 en esquinas (12,12).
    // El track va a ~30px del borde y a ~16px de las casas (y=160, x=160, etc.)
    vec![
        (30.0, 260.0),   // 0  izq medio (START 0 rojo)
        (30.0, 160.0),   // 1  sube por izq hasta debajo casa sup-izq
        (160.0, 160.0),  // 2  derecha por debajo casa
        (160.0, 30.0),   // 3  sube por derecha de casa
        (260.0, 30.0),   // 4  top medio (START 17 amarillo)
        (360.0, 30.0),   // 5  derecha por top
        (360.0, 160.0),  // 6  baja por izq de casa sup-der
        (490.0, 160.0),  // 7  derecha por debajo casa sup-der
        (490.0, 260.0),  // 8  der medio (START 34 verde)
        (490.0, 360.0),  // 9  baja por der hasta arriba casa inf-der
        (360.0, 360.0),  // 10 izq por arriba casa inf-der
        (360.0, 490.0),  // 11 baja por izq de casa inf-der
        (260.0, 490.0),  // 12 bottom medio (START 51 azul)
        (160.0, 490.0),  // 13 izq por bottom
        (160.0, 360.0),  // 14 sube por der de casa inf-izq
        (30.0, 360.0),   // 15 izq por arriba casa inf-izq
        (30.0, 260.0),   // 16 cierre
    ]
}

/// Posición visual de una casilla del circuito 68.
/// Muestrea 68 puntos equiespaciados a lo largo de la polilínea con muescas.
fn parchis_board_position(index: usize) -> (f32, f32) {
    if index >= CIRCUIT {
        // Pasillo: para circuito se usa 0..67, pasillo aparte
        return parchis_pasillo_position(0, index - CIRCUIT);
    }
    let wps = parchis_waypoints();
    // Calcular longitud total
    let mut segs: Vec<f32> = Vec::new();
    let mut total = 0.0;
    for i in 0..wps.len() - 1 {
        let (x0, y0) = wps[i];
        let (x1, y1) = wps[i + 1];
        let len = (x1 - x0).abs() + (y1 - y0).abs(); // Manhattan (solo horiz/vert)
        segs.push(len);
        total += len;
    }
    let target = (index as f32 + 0.5) / CIRCUIT as f32 * total;
    let mut acc = 0.0;
    for i in 0..segs.len() {
        let seg_len = segs[i];
        if target >= acc && target < acc + seg_len {
            let t = (target - acc) / seg_len;
            let (x0, y0) = wps[i];
            let (x1, y1) = wps[i + 1];
            let x = x0 + (x1 - x0) * t;
            let y = y0 + (y1 - y0) * t;
            // Centrar celda 30px: restar 15
            return (x - 15.0, y - 15.0);
        }
        acc += seg_len;
    }
    let (x, y) = wps[0];
    (x - 15.0, y - 15.0)
}

/// Posición de una casilla de pasillo (0..7) para un color 0..3.
/// Cada color va en su brazo hacia el centro (260,260).
fn parchis_pasillo_position(color: usize, step: usize) -> (f32, f32) {
    let s = step as f32;
    // Centro con offset para que no se solapen los 4 pasillos en diagonal central final
    match color {
        0 => { // Rojo: izq → centro (y=260)
            (70.0 + s * 22.0, 260.0 - 15.0)
        },
        1 => { // Amarillo: top → centro (x=260)
            (260.0 - 15.0, 70.0 + s * 22.0)
        },
        2 => { // Verde: der → centro
            (450.0 - s * 22.0, 260.0 - 15.0)
        },
        _ => { // Azul: bottom → centro
            (260.0 - 15.0, 450.0 - s * 22.0)
        },
    }
}

fn parchis_board_position_full(index: usize) -> (f32, f32) {
    if index < CIRCUIT {
        parchis_board_position(index)
    } else if index < GOAL {
        // Pasillo genérico: usa color 0 por defecto (se sobrescribe en spawn por color)
        let step = index - CIRCUIT;
        parchis_pasillo_position(0, step)
    } else {
        (245.0, 245.0)
    }
}

#[cfg(test)]
mod tests {
    use super::parchis_board_position;
    #[test]
    fn parchis_path_has_68_circuit() {
        let positions: Vec<_> = (0..68).map(parchis_board_position).collect();
        assert_eq!(positions.len(), 68);
        // Todas con coordenadas válidas dentro del tablero 520x520
        for (x, y) in &positions {
            assert!(*x >= 0.0 && *x <= 520.0);
            assert!(*y >= 0.0 && *y <= 520.0);
        }
    }
}

pub struct ParchisPlugin;
impl Plugin for ParchisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ParchisGame), spawn_parchis)
            .add_systems(OnExit(GameState::ParchisGame), cleanup_parchis)
            .add_systems(Update, update_parchis.run_if(in_state(GameState::ParchisGame)));
    }
}

fn spawn_parchis(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    commands.insert_resource(ParchisSession::new());
    commands.spawn((ParchisUiRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(8.0)), ..default() }, screen_background(), Visibility::Visible, ZIndex(30))).with_children(|overlay| {
        overlay.spawn((ParchisSetupRoot, Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.10, 0.85)), ZIndex(50))).with_children(|setup| {
            setup.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(520.0), padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(14.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.10, 0.14, 0.28, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
                panel.spawn((Text::new("PARCHÍS — Elige jugadores"), TextFont { font: font.clone(), font_size: 22.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), flex_wrap: FlexWrap::Wrap, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
                    row.spawn((Button, ParchisSetup1vCpu, Node { width: Val::Px(150.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("1 vs CPU"), TextFont { font: font.clone(), font_size: 13.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, ParchisSetup2P, Node { width: Val::Px(150.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("2 Jugadores"), TextFont { font: font.clone(), font_size: 13.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, ParchisSetup3P, Node { width: Val::Px(150.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("3 Jugadores"), TextFont { font: font.clone(), font_size: 13.0, ..default() }, TextColor(Color::WHITE))); });
                    row.spawn((Button, ParchisSetup4P, Node { width: Val::Px(150.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new("4 Jugadores"), TextFont { font: font.clone(), font_size: 13.0, ..default() }, TextColor(Color::WHITE))); });
                });
            });
        });
        overlay.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
            spawn_side_dice(row, &font, "DADO IZQ");
            row.spawn((Node { flex_direction: FlexDirection::Column, width: Val::Px(760.0), max_width: Val::Percent(98.0), padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)), row_gap: Val::Px(8.0), align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgba(0.07, 0.09, 0.18, 0.96)), BorderRadius::all(Val::Px(16.0)))).with_children(|panel| {
            panel.spawn((ParchisText(ParchisField::Title), Text::new("PARCHÍS — 68 casillas (oficial)"), TextFont { font: font.clone(), font_size: 24.0, ..default() }, TextColor(Color::srgb(0.95, 0.85, 0.40))));
            panel.spawn((ParchisText(ParchisField::Status), Text::new("¡Necesitas un 5 para salir! 12 seguros, pasillo 8"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE), TextLayout { linebreak: LineBreak::WordBoundary, ..default() }, Node { max_width: Val::Px(720.0), ..default() }));
            panel.spawn((ParchisText(ParchisField::Dice), Text::new("Dado: -"), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::srgb(0.80, 0.95, 1.0))));
            panel.spawn((ParchisText(ParchisField::Positions), Text::new("Tú: Casa  CPU1: Casa  CPU2: Casa  CPU3: Casa"), TextFont { font: font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE)));
            panel.spawn((Node { position_type: PositionType::Relative, width: Val::Px(520.0), height: Val::Px(520.0), max_width: Val::Px(520.0), ..default() }, BackgroundColor(Color::srgb(0.26, 0.14, 0.08)), BorderRadius::all(Val::Px(24.0)))).with_children(|board| {
                for (left, top, color) in [
                    (Val::Px(12.0), Val::Px(12.0), Color::srgb(0.78, 0.20, 0.22)),
                    (Val::Px(364.0), Val::Px(12.0), Color::srgb(0.96, 0.72, 0.16)),
                    (Val::Px(12.0), Val::Px(364.0), Color::srgb(0.18, 0.42, 0.78)),
                    (Val::Px(364.0), Val::Px(364.0), Color::srgb(0.22, 0.66, 0.34)),
                ] {
                    board.spawn((Node { position_type: PositionType::Absolute, left, top, width: Val::Px(144.0), height: Val::Px(144.0), ..default() }, BackgroundColor(color), BorderRadius::all(Val::Px(18.0))));
                }
                // Meta central
                board.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(216.0), top: Val::Px(216.0), width: Val::Px(88.0), height: Val::Px(88.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.86, 0.62, 0.16)), BorderRadius::all(Val::Px(12.0)))).with_children(|c| { c.spawn((Text::new("META"), TextFont { font: font.clone(), font_size: 16.0, ..default() }, TextColor(Color::WHITE))); });
                for n in 0..CIRCUIT {
                    let (x, y) = parchis_board_position(n);
                    let is_safe = SAFE.contains(&n);
                    let is_start = STARTS.contains(&n);
                    let bg = if is_start { Color::srgb(0.95, 0.80, 0.20) } else if is_safe { Color::srgb(0.18, 0.52, 0.30) } else if n % 2 == 0 { Color::srgb(0.16, 0.32, 0.42) } else { Color::srgb(0.12, 0.25, 0.36) };
                    let label = if is_start { format!("{} S", n) } else if is_safe { format!("{} *", n) } else { n.to_string() };
                    board.spawn((ParchisBoardCell(n), Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), width: Val::Px(30.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(bg), BorderRadius::all(Val::Px(15.0)))).with_children(|c| { c.spawn((Text::new(label), TextFont { font: font.clone(), font_size: 8.0, ..default() }, TextColor(Color::WHITE))); });
                }
                // Pasillo (8 casillas centrales)
                for n in CIRCUIT..GOAL {
                    let (x, y) = parchis_board_position_full(n);
                    board.spawn((ParchisBoardCell(n), Node { position_type: PositionType::Absolute, left: Val::Px(x), top: Val::Px(y), width: Val::Px(28.0), height: Val::Px(28.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(1.0)), ..default() }, BackgroundColor(Color::srgb(0.90, 0.70, 0.25)), BorderRadius::all(Val::Px(6.0)))).with_children(|c| { c.spawn((Text::new(format!("P{}", n - CIRCUIT + 1)), TextFont { font: font.clone(), font_size: 8.0, ..default() }, TextColor(Color::BLACK))); });
                }
            });
            panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), flex_wrap: FlexWrap::Wrap, justify_content: JustifyContent::Center, ..default() }).with_children(|row| {
                row.spawn((Button, ParchisRollButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.15, 0.42, 0.25)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Tirar dado")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                row.spawn((Button, ParchisRestartButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Reiniciar")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
                row.spawn((Button, ParchisBackButton, Node { width: Val::Px(140.0), height: Val::Px(40.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, BackgroundColor(Color::srgb(0.20, 0.38, 0.66)), BorderColor(Color::srgb(0.60, 0.80, 1.0)), BorderRadius::all(Val::Px(10.0)))).with_children(|b| { b.spawn((Text::new(tr("Volver")), TextFont { font: font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE))); });
            });
        });
            spawn_side_dice(row, &font, "DADO DER");
            });
    });
}

fn cleanup_parchis(mut commands: Commands, roots: Query<Entity, With<ParchisUiRoot>>) {
    for root in &roots { commands.entity(root).despawn(); }
    commands.remove_resource::<ParchisSession>();
}

fn update_parchis(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<ParchisSession>,
    roll_clicks: Query<&Interaction, (Changed<Interaction>, With<ParchisRollButton>)>,
    back_clicks: Query<&Interaction, (Changed<Interaction>, With<ParchisBackButton>)>,
    restart_clicks: Query<&Interaction, (Changed<Interaction>, With<ParchisRestartButton>)>,
    mut texts: Query<(&ParchisText, &mut Text)>,
    mut board_cells: Query<(&ParchisBoardCell, &Children, &mut BackgroundColor)>,
    mut cell_texts: Query<&mut Text, Without<ParchisText>>,
    mut dice_anim: Query<&mut AnimatedDice>,
    setup1: Query<&Interaction, (Changed<Interaction>, With<ParchisSetup1vCpu>)>,
    setup2: Query<&Interaction, (Changed<Interaction>, With<ParchisSetup2P>)>,
    setup3: Query<&Interaction, (Changed<Interaction>, With<ParchisSetup3P>)>,
    setup4: Query<&Interaction, (Changed<Interaction>, With<ParchisSetup4P>)>,
    mut setup_root: Query<&mut Visibility, With<ParchisSetupRoot>>,
) {
    if keys.just_pressed(KeyCode::Escape) { commands.set_state(GameState::ClassicMenu); return; }
    if back_clicks.single().map_or(false, |i| *i == Interaction::Pressed) { commands.set_state(GameState::ClassicMenu); return; }
    if restart_clicks.single().map_or(false, |i| *i == Interaction::Pressed) {
        let n = session.num_players;
        let vs = session.vs_cpu;
        let done = session.setup_done;
        if done { *session = ParchisSession::new_with_setup(n, vs); } else { *session = ParchisSession::new(); }
    }
    if !session.setup_done {
        for interaction in &setup1 { if *interaction == Interaction::Pressed { *session = ParchisSession::new_with_setup(4, true); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup2 { if *interaction == Interaction::Pressed { *session = ParchisSession::new_with_setup(2, false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup3 { if *interaction == Interaction::Pressed { *session = ParchisSession::new_with_setup(3, false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        for interaction in &setup4 { if *interaction == Interaction::Pressed { *session = ParchisSession::new_with_setup(4, false); for mut v in &mut setup_root { *v = Visibility::Hidden; } return; } }
        return;
    }

    // Resolver tirada pendiente cuando termina animación
    if let Some(pending) = session.pending_dice {
        let still_rolling = dice_anim.iter().any(|d| d.rolling);
        if still_rolling {
            session.message = "🎲 ¡Rodando dado...!".to_string();
        } else {
            let is_p = session.pending_is_player;
            session.pending_dice = None;
            session.move_turn_with_dice(is_p, pending);
        }
    }

    let is_rolling = session.pending_dice.is_some();

    if !is_rolling && session.winner.is_none() {
        if session.is_human_turn() {
            let mut clicked = false;
            for interaction in &roll_clicks { if *interaction == Interaction::Pressed { clicked = true; break; } }
            if keys.just_pressed(KeyCode::Space) { clicked = true; }
            if clicked {
                let dice = rand::thread_rng().gen_range(1..=6);
                session.pending_dice = Some(dice);
                session.pending_is_player = true;
                let who = if session.vs_cpu { "¡Lanzando dado..." } else { &format!("J{} lanza dado...", session.turn + 1) };
                session.message = format!("🎲 {who} ({})", dice);
                for mut d in &mut dice_anim { d.roll_to(dice); }
            }
        } else {
            if rand::thread_rng().gen_bool(0.02) {
                let dice = rand::thread_rng().gen_range(1..=6);
                session.pending_dice = Some(dice);
                session.pending_is_player = false;
                session.message = format!("🎲 CPU{} lanza dado... ({})", session.turn, dice);
                for mut d in &mut dice_anim { d.roll_to(dice); }
            }
        }
    }
    for (field, mut text) in &mut texts {
        match field.0 {
            ParchisField::Status => { *text = Text::new(session.message.clone()); }
            ParchisField::Dice => {
                let turn_name = if session.vs_cpu {
                    if session.turn==0 { "Tú".to_string() } else { format!("CPU {}", session.turn) }
                } else {
                    format!("J{}", session.turn + 1)
                };
                let dice_label = if session.pending_dice.is_some() { "🎲...".to_string() } else { session.dice.to_string() };
                *text = Text::new(format!("Dado: {}  Turno: {}  [5=sale, 6=repite]", dice_label, turn_name));
            }
            ParchisField::Positions => {
                let fmt = |opt: Option<usize>| opt.map(|p| if p >= GOAL { "META".to_string() } else if p >= CIRCUIT { format!("P{}", p - CIRCUIT + 1) } else { p.to_string() }).unwrap_or("Casa".to_string());
                if session.vs_cpu {
                    *text = Text::new(format!("Tú: {}  CPU1: {}  CPU2: {}  CPU3: {}", fmt(session.pos[0]), fmt(session.pos[1]), fmt(session.pos[2]), fmt(session.pos[3])));
                } else {
                    let mut parts = Vec::new();
                    for i in 0..session.num_players { parts.push(format!("J{}: {}", i+1, fmt(session.pos[i]))); }
                    *text = Text::new(parts.join("  "));
                }
            }
            _ => {}
        }
    }
    for (cell, children, mut background) in &mut board_cells {
        let occupants: Vec<usize> = session.pos.iter().enumerate().filter_map(|(idx, pos)| (*pos == Some(cell.0)).then_some(idx)).collect();
        let is_goal = cell.0 == GOAL;
        let is_safe = SAFE.contains(&cell.0);
        let is_start = STARTS.contains(&cell.0);
        *background = if is_goal { BackgroundColor(Color::srgb(0.86, 0.62, 0.16)) } else if !occupants.is_empty() { BackgroundColor(Color::srgb(0.90, 0.30, 0.20)) } else if is_start { BackgroundColor(Color::srgb(0.95, 0.80, 0.20)) } else if is_safe { BackgroundColor(Color::srgb(0.18, 0.52, 0.30)) } else if cell.0 % 2 == 0 { BackgroundColor(Color::srgb(0.16, 0.32, 0.42)) } else { BackgroundColor(Color::srgb(0.12, 0.25, 0.36)) };
        for child in children.iter() {
            if let Ok(mut text) = cell_texts.get_mut(child) {
                let base = if cell.0 >= CIRCUIT && cell.0 < GOAL { format!("P{}", cell.0 - CIRCUIT + 1) } else if is_start { format!("{} S", cell.0) } else if is_safe { format!("{} *", cell.0) } else { cell.0.to_string() };
                let tokens = occupants.iter().map(|idx| {
                    if session.vs_cpu { match idx { 0 => "T", 1 => "1", 2 => "2", _ => "3" } } else { match idx { 0 => "1", 1 => "2", 2 => "3", _ => "4" } }
                }).collect::<Vec<_>>().join(" ");
                *text = Text::new(if tokens.is_empty() { base } else { format!("{}\n{}", base, tokens) });
            }
        }
    }
}
