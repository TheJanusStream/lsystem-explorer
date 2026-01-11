pub struct LSystemPreset {
    pub name: &'static str,
    pub code: &'static str,
}

pub const PRESETS: &[LSystemPreset] = &[
    LSystemPreset {
        name: "Monopodial Tree (ABOP Fig 2.6)",
        code: "config: angle 45\n#define d 137.5\n#define wr 0.707\nomega: A(100, 10)\np1: A(l, w) -> !(w) F(l) [ & B(l*0.6, w*wr) ] / (d) A(l*0.9, w*wr)\np2: B(l, w) -> !(w) F(l) [ - $ C(l*0.6, w*wr) ] C(l*0.9, w*wr)\np3: C(l, w) -> !(w) F(l) [ + $ B(l*0.6, w*wr) ] B(l*0.9, w*wr)\np4: F(l) -> F(l)\np5: !(w) -> !(w)\np6: $ -> $",
    },
    LSystemPreset {
        name: "Sympodial Tree (ABOP Fig 2.7)",
        code: "config: angle 18\n#define r1 0.9\n#define r2 0.7\n#define a1 10\n#define a2 60\n#define wr 0.707\nomega: A(100, 10)\np1: A(l,w) -> !(w)F(l)[&(a1)B(l*r1,w*wr)] /(180)[&(a2)B(l*r2,w*wr)]\np2: B(l,w) -> !(w)F(l)[+(a1)$B(l*r1,w*wr)] [-(a2)$B(l*r2,w*wr)]\np3: F(l) -> F(l)",
    },
    LSystemPreset {
        name: "Ternary Tree (ABOP Fig 2.8)",
        code: "config: angle 30\n#define d1 94.74\n#define d2 132.63\n#define a 18.95\n#define lr 1.109\n#define vr 1.732\nomega: !(1)F(200)/(45)A\np1: A : * -> !(vr)F(50)[&(a)F(50)A]/(d1)[&(a)F(50)A]/(d2)[&(a)F(50)A]\np2: F(l) : * -> F(l*lr)\np3: !(w) : * -> !(w*vr)",
    },
    LSystemPreset {
        name: "Ternary Tree (Gravity) (ABOP Fig 2.8d)",
        code: "config: angle 36\nconfig: elasticity 0.40\nconfig: tropism 0.0 -1.0 0.0\n#define d1 180\n#define d2 252\n#define a 36\n#define lr 1.07\n#define vr 1.732\nomega: !(1)F(200)/(45)A\np1: A : * -> !(vr)F(50)[&(a)F(50)A]/(d1)[&(a)F(50)A]/(d2)[&(a)F(50)A]\np2: F(l) : * -> F(l*lr)\np3: !(w) : * -> !(w*vr)",
    },
    LSystemPreset {
        name: "Stochastic Bush",
        code: "config: iterations 5\nconfig: angle 25\nomega: A(100)\np1: A(s) : 0.33 -> F(s) [ + A(s/1.5) ] [ - A(s/1.5) ]\np2: A(s) : 0.33 -> F(s) [ & A(s/1.5) ]\np3: A(s) : 0.34 -> F(s) [ / A(s/1.5) ]\np4: F(l) -> F(l)",
    },
    LSystemPreset {
        name: "Quadratic Koch Island",
        code: "config: angle 90\nconfig: step 10\nconfig: iterations 3\nomega: F(100)-F(100)-F(100)-F(100)\np1: F(s) -> F(s/3)+F(s/3)-F(s/3)-F(s/3)F(s/3)+F(s/3)+F(s/3)-F(s/3)",
    },
    LSystemPreset {
        name: "Compound Leaves (ABOP Fig 5.11a)",
        code: "config: angle 90\nconfig: iterations 10\n#define D 0\n#define R 2.0\nomega: A(0)\np1: A(d) : d>0 -> A(d-1)\np2: A(d) : d=0 -> F(1)[+A(D)][-A(D)]F(1)A(0)\np3: F(a) : * -> F(a*R)",
    },
    LSystemPreset {
        name: "Compound Leaves (Alternating) (ABOP Fig 5.12a)",
        code: "config: angle 90\nconfig: iterations 10\n#define D 1\n#define R 1.36\nomega: A(0)\np1: A(d) : d>0 -> A(d-1)\np2: A(d) : d=0 -> F(1)[+A(D)]F(1)B(0)\np3: B(d) : d>0 -> B(d-1)\np4: B(d) : d=0 -> F(1)[-B(D)]F(1)A(0)\np5: F(a) : * -> F(a*R)",
    },
];
