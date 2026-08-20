//! Construcción del edificio del colegio con primitivas 3D y texturas
//! procedurales.
//!
//! Distribución (vista superior, X horizontal y Z vertical):
//!
//! ```text
//!          (-12, -9)              (12, -9)
//!        +----------------------+
//!        | Matemáticas | Historia | Informática |
//!        |  (-12..-4)  | (-4..4)  |  (4..12)    |   ← pared trasera (ventanas)
//!        +------+------+------+------+
//!        |   (puertas en la pared z = 0)          |
//!        |  PASILLO (z: 0..6)                     |
//!        |  RECEPCIÓN (z: 6..10)                  |
//!        |        [entrada x: -1.5..1.5]          |   ← pared frontal
//!        +----------------------------------------+
//!                    ↓ exterior (z > 10)
//! ```

use bevy::prelude::*;

use super::collision::Collider;
use super::textures;

/// Grosor de las paredes del edificio.
const WALL_THICKNESS: f32 = 0.3;
/// Altura de las paredes.
const WALL_HEIGHT: f32 = 3.2;
/// Ancho de los vanos de las puertas de las aulas.
const DOOR_WIDTH: f32 = 1.6;
/// Altura de las puertas de las aulas.
const DOOR_HEIGHT: f32 = 2.2;
/// Ancho de la entrada principal del colegio.
const ENTRANCE_WIDTH: f32 = 3.0;
/// Pared trasera (fondo de las aulas).
const BACK_WALL_Z: f32 = -9.0;
/// Pared frontal de las aulas (con las puertas hacia el pasillo).
const CLASSROOM_FRONT_Z: f32 = 0.0;
/// Pared frontal de la recepción (con la entrada principal).
const RECEPTION_FRONT_Z: f32 = 10.0;
/// Límite del edificio en el eje X (el edificio va de -X a +X).
const BUILDING_HALF_X: f32 = 12.0;

/// Colores de las asignaturas iniciales.
const COLOR_MATH: Color = Color::srgb(0.28, 0.52, 0.90);
const COLOR_HISTORY: Color = Color::srgb(0.85, 0.58, 0.28);
const COLOR_CS: Color = Color::srgb(0.38, 0.72, 0.45);

/// Nombres de las asignaturas para los carteles.
const SUBJECT_MATH: &str = "Matemáticas";
const SUBJECT_HISTORY: &str = "Historia";
const SUBJECT_CS: &str = "Informática";

/// Plugin que construye el colegio al iniciar la escena.
pub struct SchoolPlugin;

impl Plugin for SchoolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_school);
    }
}

/// Paleta de materiales reutilizada durante la construcción.
struct Palette {
    ground: Handle<StandardMaterial>,
    floor: Handle<StandardMaterial>,
    wall: Handle<StandardMaterial>,
    door: Handle<StandardMaterial>,
    window: Handle<StandardMaterial>,
    desk: Handle<StandardMaterial>,
    blackboard: Handle<StandardMaterial>,
    math_floor: Handle<StandardMaterial>,
    history_floor: Handle<StandardMaterial>,
    cs_floor: Handle<StandardMaterial>,
    math_accent: Handle<StandardMaterial>,
    history_accent: Handle<StandardMaterial>,
    cs_accent: Handle<StandardMaterial>,
    roof: Handle<StandardMaterial>,
    cement: Handle<StandardMaterial>,
    trunk: Handle<StandardMaterial>,
    foliage: Handle<StandardMaterial>,
    metal: Handle<StandardMaterial>,
    lamp: Handle<StandardMaterial>,
    frame: Handle<StandardMaterial>,
    planter: Handle<StandardMaterial>,
    banner: Handle<StandardMaterial>,
    water: Handle<StandardMaterial>,
    screen: Handle<StandardMaterial>,
    cork: Handle<StandardMaterial>,
    flower_red: Handle<StandardMaterial>,
    flower_yellow: Handle<StandardMaterial>,
    flower_purple: Handle<StandardMaterial>,
    cloud: Handle<StandardMaterial>,
    sun: Handle<StandardMaterial>,
    /// Panel de luz de techo (emisivo, casi blanco).
    panel: Handle<StandardMaterial>,
    /// Marco de la portería de fútbol (blanco).
    goal: Handle<StandardMaterial>,
    /// Cristal de ventana con brillo interior.
    glass: Handle<StandardMaterial>,
}

