//! Plant genotype representation for evolutionary L-systems.
//!
//! This module provides `PlantGenotype`, a wrapper around L-system source code
//! and material settings that implements the `Genotype` trait from symbios-genetics.
//!
//! The key design principle is that the **source code is the single source of truth**.
//! Mutations operate on the compiled System, but the results are decompiled back to
//! source code after each operation.

use bevy::platform::collections::HashMap;
use bevy_symbios::materials::{MaterialSettings, TextureType};
use bevy_symbios_texture::TextureConfig;
use rand::Rng;
use serde::{Deserialize, Serialize};
use symbios::System;
use symbios::system::crossover::CrossoverConfig;
use symbios::system::mutate::{MutationConfig, StructuralMutationConfig};
use symbios_genetics::Genotype;

/// Mutates the active variant of a [`TextureConfig`] in place via its
/// [`Genotype`] impl. No-op for [`TextureConfig::None`].
fn mutate_texture_config<R: Rng>(cfg: &mut TextureConfig, rng: &mut R, rate: f32) {
    match cfg {
        TextureConfig::None => {}
        TextureConfig::Leaf(c) => c.mutate(rng, rate),
        TextureConfig::Twig(c) => c.mutate(rng, rate),
        TextureConfig::Bark(c) => c.mutate(rng, rate),
        TextureConfig::Window(c) => c.mutate(rng, rate),
        TextureConfig::StainedGlass(c) => c.mutate(rng, rate),
        TextureConfig::IronGrille(c) => c.mutate(rng, rate),
        TextureConfig::Ground(c) => c.mutate(rng, rate),
        TextureConfig::Rock(c) => c.mutate(rng, rate),
        TextureConfig::Brick(c) => c.mutate(rng, rate),
        TextureConfig::Plank(c) => c.mutate(rng, rate),
        TextureConfig::Shingle(c) => c.mutate(rng, rate),
        TextureConfig::Stucco(c) => c.mutate(rng, rate),
        TextureConfig::Concrete(c) => c.mutate(rng, rate),
        TextureConfig::Metal(c) => c.mutate(rng, rate),
        TextureConfig::Pavers(c) => c.mutate(rng, rate),
        TextureConfig::Ashlar(c) => c.mutate(rng, rate),
        TextureConfig::Cobblestone(c) => c.mutate(rng, rate),
        TextureConfig::Thatch(c) => c.mutate(rng, rate),
        TextureConfig::Marble(c) => c.mutate(rng, rate),
        TextureConfig::Corrugated(c) => c.mutate(rng, rate),
        TextureConfig::Asphalt(c) => c.mutate(rng, rate),
        TextureConfig::Wainscoting(c) => c.mutate(rng, rate),
        TextureConfig::Encaustic(c) => c.mutate(rng, rate),
    }
}

