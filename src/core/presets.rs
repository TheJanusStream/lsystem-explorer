pub struct LSystemPreset {
    pub name: &'static str,
    pub code: &'static str,
}

pub const PRESETS: &[LSystemPreset] = &[
    LSystemPreset {
        name: "Monopodial Tree (ABOP Fig 2.6)",
        // Refactored to use implicit &, +, - symbols so the 'config: angle'
        // controls the geometry dynamically.
        code: "config: angle 45\n#define d 137.5\n#define wr 0.707\nomega: A(100, 10)\np1: A(l, w) -> !(w) F(l) [ & B(l*0.6, w*wr) ] / (d) A(l*0.9, w*wr)\np2: B(l, w) -> !(w) F(l) [ - $ C(l*0.6, w*wr) ] C(l*0.9, w*wr)\np3: C(l, w) -> !(w) F(l) [ + $ B(l*0.6, w*wr) ] B(l*0.9, w*wr)\np4: F(l) -> F(l)\np5: !(w) -> !(w)\np6: $ -> $",
    },
    LSystemPreset {
        name: "Sympodial Tree (ABOP Fig 2.7)",
        // Complex specific angles, kept parametric.
        code: "config: angle 18\n#define r1 0.9\n#define r2 0.7\n#define a1 10\n#define a2 60\n#define wr 0.707\nomega: A(100, 10)\np1: A(l,w) -> !(w)F(l)[&(a1)B(l*r1,w*wr)] /(180)[&(a2)B(l*r2,w*wr)]\np2: B(l,w) -> !(w)F(l)[+(a1)$B(l*r1,w*wr)] [-(a2)$B(l*r2,w*wr)]\np3: F(l) -> F(l)",
    },
    LSystemPreset {
        name: "Ternary Tree (ABOP Fig 2.8)",
        // Complex specific angles, kept parametric.
        code: "config: angle 30\n#define d1 94.74\n#define d2 132.63\n#define a 18.95\n#define lr 1.109\n#define vr 1.732\nomega: !(1)F(200)/(45)A\np1: A -> !(vr)F(50)[&(a)F(50)A]/(d1)[&(a)F(50)A]/(d2)[&(a)F(50)A]\np2: F(l) -> F(l*lr)\np3: !(w) -> !(w*vr)",
    },
    LSystemPreset {
        name: "Stochastic Bush",
        // Relies on implicit + - & / for random variation
        code: "config: iterations 5\nconfig: angle 25\nomega: A(100)\np1: A(s) : 0.33 -> F(s) [ + A(s/1.5) ] [ - A(s/1.5) ]\np2: A(s) : 0.33 -> F(s) [ & A(s/1.5) ]\np3: A(s) : 0.34 -> F(s) [ / A(s/1.5) ]\np4: F(l) -> F(l)",
    },
    LSystemPreset {
        name: "Quadratic Koch Island",
        // Non-parametric turtle, relies on config angle 90
        code: "config: angle 90\nconfig: step 10\nconfig: iterations 3\nomega: F(100)-F(100)-F(100)-F(100)\np1: F(s) -> F(s/3)+F(s/3)-F(s/3)-F(s/3)F(s/3)+F(s/3)+F(s/3)-F(s/3)",
    },
];
