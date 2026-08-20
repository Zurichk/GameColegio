//! Modo tablero estilo Trivial: dados, categorías, preguntas y Estrellitas.
//!
//! El tablero reproduce la estructura clásica del Trivial Pursuit: una
//! **pista circular exterior** conectada por **6 radios** que convergen en
//! una **casilla central de salida** (hub).
//!
//! Distribución (85 casillas):
//! - 54 casillas de pista (hexágono de radio 9), de las cuales 6 son los
//!   **Estrellitas** (los vértices donde arranca cada radio).
//! - 30 casillas de radio (5 por cada uno de los 6 radios).
//! - 1 casilla central (hub): salida de los jugadores y reto final Tabú
//!   (la 7ª Estrellita).

pub mod questions;
mod bank_en;
mod bank_fr;
pub mod ui;

use std::sync::LazyLock;

use bevy::prelude::*;

use crate::game::GameState;
use crate::i18n::tr;
use questions::{
    normalize_answer, random_closed_question, random_question, random_taboo_card, Category,
    Difficulty, Question, TabooCard,
};

/// Casillas de la pista exterior (hexágono de radio 9: 6×9 = 54).
pub const BOARD_SIZE: usize = 54;
/// Casillas que tiene cada radio (entre el vértice-Estrellita y el centro).
pub const RADIO_LEN: u8 = 5;
/// Casilla central del tablero (hub de salida y reto final). Su índice va
/// después de las de la pista.
pub const CENTER_CELL: usize = BOARD_SIZE;
/// Número de categorías (y de Estrellitas de color que hay que reunir).
pub const NUM_CATEGORIES: usize = 6;
/// Radio del hexágono de la pista (casillas por lado).
const RING_RADIUS: i32 = 9;
/// Separación en píxeles entre casillas (eje x).
pub const BOARD_STEP: f32 = 46.0;

/// Colores de las fichas de los jugadores.
pub const PLAYER_COLORS: [Color; 4] = [
    Color::srgb(0.87, 0.26, 0.24),
    Color::srgb(0.24, 0.44, 0.86),
    Color::srgb(0.24, 0.70, 0.36),
    Color::srgb(0.96, 0.68, 0.12),
];

/// Tipo de casilla de la pista.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellKind {
    /// Vértice de la pista (Estrellita): otorga la Estrellita de esa categoría si
    /// se acierta. De aquí arranca el radio hacia el centro.
    Wedge(Category),
    /// Casilla normal de pregunta (tema de la categoría).
    Question(Category),
    /// Casilla Tabú: pregunta de adivinanza sin decir la palabra.
    Taboo,
    /// Casilla de dado: permite volver a tirar.
    Dice,
}

/// Posiciones axiales (q, r) de las 54 casillas de la pista, en orden
/// circular. El índice de la casilla es el índice del vector.
pub static RING: LazyLock<Vec<(i32, i32)>> = LazyLock::new(build_ring);

/// Índices (en `RING`) de las 6 casillas-Estrellita (los vértices donde
/// arrancan los radios), en orden circular.
pub static WEDGE_CELLS: LazyLock<Vec<usize>> = LazyLock::new(build_wedge_cells);

/// Tipo de cada casilla de la pista (índice = casilla).
static CELL_KINDS: LazyLock<Vec<CellKind>> = LazyLock::new(build_cell_kinds);

/// Genera las posiciones axiales (q, r) del hexágono de radio 9 (54
/// casillas), ordenadas circularmente por ángulo para que el movimiento
/// por la pista sea continuo.
fn build_ring() -> Vec<(i32, i32)> {
    let mut cells = Vec::new();
    for q in -RING_RADIUS..=RING_RADIUS {
        for r in -RING_RADIUS..=RING_RADIUS {
            if q.abs() <= RING_RADIUS && r.abs() <= RING_RADIUS && (q + r).abs() <= RING_RADIUS {
                let edge = q.abs().max(r.abs()).max((q + r).abs());
                if edge == RING_RADIUS {
                    cells.push((q, r));
                }
            }
        }
    }
    cells.sort_by(|a, b| {
        let angle = |(q, r): &(i32, i32)| {
            let x = *q as f32 + *r as f32 * 0.5;
            let y = *r as f32 * 0.8660254;
            y.atan2(x)
        };
        angle(a).partial_cmp(&angle(b)).unwrap()
    });
    cells
}

/// Los vértices son las casillas donde dos de las tres coordenadas axiales
/// tocan el borde del hexágono (p. ej. (9, 0) o (-9, 9)).
fn build_wedge_cells() -> Vec<usize> {
    RING.iter()
        .enumerate()
        .filter(|&(_, &(q, r))| {
            let s = -q - r;
            [q.abs(), r.abs(), s.abs()]
                .iter()
                .filter(|&&e| e == RING_RADIUS)
                .count()
                >= 2
        })
        .map(|(i, _)| i)
        .collect()
}

/// Asigna el tipo de cada casilla de la pista:
/// - Los 6 vértices son Estrellitas (una categoría de color cada uno).
/// - Las 48 intermedias: 30 de categorías (5 por color), 12 de dado
///   (como las 12 "roll again" del Trivial clásico) y 6 Tabú.
fn build_cell_kinds() -> Vec<CellKind> {
    let mut kinds = vec![CellKind::Question(Category::Math); RING.len()];

    for (i, kind) in kinds.iter_mut().enumerate() {
        if WEDGE_CELLS.binary_search(&i).is_ok() {
            let vertex = WEDGE_CELLS.iter().position(|&w| w == i).unwrap();
            *kind = CellKind::Wedge(Category::colored()[vertex % NUM_CATEGORIES]);
        }
    }

    // Reparto de las 48 intermedias: en cada bloque de 8 →
    // [cat, cat, cat, cat, cat, dado, Tabú, dado].
    let mut inter = 0usize;
    for kind in kinds.iter_mut() {
        if matches!(kind, CellKind::Wedge(_)) {
            continue;
        }
        *kind = match inter % 8 {
            5 => CellKind::Dice,
            6 => CellKind::Taboo,
            7 => CellKind::Dice,
            _ => CellKind::Question(Category::colored()[(inter / 8 * 5 + inter % 8) % 6]),
        };
        inter += 1;
    }
    kinds
}