/// Crosses over two [`TextureConfig`] values when both are the same generator
/// variant, otherwise returns a clone of one parent.
fn crossover_texture_config<R: Rng>(
    a: &TextureConfig,
    b: &TextureConfig,
    rng: &mut R,
) -> TextureConfig {
    match (a, b) {
        (TextureConfig::Leaf(x), TextureConfig::Leaf(y)) => {
            TextureConfig::Leaf(x.crossover(y, rng))
        }
        (TextureConfig::Twig(x), TextureConfig::Twig(y)) => {
            TextureConfig::Twig(x.crossover(y, rng))
        }
        (TextureConfig::Bark(x), TextureConfig::Bark(y)) => {
            TextureConfig::Bark(x.crossover(y, rng))
        }
        (TextureConfig::Window(x), TextureConfig::Window(y)) => {
            TextureConfig::Window(x.crossover(y, rng))
        }
        (TextureConfig::StainedGlass(x), TextureConfig::StainedGlass(y)) => {
            TextureConfig::StainedGlass(x.crossover(y, rng))
        }
        (TextureConfig::IronGrille(x), TextureConfig::IronGrille(y)) => {
            TextureConfig::IronGrille(x.crossover(y, rng))
        }
        (TextureConfig::Ground(x), TextureConfig::Ground(y)) => {
            TextureConfig::Ground(x.crossover(y, rng))
        }
        (TextureConfig::Rock(x), TextureConfig::Rock(y)) => {
            TextureConfig::Rock(x.crossover(y, rng))
        }
        (TextureConfig::Brick(x), TextureConfig::Brick(y)) => {
            TextureConfig::Brick(x.crossover(y, rng))
        }
        (TextureConfig::Plank(x), TextureConfig::Plank(y)) => {
            TextureConfig::Plank(x.crossover(y, rng))
        }
        (TextureConfig::Shingle(x), TextureConfig::Shingle(y)) => {
            TextureConfig::Shingle(x.crossover(y, rng))
        }
        (TextureConfig::Stucco(x), TextureConfig::Stucco(y)) => {
            TextureConfig::Stucco(x.crossover(y, rng))
        }
        (TextureConfig::Concrete(x), TextureConfig::Concrete(y)) => {
            TextureConfig::Concrete(x.crossover(y, rng))
        }
        (TextureConfig::Metal(x), TextureConfig::Metal(y)) => {
            TextureConfig::Metal(x.crossover(y, rng))
        }
        (TextureConfig::Pavers(x), TextureConfig::Pavers(y)) => {
            TextureConfig::Pavers(x.crossover(y, rng))
        }
        (TextureConfig::Ashlar(x), TextureConfig::Ashlar(y)) => {
            TextureConfig::Ashlar(x.crossover(y, rng))
        }
        (TextureConfig::Cobblestone(x), TextureConfig::Cobblestone(y)) => {
            TextureConfig::Cobblestone(x.crossover(y, rng))
        }
        (TextureConfig::Thatch(x), TextureConfig::Thatch(y)) => {
            TextureConfig::Thatch(x.crossover(y, rng))
        }
        (TextureConfig::Marble(x), TextureConfig::Marble(y)) => {
            TextureConfig::Marble(x.crossover(y, rng))
        }
        (TextureConfig::Corrugated(x), TextureConfig::Corrugated(y)) => {
            TextureConfig::Corrugated(x.crossover(y, rng))
        }
        (TextureConfig::Asphalt(x), TextureConfig::Asphalt(y)) => {
            TextureConfig::Asphalt(x.crossover(y, rng))
        }
        (TextureConfig::Wainscoting(x), TextureConfig::Wainscoting(y)) => {
            TextureConfig::Wainscoting(x.crossover(y, rng))
        }
        (TextureConfig::Encaustic(x), TextureConfig::Encaustic(y)) => {
            TextureConfig::Encaustic(x.crossover(y, rng))
        }
        // Mismatched variants (or None): pick one parent.
        _ => a.clone(),
    }
}

/// Blends two [`TextureType`] values for crossover.
///
/// Picks one parent's variant by `blend` weight; if both share the same
/// procedural generator, the inner config is also crossed over.
fn blend_texture_type<R: Rng>(
    a: &TextureType,
    b: &TextureType,
    blend: f32,
    rng: &mut R,
) -> TextureType {
    match (a, b) {
        (TextureType::Procedural(ca), TextureType::Procedural(cb)) => {
            TextureType::Procedural(crossover_texture_config(ca, cb, rng))
        }
        _ => {
            if blend >= 0.5 {
                a.clone()
            } else {
                b.clone()
            }
        }
    }
}

use crate::core::config::{PropMeshType, scan_max_material_id, split_source_code};
use crate::core::presets::LSystemPreset;

/// Serializable version of material settings for genetic storage.
///
/// `texture_type` now carries the active generator config inline (via
/// [`TextureType::Procedural`]) — no separate per-generator fields.
#[derive(Clone, Serialize, Deserialize)]
pub struct SerializableMaterial {
    pub base_color: [f32; 3],
    pub emission_color: [f32; 3],
    pub emission_strength: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub uv_scale: f32,
    #[serde(default)]
    pub texture_type: TextureType,
}

