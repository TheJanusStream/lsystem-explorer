use bevy::math::Vec3;

use crate::core::config::TextureType;

/// Preset material configuration for material slot 0.
#[derive(Clone, Copy)]
pub struct PresetMaterial {
    pub base_color: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub emission_color: [f32; 3],
    pub emission_strength: f32,
    pub uv_scale: f32,
    pub texture_type: TextureType,
}

impl Default for PresetMaterial {
    fn default() -> Self {
        Self {
            base_color: [0.55, 0.27, 0.07], // Brown wood color
            roughness: 0.8,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        }
    }
}

/// Preset camera configuration.
#[derive(Clone, Copy)]
pub struct PresetCamera {
    pub focus: Vec3,
    pub distance: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for PresetCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::new(0.0, 400.0, 0.0),
            distance: 1200.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }
    }
}

pub struct LSystemPreset {
    pub name: &'static str,
    pub code: &'static str,
    pub iterations: usize,
    pub angle: f32,
    pub step: f32,
    pub width: f32,
    pub elasticity: f32,
    pub tropism: Option<Vec3>,
    /// Initial turtle color (RGB, 0-1 range).
    pub initial_color: Option<[f32; 3]>,
    /// Material settings for each material slot (slot_id, material).
    pub materials: &'static [(u8, PresetMaterial)],
    /// Camera settings override.
    pub camera: Option<PresetCamera>,
}