/// Configuración de la partida elegida en la pantalla de configuración.
#[derive(Resource, Clone, Copy)]
pub struct BoardConfig {
    pub num_players: usize,
    pub difficulty: Difficulty,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            num_players: 2,
            difficulty: Difficulty::Easy,
        }
    }
}

/// Fase del turno en curso.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TurnPhase {
    /// Esperando a que el jugador actual lance el dado.
    Roll,
    /// El jugador está en el CENTRO y debe elegir por qué radio (Estrellita)
    /// salir hacia la pista.
    ChooseSpoke,
    /// Después de tirar el dado, el jugador elige la dirección
    /// (izquierda/derecha) antes de mover. `from_center_vertex`/`from_spoke`
    /// indican de dónde sale el movimiento cuando hay que elegir dirección
    /// por haber sobrepasado un vértice al salir del centro o de un radio.
    ChooseDirection {
        dice: u8,
        from_center_vertex: Option<usize>,
        from_spoke: Option<(usize, u8)>,
    },
    /// La ficha baja por el radio del vértice elegido contando casillas:
    /// `step` es la casilla de radio por la que va (1 = cerca del centro),
    /// `remaining` lo que queda por mover y `direction` la dirección de la
    /// pista si el dado sobrepasa el vértice.
    LeaveCenter {
        step: u8,
        vertex: usize,
        remaining: u8,
        direction: Option<i8>,
    },
    /// La ficha sube por un radio (desde una casilla de radio) hacia la
    /// pista, contando casillas. `step` es la casilla de radio actual
    /// (0 = junto al vértice).
    MoveSpoke {
        vertex: usize,
        step: u8,
        remaining: u8,
        direction: Option<i8>,
    },
    /// La ficha se está moviendo casilla a casilla por la pista.
    Move { remaining: u8, direction: i8 },
    /// La ficha sube por el radio hacia el centro para el reto final.
    ReturnToCenter { step: u8, vertex: usize },
    /// Respondiendo a una pregunta.
    Question,
    /// Tarjeta Tabú: describir la palabra sin decir las prohibidas.
    Taboo,
    /// Casilla de dado: se muestra el aviso de volver a tirar.
    ExtraTurn,
    /// Reto final (7ª Estrellita Tabú) disputado en el centro del tablero.
    Final,
    /// Viendo el resultado de la respuesta.
    Feedback,
    /// Partida terminada.
    Won,
}

/// Estado vivo de la partida de tablero.
#[derive(Resource)]
pub struct BoardState {
    pub config: BoardConfig,
    /// Casilla de la pista en la que está cada jugador, o `CENTER_CELL`
    /// si está en el centro del tablero.
    pub positions: Vec<usize>,
    /// Dirección de avance elegida por cada jugador (1 = horario,
    /// -1 = antihorario).
    pub directions: Vec<i8>,
    /// Estrellitas ganadas por jugador (por índice de categoría).
    pub wedges: Vec<Vec<bool>>,
    /// Casilla de radio en la que está cada jugador (si no está ni en el
    /// centro ni en la pista): `Some((vértice, paso))` con paso 0 junto al
    /// Estrellita.
    pub spoke: Vec<Option<(usize, u8)>>,
    /// Índice del jugador con el turno.
    pub current: usize,
    /// Último valor del dado.
    pub dice: Option<u8>,
    /// Fase del turno.
    pub phase: TurnPhase,
    /// Pregunta activa.
    pub question: Option<Question>,
    /// Tarjeta Tabú activa (casilla Tabú o reto final).
    pub taboo: Option<TabooCard>,
    /// Si la última respuesta fue correcta.
    pub last_correct: Option<bool>,
    /// Ganador de la partida, si existe.
    pub winner: Option<usize>,
    /// Contador de cambios; la interfaz lo usa para saber cuándo repintar.
    pub revision: u64,
}

impl BoardState {
    /// Crea una partida nueva a partir de la configuración.
    pub fn new(config: BoardConfig) -> Self {
        Self {
            config,
            // Todos los jugadores empiezan en el centro (hub) del tablero.
            positions: vec![CENTER_CELL; config.num_players],
            directions: vec![1; config.num_players],
            wedges: vec![vec![false; NUM_CATEGORIES]; config.num_players],
            spoke: vec![None; config.num_players],
            current: 0,
            dice: None,
            phase: TurnPhase::Roll,
            question: None,
            taboo: None,
            last_correct: None,
            winner: None,
            revision: 0,
        }
    }

    /// Nombre legible de un jugador.
    pub fn player_name(&self, index: usize) -> String {
        tr("Jugador {}").replace("{}", &(index + 1).to_string())
    }

    /// Lanza el dado y aplica la tirada.
    ///
    /// El valor del dado cuenta CASILLAS: desde el centro se baja por el
    /// radio elegido contando casillas (la Estrellita está a RADIO_LEN+1 del
    /// centro); si el número sobrepasa el vértice, se continúa por la pista
    /// eligiendo dirección. Desde una casilla de radio se sube hacia la
    /// pista contando casillas. Desde la pista se mueven las casillas
    /// indicadas en la dirección elegida.
    pub fn roll_dice(&mut self) {
        use rand::Rng;
        let dice = rand::thread_rng().gen_range(1..=6);
        self.roll_with(dice);
    }