impl Default for SerializableMaterial {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0],
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            roughness: 0.5,
            metallic: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        }
    }
}

impl From<&MaterialSettings> for SerializableMaterial {
    fn from(m: &MaterialSettings) -> Self {
        Self {
            base_color: m.base_color,
            emission_color: m.emission_color,
            emission_strength: m.emission_strength,
            roughness: m.roughness,
            metallic: m.metallic,
            uv_scale: m.uv_scale,
            texture_type: m.texture.clone(),
        }
    }
}

impl SerializableMaterial {
    /// Converts back to full [`MaterialSettings`], preserving texture type and inline configs.
    pub fn to_material_settings(&self) -> MaterialSettings {
        MaterialSettings {
            base_color: self.base_color,
            emission_color: self.emission_color,
            emission_strength: self.emission_strength,
            roughness: self.roughness,
            metallic: self.metallic,
            texture: self.texture_type.clone(),
            uv_scale: self.uv_scale,
        }
    }
}

/// A plant genotype encoding an L-system with material settings.
///
/// This struct wraps the L-system source code and associated configuration,
/// implementing genetic operators (mutation, crossover) that maintain the
/// source code as the single source of truth.
#[derive(Clone, Serialize, Deserialize)]
pub struct PlantGenotype {
    /// The growth phase L-system source code (single source of truth).
    pub source_code: String,
    /// Optional finalization/decomposition code for two-pass derivation.
    pub finalization_code: String,
    /// Material settings by slot ID (serializable).
    pub materials: HashMap<u16, SerializableMaterial>,
    /// Number of derivation iterations.
    pub iterations: usize,
    /// Default turn angle in degrees.
    pub angle: f32,
    /// Step size for forward movement.
    pub step: f32,
    /// Default branch width.
    pub width: f32,
    /// Elasticity factor for tropism bending.
    pub elasticity: f32,
    /// Tropism direction vector stored as \[x, y, z\] for serialization.
    pub tropism: Option<[f32; 3]>,
    /// Random seed for stochastic rules.
    pub seed: u64,
    /// Prop ID to mesh type mapping, persisted so nursery champions retain their prop visuals.
    #[serde(default)]
    pub prop_mappings: HashMap<u16, PropMeshType>,
}

impl PlantGenotype {
    /// Creates a new PlantGenotype from source code with default settings.
    pub fn new(source_code: String) -> Self {
        Self {
            source_code,
            finalization_code: String::new(),
            materials: HashMap::new(),
            iterations: 4,
            angle: 25.0,
            step: 1.0,
            width: 0.1,
            elasticity: 0.0,
            tropism: None,
            seed: 42,
            prop_mappings: HashMap::new(),
        }
    }

    /// Creates a PlantGenotype with finalization code for two-pass derivation.
    pub fn with_finalization(mut self, finalization_code: String) -> Self {
        self.finalization_code = finalization_code;
        self
    }

    /// Sets the material settings from a MaterialSettings HashMap.
    pub fn with_materials(mut self, materials: &HashMap<u16, MaterialSettings>) -> Self {
        self.materials = materials
            .iter()
            .map(|(&k, v)| (k, SerializableMaterial::from(v)))
            .collect();
        self
    }

    /// Sets derivation parameters.
    pub fn with_params(mut self, iterations: usize, angle: f32, step: f32, width: f32) -> Self {
        self.iterations = iterations;
        self.angle = angle;
        self.step = step;
        self.width = width;
        self
    }

