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
use rand::Rng;
use serde::{Deserialize, Serialize};
use symbios::System;
use symbios::system::{CrossoverConfig, MutationConfig, StructuralMutationConfig};
use symbios_genetics::Genotype;

use crate::core::config::split_source_code;
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
        }
    }
}

impl SerializableMaterial {
    /// Converts back to MaterialSettings (texture defaults to None).
    pub fn to_material_settings(&self) -> MaterialSettings {
        MaterialSettings {
            base_color: self.base_color,
            emission_color: self.emission_color,
            emission_strength: self.emission_strength,
            roughness: self.roughness,
            metallic: self.metallic,
            texture: TextureType::None,
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
    pub materials: HashMap<u8, SerializableMaterial>,
    /// Number of derivation iterations.
    pub iterations: usize,
    /// Default turn angle in degrees.
    pub angle: f32,
    /// Step size for forward movement.
    pub step: f32,
    /// Default branch width.
    pub width: f32,
    /// Random seed for stochastic rules.
    pub seed: u64,
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
            seed: 42,
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
            seed: 42,
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
        let mut system = System::new();

        // Parse line by line to handle axiom and rules
        for line in self.source_code.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            if trimmed.starts_with("omega:") {
                // Extract axiom
                let axiom = trimmed.strip_prefix("omega:")?.trim();
                system.set_axiom(axiom).ok()?;
            } else if trimmed.starts_with('#') {
                // Handle #define directives
                if let Some(rest) = trimmed.strip_prefix("#define") {
                    let parts: Vec<&str> = rest.trim().splitn(2, char::is_whitespace).collect();
                    if parts.len() == 2
                        && let Ok(val) = parts[1].trim().parse::<f64>()
                    {
                        system.constants.insert(parts[0].to_string(), val);
                    }
                }
            } else if trimmed.contains("->") {
                // This is a rule
                system.add_rule(trimmed).ok()?;
            }
        }

        Some(system)
    }

    /// Reconstructs source code from a mutated System.
    ///
    /// This is the key round-trip operation: after mutating a System,
    /// we export its rules back to source code to maintain the source
    /// as the single source of truth.
    ///
    /// Output order: comments/directives → #define lines → omega line → rules
    fn reconstruct_source(system: &System, original_source: &str) -> String {
        // Export all rules from the system
        let exported_rules = system.export_rules();

        // Extract non-rule lines from original source, EXCLUDING #define directives
        // (we'll regenerate those from system.constants to preserve mutations)
        // Separate the omega line from other preamble lines
        let mut preamble_lines = Vec::new();
        let mut omega_line: Option<String> = None;
        let mut seen_rules = false;

        for line in original_source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                // Keep comments and blank lines until we hit rules
                if !seen_rules {
                    preamble_lines.push(line.to_string());
                }
            } else if trimmed.starts_with("omega:") {
                // Store omega line separately to place after #define directives
                omega_line = Some(line.to_string());
            } else if trimmed.starts_with("#define") {
                // Skip old #define directives - we'll regenerate from system.constants
            } else if trimmed.starts_with('#') {
                // Keep other directives (#ignore, etc.)
                preamble_lines.push(line.to_string());
            } else if trimmed.contains("->") {
                // This is a rule line
                seen_rules = true;
            } else if !seen_rules {
                // Keep other preamble lines
                preamble_lines.push(line.to_string());
            }
        }

        // Build new source: preamble first (comments, directives like #ignore)
        let mut result = preamble_lines.join("\n");
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }

        // Append #define directives from system.constants (mutated values)
        let mut constants: Vec<_> = system.constants.iter().collect();
        constants.sort_by_key(|(k, _)| *k);
        for (name, value) in constants {
            result.push_str(&format!("#define {} {}\n", name, value));
        }

        // Add omega line after #define directives so constants are defined
        if let Some(omega) = omega_line {
            result.push_str(&omega);
            result.push('\n');
        }

        // Add exported rules
        for (_, rule_source) in exported_rules {
            result.push_str(&rule_source);
            result.push('\n');
        }

        result.trim_end().to_string()
    }

    /// Mutates the material colors slightly.
    fn mutate_materials<R: Rng>(&mut self, rng: &mut R, rate: f32) {
        for settings in self.materials.values_mut() {
            if rng.random::<f32>() < rate {
                // Mutate base color slightly
                for channel in &mut settings.base_color {
                    *channel = (*channel + (rng.random::<f32>() - 0.5) * 0.1).clamp(0.0, 1.0);
                }
            }
            if rng.random::<f32>() < rate * 0.5 {
                // Occasionally mutate roughness/metallic
                settings.roughness =
                    (settings.roughness + (rng.random::<f32>() - 0.5) * 0.1).clamp(0.0, 1.0);
            }
        }
    }

    /// Blends materials from two parents.
    fn blend_materials(
        a: &HashMap<u8, SerializableMaterial>,
        b: &HashMap<u8, SerializableMaterial>,
        blend: f32,
    ) -> HashMap<u8, SerializableMaterial> {
        let mut result = HashMap::new();

        // Collect all slot IDs from both parents
        let all_slots: std::collections::HashSet<_> = a.keys().chain(b.keys()).copied().collect();

        for slot in all_slots {
            let settings = match (a.get(&slot), b.get(&slot)) {
                (Some(ma), Some(mb)) => {
                    // Blend the two materials
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
        self.source_code = Self::reconstruct_source(&system, &self.source_code);

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
        let source_code = Self::reconstruct_source(&offspring_system, &self.source_code);

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
            materials: Self::blend_materials(&self.materials, &other.materials, blend),
            iterations: if rng.random::<bool>() {
                self.iterations
            } else {
                other.iterations
            },
            angle: self.angle * blend + other.angle * inv_blend,
            step: self.step * blend + other.step * inv_blend,
            width: self.width * blend + other.width * inv_blend,
            seed: rng.random::<u64>(),
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
    fn test_reconstruct_source_preserves_mutated_constants() {
        // Create a genotype with a #define directive
        let source = "#define angle 25.0\nomega: F\nF -> F [ + F ] F".to_string();
        let genotype = PlantGenotype::new(source);

        // Parse and manually mutate the constant
        let mut system = genotype.parse().unwrap();
        system.constants.insert("angle".to_string(), 45.0);

        // Reconstruct source from mutated system
        let reconstructed = PlantGenotype::reconstruct_source(&system, &genotype.source_code);

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
    fn test_reconstruct_source_places_define_before_omega() {
        // Create a genotype with omega using a constant
        let source = "#define len 2.0\nomega: F(len)\nF(x) -> F(x) F(x)".to_string();
        let genotype = PlantGenotype::new(source);

        let system = genotype.parse().unwrap();
        let reconstructed = PlantGenotype::reconstruct_source(&system, &genotype.source_code);

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