    /// Aplica una tirada concreta (usado por la interfaz y los tests).
    pub fn roll_with(&mut self, dice: u8) {
        self.dice = Some(dice);
        self.revision += 1;

        let pos = self.positions[self.current];

        // Con las 6 Estrellitas de color: sube por el radio más cercano hacia
        // el CENTRO para disputar el reto final (7ª Estrellita Tabú).
        if self.wedges[self.current].iter().all(|&w| w) {
            if pos == CENTER_CELL {
                // Ya está en el centro (falló el reto antes): reintentarlo.
                // El reto final siempre es cerrado (4 opciones).
                self.question = Some(random_closed_question(Category::Taboo, self.config.difficulty));
                self.phase = TurnPhase::Final;
            } else {
                let vertex = vertex_of(pos);
                self.phase = TurnPhase::ReturnToCenter { step: 0, vertex };
            }
            self.revision += 1;
            return;
        }

        // En una casilla de radio: se sube hacia la pista contando casillas.
        if let Some((vertex, step)) = self.spoke[self.current] {
            let to_ring = step as u8 + 1;
            if dice > to_ring {
                // Sobrepasa el vértice: hay que elegir la dirección de la
                // pista antes de empezar a moverse.
                self.phase = TurnPhase::ChooseDirection {
                    dice,
                    from_center_vertex: None,
                    from_spoke: Some((vertex, step)),
                };
            } else {
                self.phase = TurnPhase::MoveSpoke {
                    vertex,
                    step,
                    remaining: dice,
                    direction: None,
                };
            }
            self.revision += 1;
            return;
        }

        if pos == CENTER_CELL {
            // Sale del centro: el jugador elige por qué radio (Estrellita) ir y
            // el dado cuenta las casillas.
            self.phase = TurnPhase::ChooseSpoke;
            self.revision += 1;
            return;
        }

        // En cualquier casilla de la pista se elige la dirección: permite
        // decidir en qué orden reunir las 6 Estrellitas.
        self.phase = TurnPhase::ChooseDirection {
            dice,
            from_center_vertex: None,
            from_spoke: None,
        };
        self.revision += 1;
    }

    /// El jugador elige el radio (Estrellita) por el que sale del centro. Si el
    /// dado sobrepasa el vértice (Estrellita), primero debe elegir la dirección
    /// de la pista por la que continuar.
    pub fn choose_spoke(&mut self, vertex: usize) {
        if self.phase != TurnPhase::ChooseSpoke {
            return;
        }
        let Some(dice) = self.dice else {
            return;
        };
        if dice > RADIO_LEN as u8 + 1 {
            self.phase = TurnPhase::ChooseDirection {
                dice,
                from_center_vertex: Some(vertex),
                from_spoke: None,
            };
        } else {
            self.phase = TurnPhase::LeaveCenter {
                step: 0,
                vertex,
                remaining: dice,
                direction: None,
            };
        }
        self.revision += 1;
    }

    /// El jugador elige la dirección y la ficha empieza a moverse hacia ese
    /// lado (por la pista, o continuando tras salir del centro o de un
    /// radio).
    pub fn choose_direction(&mut self, direction: i8) {
        let TurnPhase::ChooseDirection {
            dice,
            from_center_vertex,
            from_spoke,
        } = self.phase
        else {
            return;
        };
        self.directions[self.current] = direction;
        self.phase = if let Some(vertex) = from_center_vertex {
            TurnPhase::LeaveCenter {
                step: 0,
                vertex,
                remaining: dice,
                direction: Some(direction),
            }
        } else if let Some((vertex, step)) = from_spoke {
            TurnPhase::MoveSpoke {
                vertex,
                step,
                remaining: dice,
                direction: Some(direction),
            }
        } else {
            TurnPhase::Move {
                remaining: dice,
                direction,
            }
        };
        self.revision += 1;
    }

    /// Avanza la animación en curso (movimiento por la pista, bajada o
    /// subida por un radio, o salida contando casillas). Lo llama el sistema
    /// de animación cada paso.
    pub fn advance_animation(&mut self) {
        match self.phase {
            TurnPhase::Move { .. } => self.advance_move(),
            TurnPhase::LeaveCenter { .. } => self.advance_leave(),
            TurnPhase::MoveSpoke { .. } => self.advance_spoke(),
            TurnPhase::ReturnToCenter { .. } => self.advance_return(),
            _ => {}
        }
    }