    /// Sets the random seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Creates a PlantGenotype from a static LSystemPreset.
    ///
    /// This converts a preset's configuration into an evolvable genotype,
    /// allowing presets to be injected into the nursery for breeding.
    pub fn from_preset(preset: &LSystemPreset) -> Self {
        let (growth, finalization) = split_source_code(preset.code);

        // Convert preset materials to serializable format
        let materials: HashMap<u16, SerializableMaterial> = preset
            .materials
            .iter()
            .map(|(slot, mat)| {
                (
                    *slot,
                    SerializableMaterial {
                        base_color: mat.base_color,
                        emission_color: mat.emission_color,
                        emission_strength: mat.emission_strength,
                        roughness: mat.roughness,
                        metallic: mat.metallic,
                        uv_scale: mat.uv_scale,
                        texture_type: (mat.texture)(),
                    },
                )
            })
            .collect();

        Self {
            source_code: growth,
            finalization_code: finalization,
            materials,
            iterations: preset.iterations,
            angle: preset.angle,
            step: preset.step,
            width: preset.width,
            elasticity: preset.elasticity,
            tropism: preset.tropism.map(|v| [v.x, v.y, v.z]),
            seed: 42,
            prop_mappings: preset.prop_meshes.iter().copied().collect(),
        }
    }

    /// Returns materials converted to MaterialSettings.
    pub fn get_material_settings(&self) -> HashMap<u16, MaterialSettings> {
        self.materials
            .iter()
            .map(|(&k, v)| (k, v.to_material_settings()))
            .collect()
    }

    /// Parses the source code into a System.
    ///
    /// Returns None if parsing fails.
    pub fn parse(&self) -> Option<System> {
        System::from_source(&self.source_code).ok()
    }

    /// Mutates material colors and the active procedural texture config.
    fn mutate_materials<R: Rng>(&mut self, rng: &mut R, rate: f32) {
        for settings in self.materials.values_mut() {
            if rng.random::<f32>() < rate {
                for channel in &mut settings.base_color {
                    *channel = (*channel + (rng.random::<f32>() - 0.5) * 0.3).clamp(0.0, 1.0);
                }
            }
            if rng.random::<f32>() < rate * 0.5 {
                settings.roughness =
                    (settings.roughness + (rng.random::<f32>() - 0.5) * 0.3).clamp(0.0, 1.0);
            }
            // Mutate the active procedural-texture config via its Genotype impl.
            if let TextureType::Procedural(cfg) = &mut settings.texture_type {
                mutate_texture_config(cfg, rng, rate);
            }
        }
    }

    /// Blends materials from two parents using foliage config crossover.
    fn blend_materials<R: Rng>(
        a: &HashMap<u16, SerializableMaterial>,
        b: &HashMap<u16, SerializableMaterial>,
        blend: f32,
        rng: &mut R,
    ) -> HashMap<u16, SerializableMaterial> {
        let mut result = HashMap::new();

        let all_slots: std::collections::HashSet<_> = a.keys().chain(b.keys()).copied().collect();

        for slot in all_slots {
            let settings = match (a.get(&slot), b.get(&slot)) {
                (Some(ma), Some(mb)) => {
                    let inv_blend = 1.0 - blend;
                    SerializableMaterial {
                        base_color: [
                            ma.base_color[0] * blend + mb.base_color[0] * inv_blend,
                            ma.base_color[1] * blend + mb.base_color[1] * inv_blend,
                            ma.base_color[2] * blend + mb.base_color[2] * inv_blend,
                        ],
                        roughness: ma.roughness * blend + mb.roughness * inv_blend,
                        metallic: ma.metallic * blend + mb.metallic * inv_blend,
                        emission_color: [
                            ma.emission_color[0] * blend + mb.emission_color[0] * inv_blend,
                            ma.emission_color[1] * blend + mb.emission_color[1] * inv_blend,
                            ma.emission_color[2] * blend + mb.emission_color[2] * inv_blend,
                        ],
                        emission_strength: ma.emission_strength * blend
                            + mb.emission_strength * inv_blend,
                        uv_scale: ma.uv_scale * blend + mb.uv_scale * inv_blend,
                        // Pick a parent's texture variant; if both are the same procedural
                        // generator, blend their configs via the Genotype crossover impl.
                        texture_type: blend_texture_type(
                            &ma.texture_type,
                            &mb.texture_type,
                            blend,
                            rng,
                        ),
                    }
                }
                (Some(m), None) | (None, Some(m)) => m.clone(),
                (None, None) => unreachable!(),
            };
            result.insert(slot, settings);
        }

        result
    }
}