impl Palette {
    fn new(
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        // Texturas procedurales (una sola instancia por tipo).
        let floor_tex = images.add(textures::floor_tiles());
        let grass_tex = images.add(textures::grass());
        let plaster_tex = images.add(textures::plaster());
        let blackboard_tex = images.add(textures::blackboard());
        let wood_tex = images.add(textures::wood());

        let wall = add_textured(
            materials,
            &plaster_tex,
            Color::srgb(0.93, 0.91, 0.86),
            0.9,
        );
        let floor = add_textured(materials, &floor_tex, Color::WHITE, 0.7);
        let ground = add_textured(materials, &grass_tex, Color::WHITE, 0.9);
        let blackboard = add_textured(materials, &blackboard_tex, Color::WHITE, 0.5);
        let math_floor = add_textured(materials, &floor_tex, COLOR_MATH, 0.7);
        let history_floor = add_textured(materials, &floor_tex, COLOR_HISTORY, 0.7);
        let cs_floor = add_textured(materials, &floor_tex, COLOR_CS, 0.7);

        let door = add_textured(materials, &wood_tex, Color::srgb(0.58, 0.38, 0.22), 0.6);
        let desk = add_textured(materials, &wood_tex, Color::srgb(0.64, 0.48, 0.30), 0.6);
        let math_accent = add_solid(materials, COLOR_MATH, 0.8);
        let history_accent = add_solid(materials, COLOR_HISTORY, 0.8);
        let cs_accent = add_solid(materials, COLOR_CS, 0.8);
        let roof = add_solid(materials, Color::srgb(0.32, 0.33, 0.37), 0.8);
        let cement = add_solid(materials, Color::srgb(0.62, 0.62, 0.60), 0.9);
        let trunk = add_solid(materials, Color::srgb(0.40, 0.28, 0.16), 0.8);
        let foliage = add_solid(materials, Color::srgb(0.22, 0.48, 0.24), 0.9);
        let metal = add_solid(materials, Color::srgb(0.25, 0.25, 0.28), 0.4);
        let frame = add_textured(materials, &wood_tex, Color::srgb(0.55, 0.42, 0.26), 0.7);
        let planter = add_solid(materials, Color::srgb(0.72, 0.38, 0.24), 0.9);
        let banner = add_solid(materials, Color::srgb(0.20, 0.42, 0.30), 0.8);

        let lamp = materials.add(StandardMaterial {
            base_color: Color::srgb(0.40, 0.37, 0.30),
            emissive: LinearRgba::new(1.0, 0.9, 0.6, 1.0),
            emissive_exposure_weight: 2.0,
            ..default()
        });
        let window = materials.add(StandardMaterial {
            base_color: Color::srgba(0.55, 0.80, 0.95, 0.55),
            perceptual_roughness: 0.2,
            emissive: LinearRgba::new(0.30, 0.50, 0.70, 1.0),
            emissive_exposure_weight: 0.5,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let water = materials.add(StandardMaterial {
            base_color: Color::srgba(0.35, 0.68, 0.95, 0.75),
            perceptual_roughness: 0.1,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let screen = materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.16, 0.30),
            emissive: LinearRgba::new(0.3, 0.55, 0.95, 1.0),
            emissive_exposure_weight: 0.6,
            ..default()
        });
        let cork = add_solid(materials, Color::srgb(0.68, 0.50, 0.28), 0.9);
        let flower_red = add_solid(materials, Color::srgb(0.90, 0.25, 0.25), 0.8);
        let flower_yellow = add_solid(materials, Color::srgb(0.95, 0.80, 0.25), 0.8);
        let flower_purple = add_solid(materials, Color::srgb(0.65, 0.40, 0.85), 0.8);
        let cloud = add_solid(materials, Color::srgb(0.96, 0.97, 1.0), 1.0);
        let panel = materials.add(StandardMaterial {
            base_color: Color::srgb(0.96, 0.96, 0.90),
            emissive: LinearRgba::new(1.0, 0.95, 0.80, 1.0),
            emissive_exposure_weight: 1.6,
            ..default()
        });
        let goal = add_solid(materials, Color::srgb(0.96, 0.96, 0.94), 0.4);
        let glass = materials.add(StandardMaterial {
            base_color: Color::srgba(0.70, 0.88, 0.98, 0.35),
            perceptual_roughness: 0.1,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let sun = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.87, 0.40),
            emissive: LinearRgba::new(1.0, 0.82, 0.35, 1.0),
            emissive_exposure_weight: 1.4,
            ..default()
        });

        Self {
            ground,
            floor,
            wall,
            door,
            window,
            desk,
            blackboard,
            math_floor,
            history_floor,
            cs_floor,
            math_accent,
            history_accent,
            cs_accent,
            roof,
            cement,
            trunk,
            foliage,
            metal,
            lamp,
            frame,
            planter,
            banner,
            water,
            screen,
            cork,
            flower_red,
            flower_yellow,
            flower_purple,
            cloud,
            sun,
            panel,
            goal,
            glass,
        }
    }
}

/// Crea un material opaco con un color sólido.
fn add_solid(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
    roughness: f32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: roughness,
        ..default()
    })
}

/// Crea un material con textura (el color base se multiplica por la textura).
fn add_textured(
    materials: &mut Assets<StandardMaterial>,
    texture: &Handle<Image>,
    color: Color,
    roughness: f32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        base_color_texture: Some(texture.clone()),
        perceptual_roughness: roughness,
        ..default()
    })
}

/// Construye el colegio completo.
fn build_school(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let palette = Palette::new(&mut materials, &mut images);
    // Fuente con acentos del español para los carteles de asignatura.
    let sign_font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");

    spawn_ground(&mut commands, &mut meshes, &palette);
    spawn_outer_walls(&mut commands, &mut meshes, &palette);
    spawn_classroom_walls(&mut commands, &mut meshes, &palette);
    spawn_roof(&mut commands, &mut meshes, &palette);
    spawn_doors(&mut commands, &mut meshes, &palette);
    spawn_windows(&mut commands, &mut meshes, &palette);
    spawn_floor_accents(&mut commands, &mut meshes, &palette);
    spawn_furniture(&mut commands, &mut meshes, &mut materials, &palette);
    spawn_entrance_path(&mut commands, &mut meshes, &palette);
    spawn_lamps(&mut commands, &mut meshes, &palette);
    spawn_benches(&mut commands, &mut meshes, &palette);
    spawn_trees(&mut commands, &mut meshes, &palette);
    spawn_signs(&mut commands, &mut meshes, &palette, &sign_font);
    spawn_garden(&mut commands, &mut meshes, &palette);
    spawn_computers(&mut commands, &mut meshes, &palette);
    spawn_desk_books(&mut commands, &mut meshes, &mut materials);
    spawn_corkboard_notes(&mut commands, &mut meshes, &mut materials, &palette);
    spawn_welcome_sign(&mut commands, &mut meshes, &palette, &sign_font);
    spawn_clouds(&mut commands, &mut meshes, &palette);
    spawn_bushes(&mut commands, &mut meshes, &palette);
    spawn_roof_details(&mut commands, &mut meshes, &palette);
    spawn_hedge(&mut commands, &mut meshes, &palette);
    spawn_flagpole(&mut commands, &mut meshes, &palette);
    spawn_sun(&mut commands, &mut meshes, &palette);
    spawn_flower_beds(&mut commands, &mut meshes, &palette);
    spawn_ceiling_lights(&mut commands, &mut meshes, &palette);
    spawn_lockers(&mut commands, &mut meshes, &palette);
    spawn_classroom_posters(&mut commands, &mut meshes, &palette);
    spawn_basketball_hoop(&mut commands, &mut meshes, &palette);
    spawn_soccer_goals(&mut commands, &mut meshes, &palette);
}