    /// Avanza la ficha del jugador actual UNA casilla en su dirección.
    fn advance_move(&mut self) {
        let TurnPhase::Move {
            remaining,
            direction,
        } = self.phase
        else {
            return;
        };
        let pos = self.positions[self.current] as i32 + direction as i32;
        self.positions[self.current] = pos.rem_euclid(BOARD_SIZE as i32) as usize;
        if remaining == 1 {
            self.resolve_landing();
        } else {
            self.phase = TurnPhase::Move {
                remaining: remaining - 1,
                direction,
            };
        }
    }

/// Baja un paso por el radio de salida contando casillas: la Estrellita está
/// a RADIO_LEN+1 casillas del centro. Si el dado no llega, se aterriza en
/// una casilla de radio (pregunta del sector); si lo sobrepasa, se continúa
/// por la pista en la dirección elegida.
fn advance_leave(&mut self) {
    let TurnPhase::LeaveCenter {
        step,
        vertex,
        remaining,
        direction,
    } = self.phase
    else {
        return;
    };
    let step = step + 1;
    let remaining = remaining - 1;
    if remaining == 0 {
        if step <= RADIO_LEN {
            // Aterriza en una casilla de radio: pregunta del sector.
            let radio_step = RADIO_LEN - step;
            self.spoke[self.current] = Some((vertex, radio_step));
            self.positions[self.current] = CENTER_CELL;
            self.question = Some(random_question(
                Category::colored()[vertex],
                self.config.difficulty,
            ));
            self.phase = TurnPhase::Question;
        } else {
            // Aterriza justo en el vértice-Estrellita.
            self.positions[self.current] = WEDGE_CELLS[vertex];
            self.spoke[self.current] = None;
            self.resolve_landing();
        }
    } else if step > RADIO_LEN {
        // Llegó al vértice y le quedan casillas: continúa por la pista.
        self.positions[self.current] = WEDGE_CELLS[vertex];
        self.spoke[self.current] = None;
        self.phase = TurnPhase::Move {
            remaining,
            direction: direction.unwrap(),
        };
    } else {
        self.phase = TurnPhase::LeaveCenter {
            step,
            vertex,
            remaining,
            direction,
        };
    }
    self.revision += 1;
}

/// Sube un paso por un radio (desde una casilla de radio) hacia la pista,
/// contando casillas. Al llegar al vértice, continúa por la pista si le
/// quedan casillas.
fn advance_spoke(&mut self) {
    let TurnPhase::MoveSpoke {
        vertex,
        step,
        remaining,
        direction,
    } = self.phase
    else {
        return;
    };
    if step == 0 {
        // Siguiente casilla: el vértice-Estrellita.
        self.positions[self.current] = WEDGE_CELLS[vertex];
        self.spoke[self.current] = None;
        let remaining = remaining - 1;
        if remaining == 0 {
            self.resolve_landing();
        } else {
            self.phase = TurnPhase::Move {
                remaining,
                direction: direction.unwrap(),
            };
        }
    } else {
        let step = step - 1;
        let remaining = remaining - 1;
        if remaining == 0 {
            // Aterriza en otra casilla de radio: pregunta del sector.
            self.spoke[self.current] = Some((vertex, step));
            self.positions[self.current] = CENTER_CELL;
            self.question = Some(random_question(
                Category::colored()[vertex],
                self.config.difficulty,
            ));
            self.phase = TurnPhase::Question;
        } else {
            self.phase = TurnPhase::MoveSpoke {
                vertex,
                step,
                remaining,
                direction,
            };
        }
    }
    self.revision += 1;
}

    /// Sube un paso por el radio hacia el centro. Al llegar, se plantea el
    /// reto final Tabú (la 7ª Estrellita).
    fn advance_return(&mut self) {
        let TurnPhase::ReturnToCenter { step, vertex } = self.phase else {
            return;
        };
        if step + 1 >= RADIO_LEN {
            self.positions[self.current] = CENTER_CELL;
            // El reto final es una tarjeta Tabú real: palabra objetivo y 5
            // palabras prohibidas. Si el equipo la acierta, gana la partida.
            self.taboo = Some(random_taboo_card());
            self.phase = TurnPhase::Final;
            self.revision += 1;
        } else {
            self.phase = TurnPhase::ReturnToCenter {
                step: step + 1,
                vertex,
            };
        }
    }

    /// Procesa la casilla de llegada de la pista: pregunta, Tabú o dado.
    fn resolve_landing(&mut self) {
        let pos = self.positions[self.current];
        match cell_kind(pos) {
            // Casilla de dado: se vuelve a tirar (lo gestiona `continue_turn`).
            CellKind::Dice => {
                self.phase = TurnPhase::ExtraTurn;
                self.revision += 1;
            }
            // Casilla de categoría (tema propio) o vértice de Estrellita.
            CellKind::Wedge(category) | CellKind::Question(category) => {
                self.question = Some(random_question(category, self.config.difficulty));
                self.phase = TurnPhase::Question;
                self.revision += 1;
            }
            // Casilla Tabú: tarjeta Tabú real (palabra objetivo + 5 palabras
            // prohibidas). El jugador la describe a su equipo en voz alta.
            CellKind::Taboo => {
                self.taboo = Some(random_taboo_card());
                self.phase = TurnPhase::Taboo;
                self.revision += 1;
            }
        }
    }

    /// Registra la respuesta del jugador actual (pregunta cerrada) y otorga
    /// la Estrellita si procede. En el reto final, acertar significa ganar.
    pub fn answer(&mut self, selected: usize) {
        let Some(question) = self.question else {
            return;
        };
        if question.open {
            return;
        }
        let correct = selected == question.correct;
        self.resolve_answer(correct);
    }

    /// Registra la respuesta escrita de una pregunta abierta (sin opciones),
    /// comparándola de forma tolerante (minúsculas y sin acentos).
    pub fn answer_text(&mut self, text: &str) {
        let Some(question) = self.question else {
            return;
        };
        if !question.open {
            return;
        }
        let correct = normalize_answer(text) == normalize_answer(question.answer);
        self.resolve_answer(correct);
    }

    /// Resultado de una tarjeta Tabú: `true` si el equipo adivinó la palabra
    /// antes de que se acabara el tiempo, `false` si no. En el reto final
    /// (7ª Estrellita), acertar significa ganar la partida.
    pub fn taboo_result(&mut self, guessed: bool) {
        if !matches!(self.phase, TurnPhase::Taboo | TurnPhase::Final) {
            return;
        }
        self.resolve_answer(guessed);
    }