impl Genotype for PlantGenotype {
    fn mutate<R: Rng>(&mut self, rng: &mut R, rate: f32) {
        // Skip mutation if rate is too low
        if rate <= 0.0 {
            return;
        }

        // Parse the source into a System
        let Some(mut system) = self.parse() else {
            return;
        };

        // Apply parametric mutations (probabilities and constants)
        let mutation_config = MutationConfig {
            rule_probability_rate: rate as f64,
            rule_probability_strength: 0.2,
            constant_rate: rate as f64,
            constant_strength: 0.3,
            gaussian_jitter_scale: 0.4,
            gaussian_jitter_rate: rate as f64,
        };
        system.mutate_with_rng(rng, &mutation_config);

        // Apply structural mutations at a lower rate
        if rng.random::<f32>() < rate * 0.5 {
            let structural_config = StructuralMutationConfig {
                successor_rate: rate as f64 * 0.3,
                insert_rate: 0.1,
                delete_rate: 0.1,
                swap_rate: 0.2,
                bytecode_rate: rate as f64 * 0.2,
                op_rate: 0.1,
                push_perturbation: 0.5,
            };
            system.structural_mutate_with_rng(rng, &structural_config);
        }

        // Reconstruct source from mutated system
        self.source_code = system.to_source();

        // Mutate finalization code if present
        if !self.finalization_code.trim().is_empty()
            && let Ok(mut fin_system) = System::from_source(&self.finalization_code)
        {
            fin_system.mutate_with_rng(rng, &mutation_config);
            self.finalization_code = fin_system.to_source();
        }

        // Ensure materials map covers all material IDs referenced in source
        let max_id = scan_max_material_id(&self.source_code)
            .max(scan_max_material_id(&self.finalization_code));
        for id in 0..=max_id {
            self.materials
                .entry(id)
                .or_insert_with(|| SerializableMaterial {
                    base_color: [
                        rng.random::<f32>(),
                        rng.random::<f32>(),
                        rng.random::<f32>(),
                    ],
                    roughness: 0.3 + rng.random::<f32>() * 0.5,
                    ..SerializableMaterial::default()
                });
        }

        // Mutate materials
        self.mutate_materials(rng, rate);

        // Occasionally mutate parameters
        if rng.random::<f32>() < rate * 0.3 {
            self.angle = (self.angle + (rng.random::<f32>() - 0.5) * 10.0).clamp(5.0, 90.0);
        }
        if rng.random::<f32>() < rate * 0.2 {
            self.step = (self.step * (0.9 + rng.random::<f32>() * 0.2)).clamp(0.1, 10.0);
        }
        if rng.random::<f32>() < rate * 0.2 {
            self.width = (self.width * (0.9 + rng.random::<f32>() * 0.2)).clamp(0.01, 1.0);
        }

        // Mutate elasticity
        if rng.random::<f32>() < rate * 0.2 {
            self.elasticity = (self.elasticity + (rng.random::<f32>() - 0.5) * 0.2).clamp(0.0, 1.0);
        }

        // Mutate tropism vector
        if rng.random::<f32>() < rate * 0.2 {
            let t = self.tropism.get_or_insert([0.0, -1.0, 0.0]);
            for component in t.iter_mut() {
                *component += (rng.random::<f32>() - 0.5) * 0.3;
            }
        }

        // Mutate seed for different stochastic outcomes
        if rng.random::<f32>() < rate {
            self.seed = rng.random::<u64>();
        }
    }

