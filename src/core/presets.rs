use bevy::math::Vec3;

use crate::core::config::{PropMeshType, TextureType};

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
    /// Prop ID to mesh type mapping.
    pub prop_meshes: &'static [(u16, PropMeshType)],
}

pub const PRESETS: &[LSystemPreset] = &[
    LSystemPreset {
        name: "Quadratic Koch Island (ABOP Fig 1.6)",
        code: "omega: F(100)-F(100)-F(100)-F(100)\n\
               F(s) -> F(s/3)+F(s/3)-F(s/3)-F(s/3)F(s/3)+F(s/3)+F(s/3)-F(s/3)",
        iterations: 3,
        angle: 90.0,
        step: 10.0,
        width: 1.0,
        elasticity: 0.0,
        tropism: None,
        initial_color: Some([0.3, 0.6, 0.9]),
        materials: &[(
            0,
            PresetMaterial {
                base_color: [0.3, 0.6, 0.9],
                roughness: 1.0,
                metallic: 0.0,
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            },
        )],
        camera: Some(PresetCamera {
            focus: Vec3::new(70.0, 120.0, 0.0),
            distance: 500.0,
            pitch: 0.0,
            yaw: 0.0,
        }),
        prop_meshes: &[],
    },
    LSystemPreset {
        name: "Sierpinski gasket (ABOP Fig 1.10 (b))",
        code: "omega: Fr\n\
               Fl -> Fr+Fl+Fr\n\
               Fr -> Fl-Fr-Fl\n\
               /// DECOMPOSITION ///\n\
               Fr -> F\n\
               /// DECOMPOSITION ///\n\
               Fl -> F",
        iterations: 5,
        angle: 60.0,
        step: 10.0,
        width: 1.0,
        elasticity: 0.0,
        tropism: None,
        initial_color: Some([1.0, 1.0, 1.0]),
        materials: &[(
            0,
            PresetMaterial {
                base_color: [0.9, 0.3, 0.6],
                roughness: 1.0,
                metallic: 0.0,
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            },
        )],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 140.0, 0.0),
            distance: 500.0,
            pitch: 0.0,
            yaw: 0.0,
        }),
        prop_meshes: &[],
    },
    LSystemPreset {
        name: "Branching pattern (ABOP Fig 1.39)",
        code: "#define R 1.456\n\
               omega: A(150)\n\
               A(s) -> F(s)[+A(s/R)][-A(s/R)]",
        iterations: 12,
        angle: 85.0,
        step: 10.0,
        width: 1.0,
        elasticity: 0.0,
        tropism: None,
        initial_color: Some([1.0, 1.0, 1.0]),
        materials: &[(
            0,
            PresetMaterial {
                base_color: [0.9, 0.6, 0.3],
                roughness: 1.0,
                metallic: 0.0,
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            },
        )],
        camera: Some(PresetCamera {
            focus: Vec3::new(-50.0, 140.0, 0.0),
            distance: 500.0,
            pitch: 0.0,
            yaw: 0.0,
        }),
        prop_meshes: &[],
    },
    LSystemPreset {
        name: "Monopodial Tree (ABOP Fig 2.6)",
        code: "#define r1 0.9\n\
               #define r2 0.6\n\
               #define a0 45\n\
               #define a2 45\n\
               #define d 137.5\n\
               #define wr 0.707\n\
               omega: A(100, 10)\n\
               p1: A(l, w) -> !(w) F(l) [ &(a0) B(l*r2, w*wr) ] / (d) A(l*r1, w*wr)\n\
               p2: B(l, w) -> !(w) F(l) [ -(a2) $ C(l*r2, w*wr) ] C(l*r1, w*wr)\n\
               p3: C(l, w) -> !(w) F(l) [ +(a2) $ B(l*r2, w*wr) ] B(l*r1, w*wr)",
        iterations: 8,
        angle: 45.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.0,
        tropism: None,
        initial_color: None,
        materials: &[(
            0,
            PresetMaterial {
                base_color: [0.55, 0.27, 0.07],
                roughness: 0.8,
                metallic: 0.0,
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            },
        )],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 300.0, 0.0),
            distance: 900.0,
            pitch: 0.0,
            yaw: 0.0,
        }),
        prop_meshes: &[],
    },
    LSystemPreset {
        name: "Sympodial Tree (ABOP Fig 2.7)",
        code: "#define r1 0.9\n\
               #define r2 0.7\n\
               #define a1 10\n\
               #define a2 60\n\
               #define wr 0.707\n\
               omega: A(100, 10)\n\
               p1: A(l,w) -> !(w)F(l)[&(a1)B(l*r1,w*wr)] /(180)[&(a2)B(l*r2,w*wr)]\n\
               p2: B(l,w) -> !(w)F(l)[+(a1)$B(l*r1,w*wr)] [-(a2)$B(l*r2,w*wr)]",
        iterations: 10,
        angle: 18.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.0,
        tropism: None,
        initial_color: None,
        materials: &[(
            0,
            PresetMaterial {
                base_color: [0.4, 0.25, 0.1],
                roughness: 0.75,
                metallic: 0.0,
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            },
        )],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 300.0, 0.0),
            distance: 900.0,
            pitch: 0.0,
            yaw: 0.0,
        }),
        prop_meshes: &[],
    },
    LSystemPreset {
        name: "Ternary Tree (Gravity) (ABOP Fig 2.8)",
        code: "#define d1 180\n\
               #define d2 252\n\
               #define a 36\n\
               #define lr 1.07\n\
               #define vr 1.732\n\
               #define s 50.0\n\
               omega: !(1)F(4*s)/(45)A\n\
               p1: A : * -> !(vr)F(s)[&(a)F(s)A]/(d1)[&(a)F(s)A]/(d2)[&(a)F(s)A]\n\
               p2: F(l) : * -> F(l*lr)\n\
               p3: !(w) : * -> !(w*vr)",
        iterations: 6,
        angle: 36.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.40,
        tropism: Some(Vec3::new(0.0, -1.0, 0.0)),
        initial_color: None,
        materials: &[(
            0,
            PresetMaterial {
                base_color: [0.35, 0.2, 0.08],
                roughness: 0.85,
                metallic: 0.0,
                emission_color: [0.0, 0.0, 0.0],
                emission_strength: 0.0,
                uv_scale: 1.0,
                texture_type: TextureType::None,
            },
        )],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 500.0, 0.0),
            distance: 1500.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
        prop_meshes: &[],
    },
    LSystemPreset {
        name: "Ternary Tree (+Props +Materials +Variations)",
        code: "#define d1 180\n\
               #define th 2.5\n\
               #define d2 252\n\
               #define a 36\n\
               #define lr 1.07\n\
               #define vr 1.732\n\
               #define ps 60.0\n\
               #define s 50.0\n\
               omega: !(th)F(4*s)/(45)A,(1)~(0,ps)\n\
               p0: A : 0.7 -> !(th*vr)F(s)[&(a)F(s)A,(1)~(0,ps)]/(d1)[&(a)F(s)A,(1)~(0,ps)]/(d2)[&(a)F(s)A,(1)~(0,ps)]\n\
               p1: A : 0.3 -> !(th*vr)F(s)A\n\
               p2: F(l) : * -> F(l*lr)\n\
               p3: !(w) : * -> !(w*vr)\n\
               p4: ,(id) : id = 1 -> ,(2)\n\
               p5: ,(id) : id = 2 -> \n\
               p6: ~(id,sc) : id = 0 -> ~(1,sc)\n\
               p7: ~(id,sc) : id = 1 ->",
        iterations: 6,
        angle: 36.0,
        step: 1.0,
        width: 0.1,
        elasticity: 0.25,
        tropism: Some(Vec3::new(0.0, -1.0, 0.0)),
        initial_color: None,
        materials: &[
            (
                0,
                PresetMaterial {
                    base_color: [0.35, 0.2, 0.08],
                    roughness: 0.85,
                    metallic: 0.0,
                    emission_color: [0.0, 0.0, 0.0],
                    emission_strength: 0.0,
                    uv_scale: 1.0,
                    texture_type: TextureType::None,
                },
            ),
            (
                1,
                PresetMaterial {
                    base_color: [0.2, 1.0, 0.2],
                    roughness: 0.5,
                    metallic: 0.0,
                    emission_color: [0.0, 1.0, 0.0],
                    emission_strength: 0.0,
                    uv_scale: 1.0,
                    texture_type: TextureType::None,
                },
            ),
            (
                2,
                PresetMaterial {
                    base_color: [1.0, 0.2, 0.2],
                    roughness: 0.3,
                    metallic: 0.3,
                    emission_color: [1.0, 0.2, 0.2],
                    emission_strength: 0.0,
                    uv_scale: 1.0,
                    texture_type: TextureType::None,
                },
            ),
        ],
        camera: Some(PresetCamera {
            focus: Vec3::new(0.0, 500.0, 0.0),
            distance: 1500.0,
            pitch: std::f32::consts::TAU / 64.0,
            yaw: std::f32::consts::TAU / 5.0,
        }),
        prop_meshes: &[(0, PropMeshType::Leaf), (1, PropMeshType::Sphere)],
    },
];