/// Carteles de asignatura sobre cada puerta, con su nombre en texto 3D.
fn spawn_signs(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
    font: &Handle<Font>,
) {
    let signs = [
        (-8.0, &palette.math_accent, SUBJECT_MATH),
        (0.0, &palette.history_accent, SUBJECT_HISTORY),
        (8.0, &palette.cs_accent, SUBJECT_CS),
    ];
    for (cx, material, name) in signs {
        let sign = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(2.2, 0.35, 0.08))),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(cx, 2.75, CLASSROOM_FRONT_Z + 0.21),
            ))
            .id();
        commands.entity(sign).with_children(|parent| {
            parent.spawn((
                Text2d::new(name),
                TextFont {
                    font: font.clone(),
                    font_size: 0.16,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.06),
            ));
        });
    }
}

/// Crea un cubo decorativo (sin colisión).
fn spawn_box(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: &Handle<StandardMaterial>,
    center: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(center),
    ));
}

/// Crea un cubo sólido con colisión AABB (paredes y mobiliario).
fn spawn_solid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: &Handle<StandardMaterial>,
    center: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(center),
        Collider::new(size / 2.0),
    ));
}

/// Suelo exterior y suelo interior del edificio.
fn spawn_ground(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    // El suelo exterior es un cubo muy fino cuya cara superior queda en y = 0.
    spawn_box(
        commands,
        meshes,
        &palette.ground,
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(220.0, 0.2, 220.0),
    );
    // Suelo interior (pasillo + recepción).
    let depth = RECEPTION_FRONT_Z - BACK_WALL_Z;
    spawn_box(
        commands,
        meshes,
        &palette.floor,
        Vec3::new(0.0, -0.02, (BACK_WALL_Z + RECEPTION_FRONT_Z) / 2.0),
        Vec3::new(BUILDING_HALF_X * 2.0, 0.04, depth),
    );
}

/// Paredes exteriores del edificio.
fn spawn_outer_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let wall = &palette.wall;
    let t = WALL_THICKNESS;

    // Pared trasera (z = BACK_WALL_Z).
    spawn_solid(
        commands,
        meshes,
        wall,
        Vec3::new(0.0, WALL_HEIGHT / 2.0, BACK_WALL_Z),
        Vec3::new(BUILDING_HALF_X * 2.0, WALL_HEIGHT, t),
    );

    // Paredes laterales (x = ±BUILDING_HALF_X), de la pared trasera a la
    // entrada.
    let depth = RECEPTION_FRONT_Z - BACK_WALL_Z + t;
    for x in [-BUILDING_HALF_X, BUILDING_HALF_X] {
        spawn_solid(
            commands,
            meshes,
            wall,
            Vec3::new(x, WALL_HEIGHT / 2.0, (BACK_WALL_Z + RECEPTION_FRONT_Z) / 2.0),
            Vec3::new(t, WALL_HEIGHT, depth),
        );
    }

    // Pared frontal de la recepción, dividida por la entrada principal.
    let half_entrance = ENTRANCE_WIDTH / 2.0;
    let side_width = BUILDING_HALF_X - half_entrance;
    for sign in [-1.0, 1.0] {
        spawn_solid(
            commands,
            meshes,
            wall,
            Vec3::new(
                sign * (BUILDING_HALF_X + half_entrance) / 2.0,
                WALL_HEIGHT / 2.0,
                RECEPTION_FRONT_Z,
            ),
            Vec3::new(side_width, WALL_HEIGHT, t),
        );
    }
}

/// Pared frontal de las aulas (con tres vanos de puerta) y separadores.
fn spawn_classroom_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let wall = &palette.wall;
    let t = WALL_THICKNESS;

    // La pared en z = 0 separa el pasillo de las aulas y contiene tres vanos
    // para las puertas, centrados en x = -8, 0 y 8.
    let half_door = DOOR_WIDTH / 2.0;
    let segments = [
        (-12.0, -8.0 - half_door),           // -12 .. -8.8
        (-8.0 + half_door, 0.0 - half_door), // -7.2 .. -0.8
        (0.0 + half_door, 8.0 - half_door),  // 0.8 .. 7.2
        (8.0 + half_door, 12.0),             // 8.8 .. 12
    ];
    for (x0, x1) in segments {
        spawn_solid(
            commands,
            meshes,
            wall,
            Vec3::new((x0 + x1) / 2.0, WALL_HEIGHT / 2.0, CLASSROOM_FRONT_Z),
            Vec3::new(x1 - x0, WALL_HEIGHT, t),
        );
    }

    // Separadores entre aulas (x = -4 y x = 4), del fondo a la pared frontal.
    for x in [-4.0, 4.0] {
        spawn_solid(
            commands,
            meshes,
            wall,
            Vec3::new(x, WALL_HEIGHT / 2.0, (BACK_WALL_Z + CLASSROOM_FRONT_Z) / 2.0),
            Vec3::new(t, WALL_HEIGHT, CLASSROOM_FRONT_Z - BACK_WALL_Z),
        );
    }
}

/// Techo del edificio: tapa el interior y sobresale ligeramente de las
/// paredes, dejando de verse el cielo desde dentro de las aulas.
fn spawn_roof(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    spawn_solid(
        commands,
        meshes,
        &palette.roof,
        Vec3::new(0.0, WALL_HEIGHT + 0.125, 0.5),
        Vec3::new(BUILDING_HALF_X * 2.0 + 1.2, 0.25, RECEPTION_FRONT_Z - BACK_WALL_Z + 1.0),
    );
}

/// Puerta deslizante que el jugador puede abrir/cerrar con la tecla E
/// cuando está cerca (Fase 2: puertas e interacción).
#[derive(Component)]
pub struct Door {
    /// `true` si está abierta (deslizada hacia el lado, escondida en la
    /// pared); `false` si bloquea el vano.
    pub open: bool,
    /// Identificador estable para guardar/restaurar su estado (0..2 aulas,
    /// 3 entrada principal).
    pub id: u8,
    /// Posición X del centro del panel cuando está cerrada (en el vano).
    pub closed_x: f32,
    /// Posición X del centro del panel cuando está abierta (dentro de la
    /// pared, junto al vano).
    pub open_x: f32,
}