    /// Procesa una respuesta (correcta o no): otorga la Estrellita en los
    /// vértices, resuelve el reto final y pasa a la fase de feedback.
    fn resolve_answer(&mut self, correct: bool) {
        self.last_correct = Some(correct);
        self.revision += 1;

        if correct {
            match self.phase {
                TurnPhase::Final => {
                    // 7ª Estrellita conseguida: ¡gana la partida!
                    self.winner = Some(self.current);
                    self.phase = TurnPhase::Won;
                    self.revision += 1;
                    return;
                }
                TurnPhase::Question => {
                    let pos = self.positions[self.current];
                    if let CellKind::Wedge(category) = cell_kind(pos) {
                        if !self.wedges[self.current][category as usize] {
                            self.wedges[self.current][category as usize] = true;
                        }
                    }
                }
                _ => {}
            }
        }
        self.phase = TurnPhase::Feedback;
    }

    /// Se agotó el tiempo (1 minuto): cuenta como fallo y pasa el turno.
    pub fn timeout_question(&mut self) {
        match self.phase {
            // En las tarjetas Tabú el tiempo también corre: si se agota, fallo.
            TurnPhase::Taboo | TurnPhase::Final => {
                self.taboo_result(false);
            }
            _ => {
                let Some(question) = self.question else {
                    return;
                };
                if question.open {
                    self.resolve_answer(false);
                } else {
                    self.answer((question.correct + 1) % 4);
                }
            }
        }
        self.continue_turn();
    }

    /// Continúa tras el feedback o la casilla de dado.
    ///
    /// - Casilla de dado: el mismo jugador vuelve a tirar.
    /// - Acierto con las 6 Estrellitas: sube por el radio al centro para el
    ///   reto final (7ª Estrellita).
    /// - Acierto sin completarlos: repite turno.
    /// - Fallo: pasa el turno.
    pub fn continue_turn(&mut self) {
        if self.phase == TurnPhase::ExtraTurn {
            self.dice = None;
            self.question = None;
            self.taboo = None;
            self.last_correct = None;
            self.phase = TurnPhase::Roll;
            self.revision += 1;
            return;
        }
        if self.last_correct == Some(true) {
            if self.wedges[self.current].iter().all(|&w| w) {
                // ¡Completó las 6 Estrellitas de color! Vuelve al centro por
                // el radio de su vértice más cercano.
                let vertex = vertex_of(self.positions[self.current]);
                self.phase = TurnPhase::ReturnToCenter { step: 0, vertex };
                self.revision += 1;
                return;
            }
            // Acierto: el mismo jugador vuelve a tirar.
            self.dice = None;
            self.question = None;
            self.taboo = None;
            self.last_correct = None;
            self.phase = TurnPhase::Roll;
            self.revision += 1;
        } else {
            self.next_turn();
        }
    }

    fn next_turn(&mut self) {
        self.current = (self.current + 1) % self.config.num_players;
        self.dice = None;
        self.question = None;
        self.taboo = None;
        self.last_correct = None;
        self.phase = TurnPhase::Roll;
        self.revision += 1;
    }
}

/// Devuelve el tipo de una casilla de la pista (índice 0..54).
pub fn cell_kind(cell: usize) -> CellKind {
    if cell >= BOARD_SIZE {
        return CellKind::Question(Category::Math);
    }
    CELL_KINDS[cell]
}

/// Índice (0..6) del vértice-Estrellita más cercano a una casilla de la pista
/// (se usa para elegir el radio por el que subir al centro).
pub fn vertex_of(cell: usize) -> usize {
    let mut best = 0;
    let mut best_dist = usize::MAX;
    for (v, &wedge) in WEDGE_CELLS.iter().enumerate() {
        let dist = circular_dist(cell, wedge);
        if dist < best_dist {
            best_dist = dist;
            best = v;
        }
    }
    best
}

/// Distancia en casillas entre dos posiciones de la pista circular.
fn circular_dist(a: usize, b: usize) -> usize {
    let d = (a as i64 - b as i64).unsigned_abs() as usize;
    d.min(BOARD_SIZE - d)
}

/// Devuelve la posición en píxeles (x, y) del CENTRO de una casilla de la
/// pista, en coordenadas relativas al centro del hexágono (el origen es el
/// centro). La casilla central (hub) devuelve el origen.
pub fn board_cell_position(cell: usize) -> (f32, f32) {
    if cell >= BOARD_SIZE {
        return (0.0, 0.0);
    }
    let (q, r) = RING[cell];
    let x = (q as f32 + r as f32 * 0.5) * BOARD_STEP;
    let y = r as f32 * 0.8660254 * BOARD_STEP;
    (x, y)
}

/// Devuelve la posición en píxeles de la casilla `step` del radio que
/// arranca en el vértice `vertex` (0 = cerca del centro, 2 = cerca del
/// vértice). Se usa para animar la salida y la vuelta al centro.
pub fn radio_cell_px(vertex: usize, step: u8) -> (f32, f32) {
    let (vx, vy) = board_cell_position(WEDGE_CELLS[vertex % 6]);
    let t = 1.0 - (step as f32 + 1.0) / (RADIO_LEN as f32 + 1.0);
    (vx * t, vy * t)
}

/// Devuelve dónde debe dibujarse la ficha de `player` según el estado
/// actual: si el jugador está animando un radio (bajando, subiendo o en una
/// casilla de radio), en la casilla del radio; si no, en su casilla de la
/// pista (o el centro).
pub fn pawn_display_position(state: &BoardState, player: usize) -> (f32, f32) {
    if player == state.current {
        match state.phase {
            TurnPhase::LeaveCenter { step, vertex, .. } => {
                if step == 0 {
                    // Aún en el centro del tablero.
                    board_cell_position(CENTER_CELL)
                } else if step <= RADIO_LEN {
                    // En una casilla de radio (step 1 = cerca del centro).
                    radio_cell_px(vertex, RADIO_LEN - step)
                } else {
                    // Ya en el vértice o por la pista.
                    board_cell_position(state.positions[player])
                }
            }
            TurnPhase::MoveSpoke { vertex, step, .. } => radio_cell_px(vertex, step),
            TurnPhase::ReturnToCenter { step, vertex } => radio_cell_px(vertex, step),
            _ => board_cell_position(state.positions[player]),
        }
    } else if let Some((vertex, step)) = state.spoke[player] {
        // Otro jugador esperando en una casilla de radio.
        radio_cell_px(vertex, step)
    } else {
        board_cell_position(state.positions[player])
    }
}