    fn crossover<R: Rng>(&self, other: &Self, rng: &mut R) -> Self {
        // Parse both parents
        let system_a = match self.parse() {
            Some(s) => s,
            None => return self.clone(),
        };
        let system_b = match other.parse() {
            Some(s) => s,
            None => return self.clone(),
        };

        // Perform crossover using symbios
        let crossover_config = CrossoverConfig {
            rule_bias: 0.5,
            constant_blend: rng.random::<f64>(),
        };

        let offspring_system = match system_a.crossover_with_rng(&system_b, rng, &crossover_config)
        {
            Ok(s) => s,
            Err(_) => return self.clone(),
        };

        // Reconstruct source from offspring
        let source_code = offspring_system.to_source();

        // Blend parameters
        let blend = rng.random::<f32>();
        let inv_blend = 1.0 - blend;

        PlantGenotype {
            source_code,
            finalization_code: if rng.random::<bool>() {
                self.finalization_code.clone()
            } else {
                other.finalization_code.clone()
            },
            materials: Self::blend_materials(&self.materials, &other.materials, blend, rng),
            iterations: if rng.random::<bool>() {
                self.iterations
            } else {
                other.iterations
            },
            angle: self.angle * blend + other.angle * inv_blend,
            step: self.step * blend + other.step * inv_blend,
            width: self.width * blend + other.width * inv_blend,
            elasticity: self.elasticity * blend + other.elasticity * inv_blend,
            tropism: match (&self.tropism, &other.tropism) {
                (Some(a), Some(b)) => Some([
                    a[0] * blend + b[0] * inv_blend,
                    a[1] * blend + b[1] * inv_blend,
                    a[2] * blend + b[2] * inv_blend,
                ]),
                (Some(t), None) | (None, Some(t)) => Some(*t),
                (None, None) => None,
            },
            seed: rng.random::<u64>(),
            prop_mappings: if rng.random::<bool>() {
                self.prop_mappings.clone()
            } else {
                other.prop_mappings.clone()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    #[test]
    fn test_parse_simple_genotype() {
        let genotype = PlantGenotype::new("omega: F\nF -> F F".to_string());
        let system = genotype.parse();
        assert!(system.is_some());
    }

    #[test]
    fn test_mutate_preserves_structure() {
        let genotype = PlantGenotype::new("omega: F\nF -> F [ + F ] F".to_string());
        let mut mutated = genotype.clone();

        let mut rng = Pcg64::seed_from_u64(42);
        mutated.mutate(&mut rng, 0.5);

        // Should still parse after mutation
        assert!(mutated.parse().is_some());
    }

    #[test]
    fn test_crossover_produces_valid_offspring() {
        let parent_a = PlantGenotype::new("omega: A\nA -> A B".to_string());
        let parent_b = PlantGenotype::new("omega: A\nA -> A A".to_string());

        let mut rng = Pcg64::seed_from_u64(42);
        let offspring = parent_a.crossover(&parent_b, &mut rng);

        // Should still parse after crossover
        assert!(offspring.parse().is_some());
    }

    #[test]
    fn test_to_source_preserves_mutated_constants() {
        // Create a genotype with a #define directive
        let source = "#define angle 25.0\nomega: F\nF -> F [ + F ] F".to_string();
        let genotype = PlantGenotype::new(source);

        // Parse and manually mutate the constant
        let mut system = genotype.parse().unwrap();
        system.constants.insert("angle".to_string(), 45.0);

        // Reconstruct source via upstream to_source()
        let reconstructed = system.to_source();

        // The reconstructed source should contain the mutated value
        assert!(
            reconstructed.contains("#define angle 45"),
            "Expected mutated angle=45, got: {}",
            reconstructed
        );
        assert!(
            !reconstructed.contains("#define angle 25"),
            "Should not contain old angle=25"
        );
    }

    #[test]
    fn test_to_source_places_define_before_omega() {
        // Create a genotype with omega using a constant
        let source = "#define len 2.0\nomega: F(len)\nF(x) -> F(x) F(x)".to_string();
        let genotype = PlantGenotype::new(source);

        let system = genotype.parse().unwrap();
        let reconstructed = system.to_source();

        // Find positions of #define and omega lines
        let define_pos = reconstructed.find("#define len");
        let omega_pos = reconstructed.find("omega:");

        assert!(
            define_pos.is_some() && omega_pos.is_some(),
            "Both #define and omega should be present in: {}",
            reconstructed
        );
        assert!(
            define_pos.unwrap() < omega_pos.unwrap(),
            "#define should appear before omega to avoid undefined constant errors.\nGot: {}",
            reconstructed
        );
    }
}