/// Puertas del colegio: tres de las aulas (en la pared z = 0) y la entrada
/// principal (en z = 10). Empiezan abiertas para que el edificio sea
/// recorrible; con la tecla E se abren y cierran deslizándose.
fn spawn_doors(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    // Puertas de las aulas: vanos de DOOR_WIDTH centrados en cx. El panel es
    // un rectángulo en el plano XY (ancho en X, delgado en Z) para tapar el
    // vano de la pared; abierta se desliza dentro de la pared (x + DOOR_WIDTH).
    for (id, cx) in [-8.0, 0.0, 8.0].into_iter().enumerate() {
        let panel = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(DOOR_WIDTH, DOOR_HEIGHT, 0.12))),
                MeshMaterial3d(palette.door.clone()),
                Transform::from_xyz(cx + DOOR_WIDTH + 0.2, DOOR_HEIGHT / 2.0, CLASSROOM_FRONT_Z),
                Door {
                    open: true,
                    id: id as u8,
                    closed_x: cx,
                    open_x: cx + DOOR_WIDTH + 0.2,
                },
                Collider::new(Vec3::new(DOOR_WIDTH / 2.0, DOOR_HEIGHT / 2.0, 0.06)),
            ))
            .id();
        // Tirador pequeño en el centro del panel.
        commands.entity(panel).with_children(|door| {
            door.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.05, 0.16, 0.05))),
                MeshMaterial3d(palette.metal.clone()),
                Transform::from_xyz(0.0, 0.0, 0.45),
            ));
        });
    }

    // Entrada principal: vano de ENTRANCE_WIDTH en la pared frontal (z = 10).
    // El panel también es un rectángulo en el plano XY; abierta se desliza
    // dentro de la pared derecha (x = 3.1).
    let entrance = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(ENTRANCE_WIDTH, 2.6, 0.14))),
            MeshMaterial3d(palette.door.clone()),
            Transform::from_xyz(3.1, 1.3, RECEPTION_FRONT_Z),
            Door {
                open: true,
                id: 3,
                closed_x: 0.0,
                open_x: 3.1,
            },
            Collider::new(Vec3::new(ENTRANCE_WIDTH / 2.0, 1.3, 0.07)),
        ))
        .id();
    commands.entity(entrance).with_children(|door| {
        door.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.05, 0.2, 0.06))),
            MeshMaterial3d(palette.metal.clone()),
            Transform::from_xyz(0.0, 0.0, -0.45),
        ));
    });
}

/// Ventanas simples en la pared trasera, con marco blanco.
fn spawn_windows(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    let win_w = 1.6;
    let win_h = 1.2;
    let win_y = 2.0;
    let win_z = BACK_WALL_Z + 0.17;
    for cx in [-8.0, 0.0, 8.0] {
        for dx in [-2.4, 2.4] {
            let x = cx + dx;
            spawn_box(
                commands,
                meshes,
                &palette.window,
                Vec3::new(x, win_y, win_z),
                Vec3::new(win_w, win_h, 0.08),
            );
            // Marcos: dos listones verticales y dos horizontales.
            for mx in [x - win_w / 2.0, x + win_w / 2.0] {
                spawn_box(
                    commands,
                    meshes,
                    &palette.wall,
                    Vec3::new(mx, win_y, win_z),
                    Vec3::new(0.08, win_h + 0.12, 0.1),
                );
            }
            for my in [win_y - win_h / 2.0, win_y + win_h / 2.0] {
                spawn_box(
                    commands,
                    meshes,
                    &palette.wall,
                    Vec3::new(x, my, win_z),
                    Vec3::new(win_w + 0.12, 0.08, 0.1),
                );
            }
        }
    }
}

/// Suelos de aula con el color de cada asignatura.
fn spawn_floor_accents(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let classrooms = [
        (-8.0, &palette.math_floor),
        (0.0, &palette.history_floor),
        (8.0, &palette.cs_floor),
    ];
    for (cx, material) in classrooms {
        spawn_box(
            commands,
            meshes,
            material,
            Vec3::new(cx, -0.01, (BACK_WALL_Z + CLASSROOM_FRONT_Z) / 2.0),
            Vec3::new(7.6, 0.02, CLASSROOM_FRONT_Z - BACK_WALL_Z - 0.4),
        );
    }
}

