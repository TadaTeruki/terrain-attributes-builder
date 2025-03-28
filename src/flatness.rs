use worley_particle::map::{
    grad::{GradDifferenceType, GradStrategy},
    lerp::InterpolationMethod,
    IDWStrategy, ParticleMap,
};

// fn gradient_to_flatness(gradient: f64) -> Option<f64> {
//     let flatness = 1.0 - gradient.abs() / 5.0;
//     if flatness < 0.0 {
//         return None;
//     }
//     Some(flatness.sqrt())
// }

pub struct FlatnessMap {
    pub particle_map: ParticleMap<f64>,
}

impl FlatnessMap {
    pub fn new(
        elevation_map: &ParticleMap<f64>,
        minimum_neighbor_num: usize,
        sea_level: f64,
        gradient_to_flatness: impl Fn(f64) -> Option<f64>,
    ) -> Self {
        let particle_map = build_flatness_map(
            elevation_map,
            minimum_neighbor_num,
            sea_level,
            gradient_to_flatness,
        );
        Self { particle_map }
    }

    pub fn map(&self) -> &ParticleMap<f64> {
        &self.particle_map
    }
}

fn build_flatness_map(
    elevation_map: &ParticleMap<f64>,
    minimum_neighbor_num: usize,
    sea_level: f64,
    gradient_to_flatness: impl Fn(f64) -> Option<f64>,
) -> ParticleMap<f64> {
    let mut flatness_map = elevation_map
        .iter()
        .filter_map(|(particle, elevation)| {
            if *elevation < sea_level {
                return None;
            }
            let (x, y) = particle.site();
            let gradient = elevation_map.get_gradient(
                x,
                y,
                &GradStrategy {
                    delta: elevation_map.params().scale,
                    difference_type: GradDifferenceType::Central,
                    ..Default::default()
                },
                &InterpolationMethod::IDW(IDWStrategy::default_from_params(elevation_map.params())),
            )?;
            let habitability = gradient_to_flatness(gradient.value)?;
            Some((*particle, habitability))
        })
        .collect::<ParticleMap<f64>>();

    if minimum_neighbor_num > 0 {
        flatness_map = flatness_map
            .iter()
            .filter(|(particle, _)| {
                let surrounding_particles = particle.calculate_voronoi().neighbors;
                let count = surrounding_particles
                    .iter()
                    .filter(|neighbor| flatness_map.get(neighbor).is_some())
                    .count();

                count >= minimum_neighbor_num
            })
            .map(|(particle, flatness)| (*particle, *flatness))
            .collect::<ParticleMap<f64>>();
    }

    flatness_map
}
