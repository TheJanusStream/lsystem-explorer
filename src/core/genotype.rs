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
use bevy_symbios_texture::bark::BarkConfig;
use bevy_symbios_texture::leaf::LeafConfig;
use bevy_symbios_texture::twig::TwigConfig;
use rand::Rng;
use serde::{Deserialize, Serialize};
use symbios::System;
use symbios::system::crossover::CrossoverConfig;
use symbios::system::mutate::{MutationConfig, StructuralMutationConfig};
use symbios_genetics::Genotype;

use crate::core::config::{PropMeshType, scan_max_material_id, split_source_code};
use crate::core::presets::LSystemPreset;

/// Serializable version of material settings for genetic storage.
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
    #[serde(default)]
    pub leaf_config: LeafConfig,
    #[serde(default)]
    pub twig_config: TwigConfig,
    #[serde(default)]
    pub bark_config: BarkConfig,
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
            leaf_config: LeafConfig::default(),
            twig_config: TwigConfig::default(),
            bark_config: BarkConfig::default(),
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
            texture_type: m.texture,
            leaf_config: m.leaf_config.clone(),
            twig_config: m.twig_config.clone(),
            bark_config: m.bark_config.clone(),
        }
    }
}

impl SerializableMaterial {
    /// Converts back to full [`MaterialSettings`], preserving texture type and foliage configs.
    pub fn to_material_settings(&self) -> MaterialSettings {
        MaterialSettings {
            base_color: self.base_color,
            emission_color: self.emission_color,
            emission_strength: self.emission_strength,
            roughness: self.roughness,
            metallic: self.metallic,
            texture: self.texture_type,
            uv_scale: self.uv_scale,
            leaf_config: self.leaf_config.clone(),
            twig_config: self.twig_config.clone(),
            bark_config: self.bark_config.clone(),
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
    pub materials: HashMap<u8, SerializableMaterial>,
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
    pub fn with_materials(mut self, materials: &HashMap<u8, MaterialSettings>) -> Self {
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
        let materials: HashMap<u8, SerializableMaterial> = preset
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
                        texture_type: mat.texture_type,
                        ..Default::default()
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
    pub fn get_material_settings(&self) -> HashMap<u8, MaterialSettings> {
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

    /// Mutates material colors and foliage texture configs.
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
            // Mutate active foliage texture config via its Genotype implementation
            match settings.texture_type {
                TextureType::Leaf => settings.leaf_config.mutate(rng, rate),
                TextureType::Twig => settings.twig_config.mutate(rng, rate),
                TextureType::Bark => settings.bark_config.mutate(rng, rate),
                _ => {}
            }
        }
    }

    /// Blends materials from two parents using foliage config crossover.
    fn blend_materials<R: Rng>(
        a: &HashMap<u8, SerializableMaterial>,
        b: &HashMap<u8, SerializableMaterial>,
        blend: f32,
        rng: &mut R,
    ) -> HashMap<u8, SerializableMaterial> {
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
                        texture_type: if blend >= 0.5 { ma.texture_type } else { mb.texture_type },
                        // Use Genotype crossover for foliage configs
                        leaf_config: ma.leaf_config.crossover(&mb.leaf_config, rng),
                        twig_config: ma.twig_config.crossover(&mb.twig_config, rng),
                        bark_config: ma.bark_config.crossover(&mb.bark_config, rng),
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