/// Mostrador de recepción, mobiliario de las aulas y carteles con texto.
fn spawn_furniture(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
) {
    // Mostrador de recepción.
    spawn_solid(
        commands,
        meshes,
        &palette.desk,
        Vec3::new(3.4, 0.45, 8.2),
        Vec3::new(1.6, 0.9, 0.7),
    );

    for cx in [-8.0, 0.0, 8.0] {
        // Pizarra en la pared trasera, con marco de madera.
        spawn_box(
            commands,
            meshes,
            &palette.blackboard,
            Vec3::new(cx, 2.3, BACK_WALL_Z + 0.17),
            Vec3::new(3.0, 1.2, 0.06),
        );
        for (mx, my, sx, sy) in [
            (cx - 1.6, 2.3, 0.1, 1.4), // marco izquierdo
            (cx + 1.6, 2.3, 0.1, 1.4), // marco derecho
            (cx, 3.0, 3.3, 0.1),       // marco superior
            (cx, 1.6, 3.3, 0.1),       // marco inferior
        ] {
            spawn_box(
                commands,
                meshes,
                &palette.frame,
                Vec3::new(mx, my, BACK_WALL_Z + 0.19),
                Vec3::new(sx, sy, 0.05),
            );
        }
        // Mesa del profesor.
        spawn_solid(
            commands,
            meshes,
            &palette.desk,
            Vec3::new(cx, 0.35, -7.6),
            Vec3::new(1.2, 0.7, 0.6),
        );
        // Pupitres en dos columnas.
        for dx in [-2.0, 2.0] {
            for dz in [-5.2, -3.6] {
                spawn_solid(
                    commands,
                    meshes,
                    &palette.desk,
                    Vec3::new(cx + dx, 0.35, dz),
                    Vec3::new(0.8, 0.7, 0.5),
                );
            }
        }
        // Estantería con libros en la pared lateral derecha de cada aula.
        spawn_bookshelf(commands, meshes, materials, palette, cx + 3.35, -4.5);
    }

    // Cuadros decorativos en la pared del pasillo (reutiliza los colores
    // de las asignaturas).
    for (x, accent) in [
        (-6.0, &palette.history_accent),
        (0.0, &palette.math_accent),
        (6.0, &palette.cs_accent),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 0.7, 0.06))),
            MeshMaterial3d(accent.clone()),
            Transform::from_xyz(x, 2.2, CLASSROOM_FRONT_Z + 0.2),
        ));
    }

    // Reloj de pared en la recepción.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.45, 0.06))),
        MeshMaterial3d(palette.wall.clone()),
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(-4.0, 2.6, RECEPTION_FRONT_Z - 0.16)),
    ));
    // Manecillas del reloj.
    for (sx, sy) in [(0.0, 0.22), (0.18, 0.0)] {
        spawn_box(
            commands,
            meshes,
            &palette.metal,
            Vec3::new(-4.0 + sx, 2.6 + sy, RECEPTION_FRONT_Z - 0.12),
            Vec3::new(0.05, if sy > 0.0 { 0.45 } else { 0.05 }, 0.05),
        );
    }

    // Macetas con plantas junto a la entrada (interior).
    for x in [-2.2, 2.2] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.28, 0.45))),
            MeshMaterial3d(palette.planter.clone()),
            Transform::from_xyz(x, 0.22, 9.4),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.3))),
            MeshMaterial3d(palette.foliage.clone()),
            Transform::from_xyz(x, 0.8, 9.4),
        ));
    }

    // Bandera junto a la entrada (poste + lienzo).
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.04, 2.2))),
        MeshMaterial3d(palette.metal.clone()),
        Transform::from_xyz(2.6, 1.1, 9.2),
    ));
    spawn_box(
        commands,
        meshes,
        &palette.banner,
        Vec3::new(3.0, 2.0, 9.2),
        Vec3::new(0.8, 0.5, 0.03),
    );

    // Carteles de asignatura sobre cada puerta, con su nombre en texto 3D.
    let signs = [
        (-8.0, &palette.math_accent, SUBJECT_MATH),
        (0.0, &palette.history_accent, SUBJECT_HISTORY),
        (8.0, &palette.cs_accent, SUBJECT_CS),
    ];
    for (cx, material, name) in signs {
        let sign = commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(2.2, 0.35, 0.08))),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(cx, 2.75, CLASSROOM_FRONT_Z + 0.21),
            ))
            .id();
        commands.entity(sign).with_children(|parent| {
            parent.spawn((
                Text2d::new(name),
                TextFont {
                    font_size: 0.16,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 0.0, 0.06),
            ));
        });
    }
}

/// Camino de cemento desde la entrada hacia el exterior (hasta el seto).
fn spawn_entrance_path(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    spawn_box(
        commands,
        meshes,
        &palette.cement,
        Vec3::new(0.0, 0.015, 13.0),
        Vec3::new(3.4, 0.03, 6.0),
    );
    // Segunda losa hasta el hueco del seto.
    spawn_box(
        commands,
        meshes,
        &palette.cement,
        Vec3::new(0.0, 0.015, 21.0),
        Vec3::new(3.4, 0.03, 10.0),
    );
}

/// Farolas decorativas junto a la entrada, con cabezal emisivo.
fn spawn_lamps(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    for x in [-3.2, 3.2] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.08, 3.6))),
            MeshMaterial3d(palette.metal.clone()),
            Transform::from_xyz(x, 1.8, 9.8),
            Collider::new(Vec3::new(0.15, 1.8, 0.15)),
        ));
        // Brazo orientado hacia el camino central.
        let dir = if x < 0.0 { 1.0 } else { -1.0 };
        spawn_box(
            commands,
            meshes,
            &palette.metal,
            Vec3::new(x + dir * 0.35, 3.55, 9.8),
            Vec3::new(0.7, 0.08, 0.08),
        );
        spawn_box(
            commands,
            meshes,
            &palette.lamp,
            Vec3::new(x + dir * 0.75, 3.55, 9.8),
            Vec3::new(0.32, 0.22, 0.22),
        );
    }
}

/// Bancos a los lados de la entrada.
fn spawn_benches(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    for x in [-3.5, 3.5] {
        let z = 9.4;
        // Patas.
        for dx in [-0.5, 0.5] {
            spawn_solid(
                commands,
                meshes,
                &palette.metal,
                Vec3::new(x + dx, 0.2, z),
                Vec3::new(0.08, 0.4, 0.4),
            );
        }
        // Asiento.
        spawn_solid(
            commands,
            meshes,
            &palette.desk,
            Vec3::new(x, 0.45, z),
            Vec3::new(1.2, 0.1, 0.45),
        );
        // Respaldo.
        spawn_solid(
            commands,
            meshes,
            &palette.desk,
            Vec3::new(x, 0.85, z - 0.2),
            Vec3::new(1.2, 0.45, 0.08),
        );
    }
}

