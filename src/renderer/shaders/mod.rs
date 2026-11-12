use super::lighting::PointLight;
use super::normal_map::NormalMap;
use super::rasterizer::Fragment;
use super::Material;
use crate::engine::math::{Vec2, Vec3};
use std::sync::OnceLock;

const FBM_OCTAVES: u32 = 4;
const SUN_MAX_DITHER_INTENSITY: f32 = 0.86;
const GAS_NIGHT_DITHER_INTENSITY: f32 = 0.28;
const ROCKY_CRATER_COUNT: usize = 18;
const GOLDEN_ANGLE: f32 = 2.399_963_1;
static ROCKY_CRATERS: OnceLock<[(Vec3, f32); ROCKY_CRATER_COUNT]> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub color: Vec3,
    pub intensity: f32,
}

pub struct ShadeInput<'a> {
    pub fragment: &'a Fragment,
    pub material: &'a Material,
    pub light: &'a PointLight,
    pub time: f32,
    pub normal_map: Option<&'a NormalMap>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderKind {
    CorePlanet,
    DroidPlanet,
    EnemyShip,
    EtaPlayer,
    GasPlanet,
    Lambert,
    Rocky,
    RockyPlanet,
    Lava,
    Toon,
    GasGiant,
    Sun,
}

impl ShaderKind {
    pub fn uses_normal_map(self) -> bool {
        matches!(self, ShaderKind::Rocky)
    }
}

pub fn shade(input: &ShadeInput) -> Surface {
    let mapped_fragment;
    let mapped_input;
    let input = match input.normal_map {
        Some(map) if input.material.shader.uses_normal_map() => {
            mapped_fragment = Fragment {
                normal: map.perturb(input.fragment),
                ..*input.fragment
            };
            mapped_input = ShadeInput {
                fragment: &mapped_fragment,
                material: input.material,
                light: input.light,
                time: input.time,
                normal_map: None,
            };
            &mapped_input
        }
        _ => input,
    };
    let surface = match input.material.shader {
        ShaderKind::CorePlanet => shade_core_planet(input),
        ShaderKind::DroidPlanet => shade_droid_planet(input),
        ShaderKind::EnemyShip => shade_enemy_ship(input),
        ShaderKind::EtaPlayer => shade_eta_player(input),
        ShaderKind::GasPlanet => shade_gas_planet(input),
        ShaderKind::Lambert => shade_lambert(input),
        ShaderKind::Rocky => shade_rocky(input),
        ShaderKind::RockyPlanet => shade_rocky_planet(input),
        ShaderKind::Lava => shade_lava(input),
        ShaderKind::Toon => shade_toon(input),
        ShaderKind::GasGiant => shade_gas_giant(input),
        ShaderKind::Sun => shade_sun(input),
    };
    Surface {
        color: surface.color + input.material.emissive,
        ..surface
    }
}

mod common;
mod core;
mod droid;
mod enemy_ship;
mod gas_giant;
mod gas_planet;
mod lava;
mod mapping;
mod player_ship;
mod rocky;
mod rocky_surface;
mod sun;
mod toon;

use common::{box_mask, hash, lit_intensity, mix, shade_lambert, smoothstep};
pub use common::{fbm, value_noise};
use core::shade_core_planet;
use droid::shade_droid_planet;
use enemy_ship::shade_enemy_ship;
use gas_giant::shade_gas_giant;
use gas_planet::shade_gas_planet;
use lava::shade_lava;
use mapping::{core_projection, panel_sample};
use player_ship::shade_eta_player;
use rocky::shade_rocky_planet;
use rocky_surface::shade_rocky;
use sun::shade_sun;
use toon::shade_toon;
