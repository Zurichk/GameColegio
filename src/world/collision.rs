//! Colisiones AABB simples para el prototipo.
//!
//! No se utiliza un motor de física externo: cada objeto sólido declara su
//! caja de colisión (`Collider`) y el sistema de movimiento del jugador
//! resuelve las penetraciones eje a eje.

use bevy::prelude::*;

/// Caja de colisión axis-aligned (AABB) de una entidad.
#[derive(Component, Debug, Clone, Copy)]
pub struct Collider {
    pub half_extents: Vec3,
}

impl Collider {
    /// Crea un collider a partir de sus semiextensiones.
    pub fn new(half_extents: Vec3) -> Self {
        Self { half_extents }
    }
}

/// AABB en coordenadas del mundo definida por sus esquinas.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Construye una AABB a partir de su centro y semiextensiones.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Comprueba si esta caja solapa con otra (sin incluir bordes).
    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
}

/// Resuelve la posición candidata de una caja contra un conjunto de AABBs
/// estáticas, eje a eje, para que el jugador se deslice por las paredes en
/// lugar de quedarse atascado.
///
/// * `original`: posición antes del movimiento.
/// * `candidate`: posición propuesta tras aplicar el movimiento.
/// * `half_extents`: semiextensiones de la caja móvil.
/// * `aabbs`: cajas estáticas del mundo.
pub fn resolve_aabbs(
    original: Vec3,
    mut candidate: Vec3,
    half_extents: Vec3,
    aabbs: &[Aabb],
) -> Vec3 {
    for axis in 0..3 {
        // Se prueba el movimiento en un único eje manteniendo los demás en
        // su posición original, para detectar el choque correctamente.
        let mut probe = original;
        probe[axis] = candidate[axis];

        for aabb in aabbs {
            let player = Aabb::from_center_half_extents(probe, half_extents);
            if player.overlaps(aabb) {
                // El lado hacia el que empujar se deduce de la posición
                // original en este eje.
                if original[axis] < aabb.min[axis] {
                    probe[axis] = aabb.min[axis] - half_extents[axis];
                } else {
                    probe[axis] = aabb.max[axis] + half_extents[axis];
                }
            }
        }
        candidate[axis] = probe[axis];
    }
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_overlaps_detects_overlap() {
        let a = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));
        let b = Aabb::from_center_half_extents(Vec3::new(1.5, 0.0, 0.0), Vec3::splat(1.0));
        assert!(a.overlaps(&b));
    }

    #[test]
    fn aabb_overlaps_is_false_when_separated() {
        let a = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));
        let b = Aabb::from_center_half_extents(Vec3::new(3.0, 0.0, 0.0), Vec3::splat(1.0));
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn resolve_aabbs_pushes_out_of_wall() {
        let half = Vec3::splat(0.5);
        let wall = Aabb::from_center_half_extents(
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(1.0, 0.5, 0.3),
        );
        let original = Vec3::new(-3.0, 0.5, 0.0);
        let candidate = Vec3::new(-0.2, 0.5, 0.0);

        let resolved = resolve_aabbs(original, candidate, half, &[wall]);

        assert!(resolved.x <= -1.5, "el jugador debe quedar fuera de la pared");
    }

    #[test]
    fn resolve_aabbs_allows_sliding_along_wall() {
        let half = Vec3::splat(0.5);
        // Pared orientada a lo largo del eje Z.
        let wall = Aabb::from_center_half_extents(
            Vec3::new(0.0, 0.5, 0.0),
            Vec3::new(0.3, 0.5, 10.0),
        );
        let original = Vec3::new(-2.0, 0.5, 0.0);
        // Intenta atravesar la pared y desplazarse en Z al mismo tiempo.
        let candidate = Vec3::new(0.2, 0.5, 3.0);

        let resolved = resolve_aabbs(original, candidate, half, &[wall]);

        assert!(resolved.x <= -0.8, "el movimiento en X debe bloquearse");
        assert!(
            (resolved.z - 3.0).abs() < 1e-4,
            "el movimiento en Z debe conservarse (deslizamiento)"
        );
    }
}