/// Estantería con libros de colores junto a la pared lateral de un aula.
fn spawn_bookshelf(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
    x: f32,
    z: f32,
) {
    // Materiales de los libros, un color por libro.
    let book_colors = [
        Color::srgb(0.75, 0.25, 0.25),
        Color::srgb(0.25, 0.45, 0.75),
        Color::srgb(0.30, 0.65, 0.40),
        Color::srgb(0.75, 0.60, 0.20),
        Color::srgb(0.55, 0.35, 0.70),
        Color::srgb(0.20, 0.65, 0.65),
    ];
    let book_materials: Vec<_> = book_colors
        .iter()
        .map(|c| add_solid(materials, *c, 0.8))
        .collect();

    // Laterales.
    for dx in [-0.58, 0.58] {
        spawn_solid(
            commands,
            meshes,
            &palette.frame,
            Vec3::new(x + dx, 1.0, z),
            Vec3::new(0.08, 2.0, 0.32),
        );
    }
    // Baldas con libros.
    for (i, sy) in [0.45, 1.05, 1.65].into_iter().enumerate() {
        spawn_solid(
            commands,
            meshes,
            &palette.frame,
            Vec3::new(x, sy, z),
            Vec3::new(1.16, 0.06, 0.32),
        );
        for j in 0..6 {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.13, 0.26, 0.22))),
                MeshMaterial3d(book_materials[(j + i) % 6].clone()),
                Transform::from_xyz(x + (j as f32 - 2.5) * 0.17, sy + 0.16, z),
            ));
        }
    }
    // Tope superior.
    spawn_solid(
        commands,
        meshes,
        &palette.frame,
        Vec3::new(x, 2.05, z),
        Vec3::new(1.16, 0.06, 0.32),
    );
}

/// Árboles simples (tronco + copa frondosa) alrededor del edificio.
fn spawn_trees(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    let positions = [
        (-20.0, 18.0),
        (0.0, 22.0),
        (20.0, 18.0),
        (20.0, -14.0),
        (0.0, -18.0),
        (-20.0, -14.0),
        (-24.0, 2.0),
        (24.0, 2.0),
    ];
    for (x, z) in positions {
        // Tronco.
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.28, 2.6))),
            MeshMaterial3d(palette.trunk.clone()),
            Transform::from_xyz(x, 1.3, z),
            Collider::new(Vec3::new(0.35, 1.3, 0.35)),
        ));
        // Copa formada por cinco esferas superpuestas (más orgánica).
        for (dx, dy, dz, radius) in [
            (-0.7, 0.2, -0.3, 0.9),
            (0.7, 0.2, -0.3, 0.9),
            (-0.4, 0.2, 0.6, 1.0),
            (0.4, 0.2, 0.6, 1.0),
            (0.0, 0.9, 0.1, 1.3),
        ] {
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(radius))),
                MeshMaterial3d(palette.foliage.clone()),
                Transform::from_xyz(x + dx, 2.8 + dy, z + dz),
            ));
        }
    }
}

/// Jardín delantero: fuente, flores de colores y papeleras.
fn spawn_garden(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    // Fuente circular delante de la entrada: base, borde y chorros de agua.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.85, 0.16))),
        MeshMaterial3d(palette.cement.clone()),
        Transform::from_xyz(0.0, 0.08, 13.2),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.85, 0.10))),
        MeshMaterial3d(palette.frame.clone()),
        Transform::from_xyz(0.0, 0.21, 13.2),
    ));
    for dx in [-0.35, 0.35] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.06, 0.55))),
            MeshMaterial3d(palette.water.clone()),
            Transform::from_xyz(dx, 0.55, 13.2),
        ));
    }

    // Flores de colores a los lados de la entrada (tallo + cabeza).
    let flower_materials = [
        &palette.flower_red,
        &palette.flower_yellow,
        &palette.flower_purple,
    ];
    let flower_positions = [
        (-9.0, 14.0),
        (-7.5, 15.0),
        (-6.0, 14.0),
        (-8.2, 16.0),
        (6.0, 14.0),
        (7.5, 15.0),
        (9.0, 14.0),
        (8.2, 16.0),
        (-10.5, 12.5),
        (10.5, 12.5),
    ];
    for (i, (x, z)) in flower_positions.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.03, 0.35))),
            MeshMaterial3d(palette.foliage.clone()),
            Transform::from_xyz(*x, 0.17, *z),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.10))),
            MeshMaterial3d(flower_materials[i % 3].clone()),
            Transform::from_xyz(*x, 0.42, *z),
        ));
    }

    // Papeleras junto a los bancos.
    for x in [-4.7, 4.7] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.22, 0.5))),
            MeshMaterial3d(palette.metal.clone()),
            Transform::from_xyz(x, 0.25, 9.4),
        ));
    }
}

/// Ordenadores (monitor + pantalla + teclado) sobre los pupitres del aula
/// de Informática (cx = 8).
fn spawn_computers(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    for dx in [-2.0, 2.0] {
        for dz in [-5.2, -3.6] {
            let x = 8.0 + dx;
            // Monitor.
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.46, 0.34, 0.05))),
                MeshMaterial3d(palette.metal.clone()),
                Transform::from_xyz(x, 0.92, dz),
            ));
            // Pantalla azulada (emisiva).
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.42, 0.30, 0.02))),
                MeshMaterial3d(palette.screen.clone()),
                Transform::from_xyz(x, 0.92, dz + 0.03),
            ));
            // Teclado.
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.42, 0.02, 0.16))),
                MeshMaterial3d(palette.metal.clone()),
                Transform::from_xyz(x, 0.73, dz + 0.12),
            ));
        }
    }
}

/// Libros de colores sobre los pupitres del aula de Matemáticas (cx = -8).
fn spawn_desk_books(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut Assets<StandardMaterial>,
) {
    let book_colors = [
        add_solid(materials, Color::srgb(0.80, 0.35, 0.30), 0.8),
        add_solid(materials, Color::srgb(0.30, 0.55, 0.85), 0.8),
        add_solid(materials, Color::srgb(0.35, 0.70, 0.40), 0.8),
        add_solid(materials, Color::srgb(0.85, 0.70, 0.25), 0.8),
    ];
    let mut color_index = 0;
    for dx in [-2.0, 2.0] {
        for dz in [-5.2, -3.6] {
            for bx in [-0.18, 0.18] {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.16, 0.12, 0.22))),
                    MeshMaterial3d(book_colors[color_index % 4].clone()),
                    Transform::from_xyz(-8.0 + dx + bx, 0.76, dz),
                ));
                color_index += 1;
            }
        }
    }
}