/// Temporizador de 1 minuto para responder una pregunta (como en un
/// concurso): si se agota, cuenta como fallo y pasa el turno.
#[derive(Resource)]
pub struct QuestionTimer {
    /// Segundos que quedan.
    pub remaining: f32,
    /// `true` mientras hay una pregunta en curso con el reloj corriendo.
    pub armed: bool,
}

impl Default for QuestionTimer {
    fn default() -> Self {
        Self {
            remaining: 60.0,
            armed: false,
        }
    }
}

/// Plugin del modo tablero.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuestionTimer>()
            .add_systems(
            OnEnter(GameState::BoardSetup),
            (ui::ensure_config, ui::spawn_setup_ui),
        )
        .add_systems(OnExit(GameState::BoardSetup), ui::despawn_setup_ui)
        .add_systems(
            Update,
            (ui::setup_input, ui::refresh_setup_ui)
                .run_if(in_state(GameState::BoardSetup)),
        )
        .add_systems(OnEnter(GameState::BoardGame), ui::spawn_board_ui)
        .add_systems(OnExit(GameState::BoardGame), ui::despawn_board_ui)
        .add_systems(
            Update,
            (
                ui::board_input,
                ui::typing_input,
                ui::tick_question_timer,
                ui::refresh_board_ui,
                ui::animate_pawns,
                ui::update_pawn_positions,
                ui::update_pawn_star,
            )
                .chain()
                .run_if(in_state(GameState::BoardGame)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::questions::random_closed_question;

    fn state_2p() -> BoardState {
        BoardState::new(BoardConfig {
            num_players: 2,
            difficulty: Difficulty::Easy,
        })
    }

    /// Índice (>= 1) de la primera casilla que cumple el predicado.
    fn cell_index(pred: impl Fn(&CellKind) -> bool) -> usize {
        CELL_KINDS
            .iter()
            .enumerate()
            .find(|(i, k)| *i >= 1 && pred(k))
            .expect("casilla esperada en la pista")
            .0
    }

    #[test]
    fn board_has_85_cells_distribution() {
        // 54 de pista + 6 Estrellitas incluidas en ella + 30 de radios
        // (5 × 6) + 1 central = 85.
        assert_eq!(RING.len(), 54);
        assert_eq!(WEDGE_CELLS.len(), 6);
        assert_eq!(BOARD_SIZE + 6 * RADIO_LEN as usize + 1, 85);

        let wedges = CELL_KINDS
            .iter()
            .filter(|k| matches!(k, CellKind::Wedge(_)))
            .count();
        let dice = CELL_KINDS
            .iter()
            .filter(|k| matches!(k, CellKind::Dice))
            .count();
        let taboo = CELL_KINDS
            .iter()
            .filter(|k| matches!(k, CellKind::Taboo))
            .count();
        let questions = CELL_KINDS
            .iter()
            .filter(|k| matches!(k, CellKind::Question(_)))
            .count();
        assert_eq!(wedges, 6);
        assert_eq!(dice, 12);
        assert_eq!(taboo, 6);
        assert_eq!(questions, 30);
    }

    #[test]
    fn players_start_in_center_hub() {
        let state = state_2p();
        assert_eq!(state.positions[0], CENTER_CELL);
        assert_eq!(state.positions[1], CENTER_CELL);
    }

    #[test]
    fn roll_from_center_asks_spoke() {
        let mut state = state_2p();
        state.roll_dice();
        assert_eq!(state.phase, TurnPhase::ChooseSpoke);
    }

    #[test]
    fn leave_center_lands_on_wedge_and_asks() {
        let mut state = state_2p();
        // El jugador saca un 6: la Estrellita está a 6 casillas del centro
        // (5 de radio + el vértice), así que cae justo en él.
        state.roll_with(6);
        assert_eq!(state.phase, TurnPhase::ChooseSpoke);
        // El jugador elige por qué radio (Estrellita) salir.
        state.choose_spoke(2);
        assert!(matches!(
            state.phase,
            TurnPhase::LeaveCenter {
                step: 0,
                vertex: 2,
                ..
            }
        ));
        // 6 pasos contando casillas hasta el vértice-Estrellita.
        for _ in 0..6 {
            state.advance_animation();
        }
        assert_eq!(state.positions[0], WEDGE_CELLS[2]);
        assert_eq!(state.phase, TurnPhase::Question);
        assert!(state.question.is_some());
    }

    #[test]
    fn roll_1_from_center_lands_mid_spoke() {
        let mut state = state_2p();
        state.roll_with(1);
        state.choose_spoke(3);
        // 1 sola casilla: cae en el radio, cerca del centro.
        state.advance_animation();
        assert_eq!(state.phase, TurnPhase::Question);
        assert_eq!(state.spoke[0], Some((3, RADIO_LEN - 1)));
        assert_eq!(state.positions[0], CENTER_CELL);
    }

    #[test]
    fn roll_7_from_center_asks_direction_and_continues() {
        let mut state = state_2p();
        state.roll_with(7);
        state.choose_spoke(0);
        // 7 casillas sobrepasan el vértice (a 6 del centro): se elige
        // dirección para continuar por la pista.
        assert!(matches!(
            state.phase,
            TurnPhase::ChooseDirection {
                from_center_vertex: Some(0),
                ..
            }
        ));
        state.choose_direction(1);
        // 5 casillas de radio + llegar al vértice (aún con 1 casilla).
        for _ in 0..6 {
            state.advance_animation();
        }
        assert!(matches!(
            state.phase,
            TurnPhase::Move {
                remaining: 1,
                direction: 1
            }
        ));
        // Última casilla por la pista: 7 en total (5 radio + vértice + 1).
        state.advance_animation();
        assert_eq!(state.positions[0], (WEDGE_CELLS[0] + 1) % BOARD_SIZE);
    }

    #[test]
    fn spoke_cell_moves_toward_ring() {
        let mut state = state_2p();
        state.spoke[0] = Some((1, 1));
        state.positions[0] = CENTER_CELL;
        state.roll_with(2);
        assert!(matches!(
            state.phase,
            TurnPhase::MoveSpoke {
                vertex: 1,
                step: 1,
                remaining: 2,
                direction: None
            }
        ));
        // Paso a la casilla de radio 0 (junto al vértice) y al vértice.
        state.advance_animation();
        state.advance_animation();
        assert_eq!(state.positions[0], WEDGE_CELLS[1]);
        assert_eq!(state.phase, TurnPhase::Question);
    }

    #[test]
    fn spoke_overflow_asks_direction() {
        let mut state = state_2p();
        state.spoke[0] = Some((1, 0)); // junto al vértice
        state.positions[0] = CENTER_CELL;
        state.roll_with(3);
        assert!(matches!(
            state.phase,
            TurnPhase::ChooseDirection {
                from_spoke: Some((1, 0)),
                ..
            }
        ));
        state.choose_direction(-1);
        assert!(matches!(
            state.phase,
            TurnPhase::MoveSpoke {
                vertex: 1,
                step: 0,
                remaining: 3,
                direction: Some(-1)
            }
        ));
        // Llega al vértice y continúa por la pista con las 2 restantes.
        state.advance_animation();
        assert!(matches!(
            state.phase,
            TurnPhase::Move {
                remaining: 2,
                direction: -1
            }
        ));
    }

    #[test]
    fn roll_from_wedge_asks_direction() {
        let mut state = state_2p();
        state.positions[0] = WEDGE_CELLS[0];
        state.roll_dice();
        assert!(matches!(state.phase, TurnPhase::ChooseDirection { .. }));
    }

    #[test]
    fn roll_from_any_track_cell_asks_direction() {
        // En cualquier casilla de la pista (no solo vértices) se elige la
        // dirección: permite reunir las Estrellitas en el orden deseado.
        let mut state = state_2p();
        let target = cell_index(|k| matches!(k, CellKind::Question(_)));
        state.positions[0] = target;
        state.roll_dice();
        assert!(matches!(state.phase, TurnPhase::ChooseDirection { .. }));
    }

    #[test]
    fn choosing_direction_starts_moving() {
        let mut state = state_2p();
        state.dice = Some(4);
        state.phase = TurnPhase::ChooseDirection {
            dice: 4,
            from_center_vertex: None,
            from_spoke: None,
        };
        state.choose_direction(-1);
        assert_eq!(state.directions[0], -1);
        assert!(matches!(
            state.phase,
            TurnPhase::Move {
                remaining: 4,
                direction: -1
            }
        ));
    }

    #[test]
    fn moving_advances_one_cell_per_step() {
        let mut state = state_2p();
        state.positions[0] = 5;
        state.phase = TurnPhase::Move {
            remaining: 3,
            direction: 1,
        };
        state.advance_animation();
        assert_eq!(state.positions[0], 6);
        assert!(matches!(
            state.phase,
            TurnPhase::Move {
                remaining: 2,
                direction: 1
            }
        ));
    }

    #[test]
    fn landing_on_category_asks_question() {
        let mut state = state_2p();
        let target = cell_index(|k| matches!(k, CellKind::Question(_)));
        state.positions[0] = target - 1;
        state.phase = TurnPhase::Move {
            remaining: 1,
            direction: 1,
        };
        state.advance_animation();
        assert_eq!(state.positions[0], target);
        assert_eq!(state.phase, TurnPhase::Question);
        assert!(state.question.is_some());
    }

    #[test]
    fn landing_on_dice_gives_extra_turn() {
        let mut state = state_2p();
        let target = cell_index(|k| matches!(k, CellKind::Dice));
        state.positions[0] = target - 1;
        state.phase = TurnPhase::Move {
            remaining: 1,
            direction: 1,
        };
        state.advance_animation();
        assert_eq!(state.phase, TurnPhase::ExtraTurn);
        state.continue_turn();
        // Casilla de dado: el mismo jugador vuelve a tirar.
        assert_eq!(state.current, 0);
        assert_eq!(state.phase, TurnPhase::Roll);
    }

    #[test]
    fn landing_on_taboo_asks_taboo_question() {
        let mut state = state_2p();
        let target = cell_index(|k| matches!(k, CellKind::Taboo));
        state.positions[0] = target - 1;
        state.phase = TurnPhase::Move {
            remaining: 1,
            direction: 1,
        };
        state.advance_animation();
        // Las casillas Tabú ahora muestran una tarjeta Tabú real (palabra
        // objetivo + 5 prohibidas), no una pregunta con opciones.
        assert_eq!(state.phase, TurnPhase::Taboo);
        assert!(state.taboo.is_some());
        assert!(state.question.is_none());
    }

    #[test]
    fn correct_wedge_answer_grants_wedge() {
        let mut state = state_2p();
        let wedge = WEDGE_CELLS[0];
        let CellKind::Wedge(category) = cell_kind(wedge) else {
            panic!("no wedge");
        };
        state.positions[0] = wedge;
        state.phase = TurnPhase::Question;
        state.question = Some(random_closed_question(category, Difficulty::Easy));
        let correct = state.question.unwrap().correct;
        state.answer(correct);
        assert!(state.wedges[0][category as usize]);
    }

    #[test]
    fn correct_answer_repeats_turn() {
        let mut state = state_2p();
        state.question = Some(random_closed_question(Category::Science, Difficulty::Easy));
        let correct = state.question.unwrap().correct;
        let current = state.current;
        state.answer(correct);
        assert_eq!(state.phase, TurnPhase::Feedback);
        state.continue_turn();
        // Acierto: el mismo jugador vuelve a tirar.
        assert_eq!(state.current, current);
        assert_eq!(state.phase, TurnPhase::Roll);
    }

    #[test]
    fn wrong_answer_passes_turn() {
        let mut state = state_2p();
        state.question = Some(random_closed_question(Category::Science, Difficulty::Easy));
        let wrong = (state.question.unwrap().correct + 1) % 4;
        state.answer(wrong);
        state.continue_turn();
        assert_eq!(state.phase, TurnPhase::Roll);
        assert_eq!(state.current, 1);
    }

    #[test]
    fn open_question_accepts_tolerant_answer() {
        let mut state = state_2p();
        state.question = Some(Question {
            category: Category::Math,
            difficulty: Difficulty::Easy,
            text: "¿Cuál es la capital de Francia?",
            open: true,
            options: ["", "", "", ""],
            correct: 0,
            answer: "París",
        });
        // Sin opciones, con mayúsculas y espacio extra: acierta igual.
        state.answer_text("PARIS ");
        assert_eq!(state.last_correct, Some(true));
        assert_eq!(state.phase, TurnPhase::Feedback);
        state.continue_turn();
        // Acierto: repite turno.
        assert_eq!(state.current, 0);
        assert_eq!(state.phase, TurnPhase::Roll);
    }

    #[test]
    fn open_question_wrong_answer_passes_turn() {
        let mut state = state_2p();
        state.question = Some(Question {
            category: Category::Cs,
            difficulty: Difficulty::Medium,
            text: "¿En qué lenguaje está programado este juego?",
            open: true,
            options: ["", "", "", ""],
            correct: 0,
            answer: "Rust",
        });
        state.answer_text("Python");
        assert_eq!(state.last_correct, Some(false));
        state.continue_turn();
        assert_eq!(state.current, 1);
        assert_eq!(state.phase, TurnPhase::Roll);
    }

    #[test]
    fn timeout_question_passes_turn() {
        let mut state = state_2p();
        state.question = Some(random_closed_question(Category::Science, Difficulty::Easy));
        state.timeout_question();
        // Sin responder en 1 minuto: fallo y pasa el turno.
        assert_eq!(state.current, 1);
        assert_eq!(state.phase, TurnPhase::Roll);
    }

    #[test]
    fn shuffled_question_moves_correct_answer() {
        // La respuesta correcta debe poder aparecer en cualquier posición.
        let mut saw_not_a = false;
        for _ in 0..64 {
            let question = random_closed_question(Category::Math, Difficulty::Easy).shuffled();
            assert!(question.correct < 4);
            if question.correct != 0 {
                saw_not_a = true;
            }
        }
        assert!(saw_not_a, "tras 64 barajados la correcta no debería ser siempre A");
    }

    #[test]
    fn completing_all_wedges_returns_to_center() {
        let mut state = state_2p();
        state.wedges[0] = vec![true; NUM_CATEGORIES];
        state.positions[0] = 5;
        state.question = Some(random_closed_question(Category::Science, Difficulty::Easy));
        let correct = state.question.unwrap().correct;
        state.answer(correct);
        state.continue_turn();
        // Sube por el radio hacia el centro (5 casillas de radio).
        assert!(matches!(state.phase, TurnPhase::ReturnToCenter { .. }));
        for _ in 0..RADIO_LEN {
            state.advance_animation();
        }
        assert_eq!(state.phase, TurnPhase::Final);
        assert_eq!(state.positions[0], CENTER_CELL);
        // El reto final es una tarjeta Tabú real, no una pregunta con opciones.
        assert!(state.taboo.is_some());
    }

    #[test]
    fn final_question_correct_wins() {
        let mut state = state_2p();
        state.wedges[0] = vec![true; NUM_CATEGORIES];
        state.positions[0] = 5;
        state.roll_dice();
        assert!(matches!(state.phase, TurnPhase::ReturnToCenter { .. }));
        for _ in 0..RADIO_LEN {
            state.advance_animation();
        }
        assert_eq!(state.phase, TurnPhase::Final);
        assert!(state.taboo.is_some());
        // Acierta la tarjeta Tabú del reto final: gana la partida.
        state.taboo_result(true);
        assert_eq!(state.phase, TurnPhase::Won);
        assert_eq!(state.winner, Some(0));
    }

    #[test]
    fn final_question_wrong_passes_turn() {
        let mut state = state_2p();
        state.wedges[0] = vec![true; NUM_CATEGORIES];
        state.positions[0] = 5;
        state.roll_dice();
        for _ in 0..RADIO_LEN {
            state.advance_animation();
        }
        assert_eq!(state.phase, TurnPhase::Final);
        assert!(state.taboo.is_some());
        // Falló la tarjeta Tabú del reto final: pasa el turno.
        state.taboo_result(false);
        state.continue_turn();
        assert_eq!(state.current, 1);
        assert_eq!(state.phase, TurnPhase::Roll);
    }
}