pub const PRESETS: &[LSystemPreset] = &[
    LSystemPreset {
        name: "Monopodial Tree (ABOP Fig 2.6)",
        code: "#define d 137.5\n#define wr 0.707\nomega: A(100, 10)\np1: A(l, w) -> !(w) F(l) [ & B(l*0.6, w*wr) ] / (d) A(l*0.9, w*wr)\np2: B(l, w) -> !(w) F(l) [ - $ C(l*0.6, w*wr) ] C(l*0.9, w*wr)\np3: C(l, w) -> !(w) F(l) [ + $ B(l*0.6, w*wr) ] B(l*0.9, w*wr)\np4: F(l) -> F(l)\np5: !(w) -> !(w)\np6: $ -> $",
        iterations: 8,
        angle: 45.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.0,
        tropism: None,
        initial_color: None,
        materials: &[(0, PresetMaterial {
            base_color: [0.55, 0.27, 0.07],
            roughness: 0.8,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: None,
    },
    LSystemPreset {
        name: "Sympodial Tree (ABOP Fig 2.7)",
        code: "#define r1 0.9\n#define r2 0.7\n#define a1 10\n#define a2 60\n#define wr 0.707\nomega: A(100, 10)\np1: A(l,w) -> !(w)F(l)[&(a1)B(l*r1,w*wr)] /(180)[&(a2)B(l*r2,w*wr)]\np2: B(l,w) -> !(w)F(l)[+(a1)$B(l*r1,w*wr)] [-(a2)$B(l*r2,w*wr)]\np3: F(l) -> F(l)",
        iterations: 8,
        angle: 18.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.0,
        tropism: None,
        initial_color: None,
        materials: &[(0, PresetMaterial {
            base_color: [0.4, 0.25, 0.1],
            roughness: 0.75,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: None,
    },
    LSystemPreset {
        name: "Ternary Tree (ABOP Fig 2.8)",
        code: "#define d1 94.74\n#define d2 132.63\n#define a 18.95\n#define lr 1.109\n#define vr 1.732\nomega: !(1)F(200)/(45)A\np1: A : * -> !(vr)F(50)[&(a)F(50)A]/(d1)[&(a)F(50)A]/(d2)[&(a)F(50)A]\np2: F(l) : * -> F(l*lr)\np3: !(w) : * -> !(w*vr)",
        iterations: 6,
        angle: 30.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.0,
        tropism: None,
        initial_color: None,
        materials: &[(0, PresetMaterial {
            base_color: [0.35, 0.2, 0.08],
            roughness: 0.85,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 600.0, 0.0),
            distance: 1800.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
    },
    LSystemPreset {
        name: "Ternary Tree (Gravity) (ABOP Fig 2.8d)",
        code: "#define d1 180\n#define d2 252\n#define a 36\n#define lr 1.07\n#define vr 1.732\nomega: !(1)F(200)/(45)A\np1: A : * -> !(vr)F(50)[&(a)F(50)A]/(d1)[&(a)F(50)A]/(d2)[&(a)F(50)A]\np2: F(l) : * -> F(l*lr)\np3: !(w) : * -> !(w*vr)",
        iterations: 6,
        angle: 36.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.40,
        tropism: Some(Vec3::new(0.0, -1.0, 0.0)),
        initial_color: None,
        materials: &[(0, PresetMaterial {
            base_color: [0.35, 0.2, 0.08],
            roughness: 0.85,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 500.0, 0.0),
            distance: 1500.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
    },
    LSystemPreset {
        name: "Stochastic Bush",
        code: "omega: A(100)\np1: A(s) : 0.33 -> F(s) [ + A(s/1.5) ] [ - A(s/1.5) ]\np2: A(s) : 0.33 -> F(s) [ & A(s/1.5) ]\np3: A(s) : 0.34 -> F(s) [ / A(s/1.5) ]\np4: F(l) -> F(l)",
        iterations: 5,
        angle: 25.0,
        step: 1.0,
        width: 2.0,
        elasticity: 0.0,
        tropism: None,
        initial_color: Some([0.2, 0.5, 0.15]),
        materials: &[(0, PresetMaterial {
            base_color: [0.2, 0.5, 0.15],
            roughness: 0.7,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 50.0, 0.0),
            distance: 300.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
    },
    LSystemPreset {
        name: "Quadratic Koch Island",
        code: "omega: F(100)-F(100)-F(100)-F(100)\np1: F(s) -> F(s/3)+F(s/3)-F(s/3)-F(s/3)F(s/3)+F(s/3)+F(s/3)-F(s/3)",
        iterations: 3,
        angle: 90.0,
        step: 10.0,
        width: 1.0,
        elasticity: 0.0,
        tropism: None,
        initial_color: Some([0.3, 0.6, 0.9]),
        materials: &[(0, PresetMaterial {
            base_color: [0.3, 0.6, 0.9],
            roughness: 0.3,
            metallic: 0.5,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 0.0, 0.0),
            distance: 500.0,
            pitch: -std::f32::consts::TAU / 4.0,
            yaw: 0.0,
        }),
    },
    LSystemPreset {
        name: "Compound Leaves (ABOP Fig 5.11a)",
        code: "#define D 0\n#define R 2.0\nomega: A(0)\np1: A(d) : d>0 -> A(d-1)\np2: A(d) : d=0 -> F(1)[+ ~ (0, 0.5) A(D)][- ~ (0, 0.5) A(D)]F(1)A(0)\np3: F(a) : * -> F(a*R)",
        iterations: 8,
        angle: 90.0,
        step: 1.0,
        width: 0.05,
        elasticity: 0.0,
        tropism: None,
        initial_color: Some([0.15, 0.4, 0.1]),
        materials: &[(0, PresetMaterial {
            base_color: [0.15, 0.4, 0.1],
            roughness: 0.6,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 100.0, 0.0),
            distance: 400.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
    },
    LSystemPreset {
        name: "Compound Leaves (Alternating) (ABOP Fig 5.12a)",
        code: "#define D 1\n#define R 1.36\nomega: A(0)\np1: A(d) : d>0 -> A(d-1)\np2: A(d) : d=0 -> F(1)[+ ~ (0, 0.5) A(D)]F(1)B(0)\np3: B(d) : d>0 -> B(d-1)\np4: B(d) : d=0 -> F(1)[- ~ (0, 0.5) B(D)]F(1)A(0)\np5: F(a) : * -> F(a*R)",
        iterations: 8,
        angle: 90.0,
        step: 1.0,
        width: 0.05,
        elasticity: 0.0,
        tropism: None,
        initial_color: Some([0.15, 0.45, 0.1]),
        materials: &[(0, PresetMaterial {
            base_color: [0.15, 0.45, 0.1],
            roughness: 0.6,
            metallic: 0.0,
            emission_color: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            uv_scale: 1.0,
            texture_type: TextureType::None,
        })],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 80.0, 0.0),
            distance: 350.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
    },
    LSystemPreset {
        name: "PBR Tech-Tree",
        code: "#define MAX 7\n// Metal Trunk (Silver)\nomega: @(1.0) #(0.2) '(0.9, 0.9, 0.9) ,(0) A(MAX)\n\np1: A(t) : t > 0 ->!(t * 0.05)F[ &(35) B(t-1) ][ /(120) &(35) B(t-1) ]/(120) A(t-1)\n\n// Branches switch to Material 1 (Energy/Glass)\n// Color Gradient: Yellow -> Red\np2: B(t) : t > 0 ->,(1)'(1.0, t/MAX, 0.0)@(0.0) #(0.1)!(t * 0.04)F(8)[ +(30) B(t-1) ][ -(30) B(t-1) ]",
        iterations: 7,
        angle: 18.0,
        step: 10.0,
        width: 0.1,
        elasticity: 0.1,
        tropism: Some(Vec3::new(0.0, -0.5, 0.0)),
        initial_color: Some([0.9, 0.9, 0.9]),
        materials: &[
            // Material 0: Metal Trunk (Silver)
            (0, PresetMaterial {
                base_color: [0.8, 0.8, 0.85],
                roughness: 0.2,
                metallic: 1.0,
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            }),
            // Material 1: Energy/Glass (glowing branches)
            (1, PresetMaterial {
                base_color: [1.0, 0.6, 0.0],
                roughness: 0.1,
                metallic: 0.0,
                emission_color: [1.0, 0.5, 0.0],
                emission_strength: 2.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            }),
        ],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 150.0, 0.0),
            distance: 600.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
    },
];