/// Corcho con notas de colores en la pared del pasillo.
fn spawn_corkboard_notes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut Assets<StandardMaterial>,
    palette: &Palette,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 0.9, 0.06))),
        MeshMaterial3d(palette.cork.clone()),
        Transform::from_xyz(6.0, 1.7, CLASSROOM_FRONT_Z + 0.2),
    ));
    let note_colors = [
        add_solid(materials, Color::srgb(0.95, 0.85, 0.40), 0.8),
        add_solid(materials, Color::srgb(0.90, 0.55, 0.50), 0.8),
        add_solid(materials, Color::srgb(0.50, 0.80, 0.95), 0.8),
        add_solid(materials, Color::srgb(0.60, 0.90, 0.60), 0.8),
        add_solid(materials, Color::srgb(0.85, 0.65, 0.90), 0.8),
    ];
    let note_positions = [
        (-0.45, 0.30),
        (0.00, 0.30),
        (0.45, 0.30),
        (-0.25, -0.25),
        (0.25, -0.25),
    ];
    for (i, (nx, ny)) in note_positions.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.22, 0.16, 0.02))),
            MeshMaterial3d(note_colors[i % 5].clone()),
            Transform::from_xyz(6.0 + nx, 1.7 + ny, CLASSROOM_FRONT_Z + 0.24),
        ));
    }
}

/// Tablón "BIENVENIDOS" sobre la entrada principal, con texto 3D.
fn spawn_welcome_sign(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
    font: &Handle<Font>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.6, 0.55, 0.08))),
        MeshMaterial3d(palette.frame.clone()),
        Transform::from_xyz(0.0, 2.6, RECEPTION_FRONT_Z - 0.3),
    ));
    commands.spawn((
        Text2d::new("BIENVENIDOS"),
        TextFont {
            font: font.clone(),
            font_size: 0.28,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 2.6, RECEPTION_FRONT_Z - 0.24),
    ));
}

/// Nubes blancas aplanadas flotando alrededor del colegio.
fn spawn_clouds(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let clouds = [
        (-30.0, 22.0, 13.0),
        (25.0, 20.0, 12.0),
        (-10.0, 24.0, 11.5),
        (35.0, 18.0, 12.5),
        (-38.0, 16.0, 12.0),
        (5.0, 21.0, 13.0),
        (-22.0, 19.0, 12.5),
        (30.0, 23.0, 12.0),
    ];
    for (x, z, y) in clouds {
        // Cada nube son tres esferas aplanadas superpuestas.
        for (dx, dy, radius) in [(-1.2, 0.0, 1.0), (1.2, 0.0, 1.0), (0.0, 0.6, 1.3)] {
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(radius))),
                MeshMaterial3d(palette.cloud.clone()),
                Transform::from_xyz(x + dx, y + dy, z).with_scale(Vec3::new(1.0, 0.55, 1.0)),
            ));
        }
    }
}

/// Arbustos redondos junto a la fachada delantera.
fn spawn_bushes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    for x in [-3.5, 3.5, -10.5, 10.5] {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.55))),
            MeshMaterial3d(palette.foliage.clone()),
            Transform::from_xyz(x, 0.55, 10.1),
        ));
    }
}

/// Detalles sobre el tejado: chimenea con caperuza y antena parabólica.
fn spawn_roof_details(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    // Chimenea (a la izquierda del tejado).
    spawn_solid(
        commands,
        meshes,
        &palette.planter,
        Vec3::new(-9.0, 3.9, 0.5),
        Vec3::new(0.7, 1.1, 0.7),
    );
    spawn_solid(
        commands,
        meshes,
        &palette.roof,
        Vec3::new(-9.0, 4.5, 0.5),
        Vec3::new(0.8, 0.08, 0.8),
    );
    // Antena parabólica (a la derecha del tejado).
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.06, 1.5))),
        MeshMaterial3d(palette.metal.clone()),
        Transform::from_xyz(9.0, 4.25, 0.5),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.5, 0.08))),
        MeshMaterial3d(palette.metal.clone()),
        Transform::from_rotation(Quat::from_rotation_x(-0.9))
            .with_translation(Vec3::new(9.0, 4.95, 0.5)),
    ));
}

/// Seto bajo que rodea el patio delantero, con un hueco para el camino.
fn spawn_hedge(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    let segments = [
        (0.0, -26.0, 54.0, 1.2),  // trasero
        (-27.0, 0.0, 1.2, 52.0),  // lateral izquierdo
        (27.0, 0.0, 1.2, 52.0),   // lateral derecho
        (-15.0, 26.0, 24.0, 1.2), // frente izquierdo (deja hueco central)
        (15.0, 26.0, 24.0, 1.2),  // frente derecho
    ];
    for (x, z, sx, sz) in segments {
        spawn_box(
            commands,
            meshes,
            &palette.foliage,
            Vec3::new(x, 0.4, z),
            Vec3::new(sx, 0.8, sz),
        );
    }
}

/// Mástil con la bandera del colegio junto a la entrada.
fn spawn_flagpole(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    let x = -6.5;
    let z = 12.5;
    // Poste.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.07, 5.2))),
        MeshMaterial3d(palette.metal.clone()),
        Transform::from_xyz(x, 2.6, z),
        Collider::new(Vec3::new(0.15, 2.6, 0.15)),
    ));
    // Esfera dorada en lo alto.
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.16))),
        MeshMaterial3d(palette.lamp.clone()),
        Transform::from_xyz(x, 5.35, z),
    ));
    // Bandera ondeando: dos paños para dar sensación de movimiento.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.7, 0.95, 0.04))),
        MeshMaterial3d(palette.banner.clone()),
        Transform::from_xyz(x + 0.88, 4.6, z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.55, 0.7, 0.04))),
        MeshMaterial3d(palette.banner.clone()),
        Transform::from_rotation(Quat::from_rotation_y(0.35))
            .with_translation(Vec3::new(x + 1.6, 4.45, z)),
    ));
}

