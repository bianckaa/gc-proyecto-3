use super::super::*;

fn sphere_lod(
    subdivisions: [u32; 3],
    deformation: Option<Deformation>,
) -> impl FnMut(usize) -> Mesh {
    move |level| {
        let mut mesh = Mesh::icosphere(subdivisions[level]);
        if let Some(deformation) = deformation {
            mesh.deform(&deformation);
        }
        mesh
    }
}

pub(in crate::game) fn load_models(assets: &mut AssetManager, system: &SolarSystem) -> Models {
    let sphere =
        Model::Lod(assets.lod_or_insert_with("esfera", sphere_lod(SPHERE_SUBDIVISIONS, None)));
    let bodies: Vec<Model> = (0..system.body_count())
        .map(|index| match system.body(index).model.file_name() {
            Some(file_name) => Model::Mesh(
                assets.load_model_or_else(file_name, || Mesh::icosphere(SPHERE_SUBDIVISIONS[0])),
            ),
            None => sphere,
        })
        .collect();
    let run_planets = [
        bodies[system.find("ROCOSO").unwrap()],
        bodies[system.find("GASEOSO").unwrap()],
        bodies[system.find("NUCLEO").unwrap()],
        bodies[system.find("DROIDE").unwrap()],
    ];
    Models {
        bodies,
        ship: assets.load_ship(),
        enemy: assets.load_model_or_else(ENEMY_MODEL_FILE, || turned_around(placeholder_ship())),
        asteroid: Model::Lod(assets.lod_or_insert_with(
            "asteroide",
            sphere_lod(ASTEROID_SUBDIVISIONS, Some(ASTEROID_DEFORMATION)),
        )),
        crystal: Model::Mesh(assets.optional_model_or_else(SURFACE_CRYSTAL_FILE, crystal)),
        boulder: Model::Lod(assets.lod_or_insert_with(
            "roca_superficie",
            sphere_lod(ASTEROID_SUBDIVISIONS, Some(BOULDER_DEFORMATION)),
        )),
        run_planets,
    }
}

pub(in crate::game) fn patrols(system: &SolarSystem) -> Vec<PatrolEnemy> {
    let plan: [(&str, f32, f32, f32, f32); 6] = [
        ("ROCOSO", 2.4, 0.42, 0.0, 0.5),
        ("ROCOSO", 3.1, -0.3, 2.5, -0.9),
        ("GASEOSO", 2.2, 0.28, 1.0, 0.3),
        ("GASEOSO", 2.8, -0.22, 4.0, -0.6),
        ("DROIDE", 1.9, 0.25, 0.5, 0.15),
        ("NUCLEO", 2.6, 0.35, 3.3, 1.1),
    ];
    plan.iter()
        .filter_map(|&(name, distance, speed, phase, tilt)| {
            let host = system.find(name)?;
            let radius = system.body(host).radius * distance;
            Some(PatrolEnemy {
                host,
                orbit: Orbit::circular(radius, speed, phase).with_tilt(tilt, phase),
                scale: ENEMY_SCALE,
            })
        })
        .collect()
}

pub(in crate::game) fn surfaces(system: &SolarSystem) -> Vec<Surface> {
    (0..system.body_count())
        .filter_map(|body| {
            let config = system.body(body).surface?;
            Some(Surface {
                body,
                props: scatter_surface(&config),
                embed_depth: config.embed_depth,
            })
        })
        .collect()
}

pub(in crate::game) fn apply_run_seed(config: &mut SystemConfig, seed: u32) {
    config.belt.seed = mix_seed(seed, 0);
    for (index, body) in config.bodies.iter_mut().enumerate() {
        if let Some(surface) = &mut body.surface {
            // Surface noise converts this value to spatial coordinates, so keep
            // it bounded to avoid overflowing its integer lattice.
            surface.seed = mix_seed(seed, index as u32 + 1) % 1_000_000;
        }
    }
}

fn mix_seed(seed: u32, stream: u32) -> u32 {
    let mut value = seed ^ stream.wrapping_mul(0x9E37_79B9);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^ (value >> 16)
}