/// Sol brillante en el cielo: esfera emisiva con rayos alrededor.
fn spawn_sun(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, palette: &Palette) {
    let pos = Vec3::new(55.0, 55.0, -35.0);
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(2.6))),
        MeshMaterial3d(palette.sun.clone()),
        Transform::from_translation(pos),
    ));
    // Rayos: cubos finos alrededor del sol.
    for (i, length) in (0..8).map(|i| (i, if i % 2 == 0 { 2.6 } else { 1.8 })).collect::<Vec<_>>() {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let offset = Vec3::new(angle.cos(), angle.sin() * 0.6, 0.0).normalize() * (3.2 + length * 0.4);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.18, 0.5))),
            MeshMaterial3d(palette.sun.clone()),
            Transform::from_translation(pos + offset),
        ));
    }
}

/// Macizos de flores a los lados del camino de entrada (tallo + cabeza).
fn spawn_flower_beds(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let flower_materials = [
        &palette.flower_red,
        &palette.flower_yellow,
        &palette.flower_purple,
    ];
    let mut index = 0;
    for side in [-2.6, 2.6] {
        for z in (11..24).step_by(2) {
            let z = z as f32;
            for dx in [-0.35, 0.0, 0.35] {
                let x = side + dx;
                commands.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.03, 0.4))),
                    MeshMaterial3d(palette.foliage.clone()),
                    Transform::from_xyz(x, 0.2, z),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.11))),
                    MeshMaterial3d(flower_materials[index % 3].clone()),
                    Transform::from_xyz(x, 0.48, z),
                ));
                index += 1;
            }
        }
    }
}

/// Paneles de luz de techo (emisivos) en cada aula, el pasillo y la
/// recepción: el interior se ve más "habitado" y con más vida.
fn spawn_ceiling_lights(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let lights = [
        (-8.0, -4.5), // Aula de Matemáticas.
        (0.0, -4.5),  // Aula de Historia.
        (8.0, -4.5),  // Aula de Informática.
        (0.0, 3.0),   // Pasillo.
        (0.0, 8.0),   // Recepción.
    ];
    for (x, z) in lights {
        // Carcasa metálica oscura (caja empotrada en el techo).
        spawn_box(
            commands,
            meshes,
            &palette.metal,
            Vec3::new(x, 3.12, z),
            Vec3::new(1.7, 0.08, 1.0),
        );
        // Panel luminoso emisivo.
        spawn_box(
            commands,
            meshes,
            &palette.panel,
            Vec3::new(x, 3.09, z),
            Vec3::new(1.5, 0.04, 0.8),
        );
    }
}

/// Taquillas de colores a lo largo de la pared izquierda del pasillo:
/// dos filas de taquillas, cada una con su color de asignatura.
fn spawn_lockers(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let locker_x = -11.55; // Interior de la pared izquierda (x = -12).
    let colors = [
        &palette.math_accent,
        &palette.history_accent,
        &palette.cs_accent,
        &palette.metal,
    ];
    let mut index = 0;
    for z in (12..=84).step_by(12) {
        let z = z as f32 / 10.0;
        for row in 0..2 {
            let y = 0.55 + row as f32 * 0.85;
            spawn_box(
                commands,
                meshes,
                colors[index % colors.len()],
                Vec3::new(locker_x, y, z),
                Vec3::new(0.6, 0.8, 0.38),
            );
            index += 1;
        }
    }
}

/// Carteles decorativos en la cara interior (aulas) de la pared frontal,
/// entre los vanos de las puertas.
fn spawn_classroom_posters(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let posters = [
        (-10.4, &palette.history_accent),
        (-4.0, &palette.cs_accent),
        (-3.0, &palette.math_accent),
        (3.0, &palette.cs_accent),
        (5.0, &palette.math_accent),
        (11.0, &palette.history_accent),
    ];
    for (x, color) in posters {
        spawn_box(
            commands,
            meshes,
            color,
            Vec3::new(x, 2.15, CLASSROOM_FRONT_Z - 0.2),
            Vec3::new(0.9, 0.6, 0.04),
        );
    }
}

/// Canasta de baloncesto en la parte trasera del colegio: poste con
/// colisión, tablero y aro (toro horizontal).
fn spawn_basketball_hoop(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    let x = 0.0;
    let z = -13.5;
    // Poste.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.09, 3.1))),
        MeshMaterial3d(palette.metal.clone()),
        Transform::from_xyz(x, 1.55, z),
        Collider::new(Vec3::new(0.12, 1.55, 0.12)),
    ));
    // Tablero.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2, 0.75, 0.05))),
        MeshMaterial3d(palette.glass.clone()),
        Transform::from_xyz(x, 2.75, z + 0.8),
    ));
    // Aro (toro horizontal, como el de verdad).
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(0.24, 0.035))),
        MeshMaterial3d(palette.metal.clone()),
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(x, 3.0, z + 1.1)),
    ));
}

/// Dos porterías de fútbol en el césped delantero: postes, larguero y
/// marco trasero, en blanco.
fn spawn_soccer_goals(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    palette: &Palette,
) {
    for gx in [-14.0, 14.0] {
        let z = 21.0;
        // Postes verticales.
        for dx in [-0.8, 0.8] {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.06, 2.0))),
                MeshMaterial3d(palette.goal.clone()),
                Transform::from_xyz(gx + dx, 1.0, z),
                Collider::new(Vec3::new(0.08, 1.0, 0.08)),
            ));
        }
        // Larguero.
        spawn_solid(
            commands,
            meshes,
            &palette.goal,
            Vec3::new(gx, 2.0, z),
            Vec3::new(1.7, 0.08, 0.08),
        );
        // Marco trasero (el "fondo" de la portería, sin red).
        spawn_solid(
            commands,
            meshes,
            &palette.goal,
            Vec3::new(gx, 1.0, z + 0.9),
            Vec3::new(1.7, 0.08, 0.08),
        );
        for dx in [-0.8, 0.8] {
            spawn_solid(
                commands,
                meshes,
                &palette.goal,
                Vec3::new(gx + dx, 1.0, z + 0.9),
                Vec3::new(0.08, 2.0, 0.08),
            );
        }
    }
}