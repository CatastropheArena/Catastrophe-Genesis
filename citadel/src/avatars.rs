use std::fmt::Write;

// MakeAvatar create svg from seed
pub fn make_avatar(seed_string: &str) -> String {
    let seed = generate_seed(seed_string);
    if seed & 1 == 0 {
        female_avatar(seed, "")
    } else {
        male_avatar(seed, "")
    }
}

// MakeFemaleAvatar create female svg from seed
pub fn make_female_avatar(seed_string: &str) -> String {
    female_avatar(generate_seed(seed_string), "")
}

// MakeMaleAvatar create male svg from seed
pub fn make_male_avatar(seed_string: &str) -> String {
    male_avatar(generate_seed(seed_string), "")
}

/**
 *  Based on TypeScript DiceBear Avatars, which in turn were inspired by 8biticon avatars:
 *  (MIT License, Copyright (c) 2012 Plastic Jam, Copyright (c) 2019 DiceBear)
 *    cf. https://github.com/DiceBear/avatars/blob/master/packages/avatars-male-sprites/src/index.ts
 */

/// 线性同余生成器
struct Lcg {
    seed: u64,
}

/// 从字符串生成种子值
fn generate_seed(seed_string: &str) -> u64 {
    let mut seed: u64 = 0;
    for c in seed_string.bytes() {
        seed = seed.rotate_left(8);
        seed ^= u64::from(c);  // 使用 u64::from 替代 as u64
    }
    seed
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg { seed }
    }

    fn random(&mut self) -> u32 {
        // Linear Congruent Generator, POSIX/glibc [de]rand48 setting
        self.seed = (25214903917_u64.wrapping_mul(self.seed).wrapping_add(11)) % 281474976710656;
        self.seed as u32
    }

    fn binomial(&mut self, p: f64) -> bool {
        let sample = f64::from(self.random()) * (1.0 / 4294967295.0);
        sample > p
    }

    fn pick_one<T>(&mut self, s: &[T]) -> T 
    where 
        T: Clone
    {
        let n = s.len() as u32;
        s[(self.random() % n) as usize].clone()
    }

    fn pick_one_float(&mut self, s: &[f64]) -> f64 {
        let n = s.len() as u32;
        s[(self.random() % n) as usize]
    }

    fn pick_a_or_b<T>(&mut self, p: f64, a: T, b: T) -> T 
    where
        T: Clone
    {
        if self.binomial(p) {
            a
        } else {
            b
        }
    }
}

fn linear_congruential_generator(seed: u64) -> Lcg {
    Lcg::new(seed)
}

#[derive(Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Clone, Copy)]
struct Hsv {
    h: f64,
    s: f64,
    v: f64,
}

/// 从浮点数值创建 RGB 颜色
fn f2rgb(r: f64, g: f64, b: f64) -> Rgb {
    Rgb {
        r: (r * (1.0 / 255.0)) as u8,  // 优化除法运算
        g: (g * (1.0 / 255.0)) as u8,
        b: (b * (1.0 / 255.0)) as u8,
        a: 255,
    }
}

/// 从浮点数值创建 HSV 颜色
fn f2hsv(h: f64, s: f64, v: f64) -> Hsv {
    Hsv { h, s, v }
}

fn to_rgb(s: &str) -> Rgb {
    let mut c = Rgb {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    
    if s.len() >= 7 && s.starts_with('#') {
        if let Ok(r) = u8::from_str_radix(&s[1..3], 16) {
            c.r = r;
        }
        if let Ok(g) = u8::from_str_radix(&s[3..5], 16) {
            c.g = g;
        }
        if let Ok(b) = u8::from_str_radix(&s[5..7], 16) {
            c.b = b;
        }
    }
    c
}

impl Rgb {
    /// 转换为 HSV 颜色空间
    fn to_hsv(&self) -> Hsv {
        let r = f64::from(self.r);
        let g = f64::from(self.g);
        let b = f64::from(self.b);
        let min = r.min(g.min(b));
        let v = r.max(g.max(b));
        let c = v - min;
        
        let s = if v != 0.0 { c / v } else { 0.0 };
        
        let h = if min != v {
            let h = if v == r {
                ((g - b) / c) % 6.0
            } else if v == g {
                (b - r) / c + 2.0
            } else {
                (r - g) / c + 4.0
            };
            h * 60.0
        } else {
            0.0
        };
        
        let h = if h < 0.0 { h + 360.0 } else { h };
        
        f2hsv(h, s, v)
    }

    /// 获取比参考颜色更亮的颜色
    fn brighter_than(&self, ref_color: &Rgb, delta: f64) -> Rgb {
        let mut primary = self.to_hsv();
        let secondary = ref_color.to_hsv();
        if primary.v >= secondary.v + delta {
            return *self;
        }
        primary.v = (secondary.v + delta).min(360.0);
        primary.to_rgb()
    }

    /// 获取比参考颜色更暗的颜色
    fn darker_than(&self, ref_color: &Rgb, delta: f64) -> Rgb {
        let mut primary = self.to_hsv();
        let secondary = ref_color.to_hsv();
        if primary.v <= secondary.v - delta {
            return *self;
        }
        primary.v = (secondary.v - delta).max(0.0);
        primary.to_rgb()
    }

    fn brighter_or_darker_than(&self, ref_color: &Rgb, delta: f64) -> Rgb {
        let primary = self.to_hsv();
        let secondary = ref_color.to_hsv();
        if primary.v <= secondary.v {
            return self.darker_than(ref_color, delta);
        }
        self.brighter_than(ref_color, delta)
    }

    fn with_alpha(&self, alpha: f64) -> Rgb {
        let mut result = *self;
        result.a = (alpha * 255.0) as u8;
        result
    }

    fn html(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Hsv {
    fn to_rgb(&self) -> Rgb {
        let h = (self.h / 60.0) as i32;
        let f = self.h / 60.0 - h as f64;
        let p = self.v * (1.0 - self.s);
        let q = self.v * (1.0 - self.s * f);
        let t = self.v * (1.0 - self.s * (1.0 - f));
        match h {
            0 | 6 => f2rgb(self.v, t, p),
            1 => f2rgb(q, self.v, p),
            2 => f2rgb(p, self.v, t),
            3 => f2rgb(p, q, self.v),
            4 => f2rgb(t, p, self.v),
            5 => f2rgb(self.v, p, q),
            _ => f2rgb(0.0, 0.0, 0.0),
        }
    }
}
struct AvatarColors {
    skin: Rgb,
    hair: Rgb,
    eyes: Rgb,
    clothes: Rgb,
    hat: Rgb,
    glasses: Rgb,
}


struct DerivedColors {
    hair: Rgb,
    eyebrows: Rgb,
    mustache: Rgb,
    mouth: Rgb,
}

/// SVG 颜色替换器
struct SvgColorReplacer<'a> {
    colors: &'a AvatarColors,
    derived_colors: &'a DerivedColors,
}

impl<'a> SvgColorReplacer<'a> {
    fn new(colors: &'a AvatarColors, derived_colors: &'a DerivedColors) -> Self {
        Self {
            colors,
            derived_colors,
        }
    }

    /// 替换 SVG 中的颜色占位符
    fn replace_colors(&self, svg: &str) -> String {
        let replacements = [
            ("${skinColor}", self.colors.skin.html()),
            ("${hairColor}", self.derived_colors.hair.html()),
            ("${eyesColor}", self.colors.eyes.html()),
            ("${eyebrowsColor}", self.derived_colors.eyebrows.html()),
            ("${mustacheColor}", self.derived_colors.mustache.html()),
            ("${mustacheColorAlpha}", self.derived_colors.mustache.a.to_string()),
            ("${mouthColor}", self.derived_colors.mouth.html()),
            ("${glassesColor}", self.colors.glasses.html()),
            ("${clothesColor}", self.colors.clothes.html()),
            ("${hatColor}", self.colors.hat.html()),
        ];

        let mut result = svg.to_string();
        for (from, to) in replacements {
            result = result.replace(from, &to);
        }
        result
    }
}

fn male_avatar(seed: u64, mood: &str) -> String {
    let mut g = linear_congruential_generator(seed);
    
    // 1. 创建基础颜色
    let colors = AvatarColors {
        skin: to_rgb(g.pick_one(&[
            "#FFDBAC", "#F5CFA0", "#EAC393", "#E0B687", 
            "#CB9E6E", "#B68655", "#A26D3D", "#8D5524",
        ])),
        hair: to_rgb(g.pick_one(&[
            "#090806", "#2c222b", "#71635a", "#b7a69e", "#b89778", 
            "#a56b46", "#b55239", "#8d4a43", "#91553d", "#533d32", 
            "#3b3024", "#554838", "#4e433f", "#504444", "#6a4e42", 
            "#a7856a", "#977961",
        ])),
        eyes: to_rgb(g.pick_one(&[
            "#76778b", "#697b94", "#647b90", "#5b7c8b", "#588387",
        ])),
        clothes: to_rgb(g.pick_one(&[
            "#5bc0de", "#5cb85c", "#428bca", "#03396c", "#005b96", 
            "#6497b1", "#1b85b8", "#5a5255", "#559e83", "#ae5a41", 
            "#c3cb71", "#666547", "#ffe28a",
        ])),
        hat: to_rgb(g.pick_one(&[
            "#18293b", "#2e1e05", "#989789", "#3d6ba7", "#517459", "#a62116",
        ])),
        glasses: to_rgb(g.pick_one(&[
            "#5f705c", "#43677d", "#5e172d", "#ffb67a", "#a04b5d", 
            "#191919", "#323232", "#4b4b4b",
        ])),
    };

    // 2. 创建派生颜色
    let derived_colors = DerivedColors {
        hair: colors.hair.brighter_or_darker_than(&colors.skin, 17.0),
        eyebrows: colors.hair.darker_than(&colors.skin, 7.0)
                            .darker_than(&colors.hair, 10.0),
        mustache: colors.hair.darker_than(&colors.skin, 7.0)
                            .with_alpha(g.pick_one_float(&[1.0, 0.75, 0.5])),
        mouth: to_rgb(g.pick_one(&["#eec1ad", "#dbac98", "#d29985"]))
                    .brighter_or_darker_than(&colors.skin, 10.0),
    };
    
    // 3. 创建颜色替换器
    let replacer = SvgColorReplacer::new(&colors, &derived_colors);


    let mood = if mood.is_empty() {
        g.pick_one(&["sad", "happy", "surprised"])
    } else {
        mood
    };

    let mouth = if mood == "sad" {
        "<path d='M8 13h3v1H8v-1z' fill='${mouthColor}'/>\
         <path d='M8 13h4v1H8v-1z' fill='${mouthColor}'/>\
         <path d='M9 13h2v1H9v-1z' fill='${mouthColor}'/>\
         <path d='M8 12v1h3v1h1v-1h-1v-1H8z' fill='${mouthColor}'/>\
         <path d='M8 13v1h1v-1h3v-1H9v1H8z' fill='${mouthColor}'/>"
    } else if mood == "happy" {
        "<path d='M7 12v1h1v1h4v-1H8v-1H7z' fill='${mouthColor}'/>\
         <path d='M10 12v1H9v1h2v-2h-1z' fill='${mouthColor}'/>\
         <path d='M8 13v1h4v-1h1v-1h-1v1H8z' fill='${mouthColor}'/>\
         <path d='M8 12v2h4v-2H8z' fill='#FFF'/>"
    } else if mood == "surprised" {
        "<path d='M9 12v2h2v-2H9z' fill='${mouthColor}'/>\
         <path d='M9 13v1h1v-1H9z' fill='${mouthColor}'/>"
    } else {
        ""
    };

    let mut s = String::new();
    s.push_str("<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' style='isolation:isolate' viewBox='0 0 20 20' version='1.1' shape-rendering='crispEdges'>");
    
    // Head
    s.push_str("<path d='M8 15v1H4v1H3v3h14v-3h-1v-1h-4v-1h3v-1h1v-1h1v-3h1V7h-1V4h-1V3h-1V2H5v1H4v1H3v3H2v3h1v3h1v1h1v1h3z' fill='${skinColor}'/><path d='M5 15v-1H4v-1H3v-3H2V7h1V4h1V3h1V2h10v1h1v1h1v3h1v3h-1v3h-1v1h-1v1H5z' fill='#FFF' fill-opacity='.1'/>");
    
    // Eyes
    s.push_str(&g.pick_one(&[
        "<path d='M5 9V7h3v2H5zm7-2h3v2h-3V7z' fill='#FFF'/><path d='M7 8v1h1V8H7zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill='#FFF'/><path d='M6 8h1v1H6V8zm7 1V8h1v1h-1z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill='#FFF'/><path d='M7 8h1v1H7V8zm5 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M6 7h1v1h1v1H6V7zm6 0h1v1h1v1h-2V7z' fill='#FFF'/><path d='M6 8h1v1H6V8zm6 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 8h2v1H5V8zm7 0h2v1h-2V8z' fill='#FFF'/><path d='M7 8h1v1H7V8zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M6 8h1v1H6V8zm7 0h1v1h-1V8z' fill='#FFF'/><path d='M7 8h1v1H7V8zm5 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7v1h3V7H5zm7 0h3v1h-3V7z' fill='#FFF'/><path d='M5 9V8h1V7h1v1h1v1H5zm7 0V8h1V7h1v1h1v1h-3z' fill='${eyesColor}'/><path d='M5 9V8h1V7h1v1h1v1H7V8H6v1H5zm7 0V8h1V7h1v1h1v1h-1V8h-1v1h-1z' fill='#FFF' fill-opacity='.5'/>",
        "<path d='M5 8h3v1H5V8zm7 0h3v1h-3V8z' fill='#FFF'/><path d='M6 8h1v1H6V8zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill='#FFF'/><path d='M5 8h2v1H5V8zm7 0h2v1h-2V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill='#FFF'/><path d='M6 8h1v1H6V8zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill='#FFF'/><path d='M6 7h2v2H6V7zm7 0h2v2h-2V7z' fill='${eyesColor}'/><path d='M6 7v1h1v1h1V8H7V7H6zm7 0v1h1v1h1V8h-1V7h-1z' fill='#FFF' fill-opacity='.4'/><path d='M7 7v1h1V7H7zm7 0h1v1h-1V7z' fill='#FFF' fill-opacity='.7'/>",
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill='#FFF'/><path d='M5 8h2v1H5V8zm7 0h2v1h-2V8z' fill='${eyesColor}'/><path d='M5 8h1v1H5V8zm7 0h1v1h-1V8z' fill='#FFF' fill-opacity='.7'/>",
        "<path d='M6 7h1v2H5V8h1V7zm7 0h1v2h-2V8h1V7z' fill='#FFF'/><path d='M7 7v1H6v1h2V7H7zm7 0v1h-1v1h2V7h-1z' fill='${eyesColor}'/><path d='M7 7v1h1V7H7zM6 8v1h1V8H6zm8-1v1h1V7h-1zm-1 1v1h1V8h-1z' fill='#FFF' fill-opacity='.5'/>",
    ]));
    
    // Eyebrows
    s.push_str(&g.pick_one(&[
        "<path d='M7 5v1H5v1H4V6h1V5h2zm7 0v1h-2v1h-1V6h1V5h2z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M8 4v1H7v1H5V5h2V4h1zm4 0h1v1h2v1h-2V5h-1V4z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5h3v2H8V6H6V5zm5 0h3v1h-2v1h-1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M5 5h3v1h1v1H8V6H5V5zm10 0h-3v1h-1v1h1V6h3V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5H4v2h1V6h1V5zm8 0h2v2h-1V6h-1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M5 6h2v1H5V6zm8 0h2v1h-2V6z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5h1v1h1v1H5V6h1V5zm7 0h1v1h1v1h-3V6h1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5h2v1h1v1H8V6H6V5zm8 0h-2v1h-1v1h1V6h2V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M12 7V6h1V5h1v1h1v1h-1V6h-1v1h-1zM5 7V6h1V5h1v1h1v1H7V6H6v1H5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M7 5v1H5v1H4V6h1V5h2zm6 0h2v1h1v1h-1V6h-2V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M4 7V5h3v1H5v1H4zm12-2v2h-1V6h-2V5h3z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M7 5h1v1h1v1H8V6H7V5zm6 0v1h-1v1h-1V6h1V5h  fill='${eyebrowsColor}'/>",
        "<path d='M7 5h1v1h1v1H8V6H7V5zm6 0v1h-1v1h-1V6h1V5h1z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M4 7V6h1V5h1v1H5v1H4zm10-2h1v1h1v1h-1V6h-1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
    ]));
    
    // 胡子部分 (50% 概率)
    let mustache_options = [
        "<path d='M3 10v3h1v1h1v1h10v-1h1v-1h1v-3h-3v1H6v-1H3z' id='Path' fill='${mustacheColor}' fill-opacity='${mustacheColorAlpha}'/>",
        "<path d='M3 13h1v1h1v1h10v-1h1v-1h1v-3h-1v1h-1v1H5v-1H4v-1H3v3z' id='Path' fill='${mustacheColor}' fill-opacity='${mustacheColorAlpha}'/>",
        "<path d='M3 11v2h1v1h1v1h10v-1h1v-1h1v-2H3z' id='Path' fill='${mustacheColor}' fill-opacity='${mustacheColorAlpha}'/>",
        "<path d='M3 7v6h1v1h1v1h10v-1h1v-1h1V7h-1v2h-1v1h-1v1H6v-1H5V9H4V7H3z' id='Path' fill='${mustacheColor}' fill-opacity='${mustacheColorAlpha}'/>"
    ];
    let selected_mustache = g.pick_one(&mustache_options);
    s.push_str(&g.pick_a_or_b(0.5, selected_mustache, ""));
    
    // Mouth
    s.push_str(&mouth);
    
    // 眼镜部分 (25% 概率)
    let glasses_options = [
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 7h1v1H7V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M12 10V7h3v3h-3zm-1-4v1H9V6H4v1H3v1h1v3h5V8h2v3h5V8h1V7h-1V6h-5zm-6 4V7h3v3H5z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7h1v1H3V7zm6 0h2v1H9V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 7h1v1H7V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M5 7v2h3V7H5zM4 6v1H3v1h1v1h1v1h3V9h1V8h2v1h1v1h3V9h1V8h1V7h-1V6h-5v1H9V6H4zm8 1v2h3V7h-3z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7h1v1H3V7zm6 0h2v1H9V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 8h3v1H5V8zm7 0h3v1h-3V8z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 8h1v1H7V8zm7 0h1v1h-1V8z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M5 8v1h3V8H5zM3 7v1h1v1h1v1h3V9h1V8h2v1h1v1h3V9h1V8h1V7H3zm9 1v1h3V8h-3z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7v1h1V7H3zm6 0v1h2V7H9zm7 0v1h1V7h-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 7h1v1H7V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M12 7v2h3V7h-3zM8 6H5v1H3v1h1v1h1v1h3V9h1V8h2v1h1v1h3V9h1V8h1V7h-2V6h-3v1H8V6zM5 7v2h3V7H5z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7h1v1H3V7zm6 0h2v1H9V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 8H3V7h1V6h5v1h2V6h5v1h1v1h-1v2h-5V8H9v2H4V8zm1 0V7h3v2H5V8zm7-1v2h3V7h-3z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M14 7h1v1h-1V7zM7 7h1v1H7V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M3 8V7h1v1H3zm6-1v1h2V7H9zm7 0v1h1V7h-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 8H3V7h14v1h-1v2h-5V8H9v2H4V8zm1 0h3v1H5V8zm7 0h3v1h-3V8z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M5 8h3v1H5V8zm7 0h3v1h-3V8z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 8v1h1V8H7zm7 0v1h1V8h-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M3 7v1h1V7H3zm13 0v1h1V7h-1zM9 7v1h2V7H9z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
    ];
    let selected_glasses = g.pick_one(&glasses_options);
    s.push_str(&g.pick_a_or_b(0.25, selected_glasses, ""));

    // Clothes
    s.push_str(&g.pick_one(&[
        "<path d='M3 20v-3h1v-1h4v-1h4v1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M3 20v-3h1v-1h12v1h1v3H3z' fill='#FFF' fill-opacity='.2'/><path d='M12 19v-1h3v1h-3z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h12v1h1v3H3z' fill='${clothesColor}'/><path d='M5 20v-2h1v-1h8v1h1v2h-2v-1h-2v1H9v-1H7v1H5z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h12v1h1v3H3z' fill='${clothesColor}'/><path d='M8 16H4v1H3v3h14v-3h-1v-1h-4v1h1v1h-1v1h-1v-1H9v1H8v-1H7v-1h1v-1z' fill='#FFF' fill-opacity='.2'/><path d='M9 16v1h2v-1H9z' fill='#FFF'/>",
        "<path d='M3 20v-3h1v-1h12v1h1v3H3z' fill='${clothesColor}'/><path d='M9 16H4v1H3v3h6v-2H8v-1h1v-1zm2 0h5v1h1v3h-6v-2h1v-1h-1v-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.3'/>",
        "<path d='M3 20v-3h1v-1h3v2h6v-2h3v1h1v3H3z' fill='${clothesColor}'/><path d='M5 16H4v1H3v3h2v-4zm1 0h1v2h6v-2h1v4H6v-4zm9 0h1v1h1v3h-2v-4z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h4v1h1v1h2v-1h1v-1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M4 17v-1h3v1H4zm9 0v-1h3v1h-3z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h3v-1h1v1h1v1h2v-1h1v-1h1v1h3v1h1v3H3z' fill='${clothesColor}'/><path d='M6 16H4v1H3v3h6v-2H8v-1H6v-1zm2 0h1-1zm3 0h1-1zm2 0h1v1h-2v1h-1v2h6v-3h-1v-1h-3z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 16H4v1H3v3h14v-3h-1v-1h-3v1H7v-1H5z' fill='${clothesColor}'/><path d='M10 20v-1h3v1h-3z' fill='#FFF' fill-opacity='.5'/><path d='M5 16H4v1H3v3h1v-1h1v-3zm1 0h1v1h6v-1h1v2H6v-2zm9 0h1v1h1v3h-1v-1h-1v-3z' fill-rule='evenodd' fill='#FFF' fill-opacity='.8'/>",
        "<path d='M3 20v-3h1v-1h4v1h4v-1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M3 20v-1h1v1H3zm2 0v-1h1v1H5zm2 0v-1h1v1H7zm2 0v-1h1v1H9zm2 0v-1h1v1h-1zm2 0v-1h1v1h-1zm2 0v-1h1v1h-1zm1-2h1v1h-1v-1zm-2 0h1v1h-1v-1zm-2 0h1v1h-1v-1zm-2 0h1v1H8v-1zm-2 0h1v1H6v-1zm-2 0h1v1H4v-1zm-1-1h1v1H3v-1zm2 0h1v1H5v-1zm2 0h1v1H7v-1zm2 0h1v1H9v-1zm2 0h1v1h-1v-1zm2 0h1v1h-1v-1zm2 0h1v1h-1v-1zM4 16h1v1H4v-1zm2 0h1v1H6v-1zm6 0h1v1h-1v-1zm2 0h1v1h-1v-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h4v1h4v-1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M3 20v-2h1v2H3zm3 0v-2h2v2H6zm4 0v-2h2v2h-2zm4 0v-2h2v2h-2zm2-3v1h1v-1h-1zm-2 1v-2h-2v2h2zm-6-1v1h2v-1H8zm-4-1v2h2v-2H4z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h4v1h4v-1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M3 19h14v1H3v-1zm0-2h14v1H3v-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h4v1h4v-1h4v1h1v3H3z' fill='${clothesColor}'/>",
        "<path d='M3 20v-3h1v-1h12v1h1v3H3z' fill='${clothesColor}'/>",
    ]));
    
    // 头发部分 (95% 概率)
    let hair_options = [
        "<path d='M3 3v2h1V4h1V3h10v1h1v1h1V3h-1V2H4v1H3z' fill='${hairColor}'/>",
        "<path d='M5 2h10v1h1v1h1v3h-1V6h-1V5h-1V4h-4v1H8v1H7v1H4V6H3V4h1V3h1V2z' fill='${hairColor}'/>",
        "<path d='M3 6h1V4h1V3h2v1h1v1h4V4h1V3h2v1h1v2h1V4h-1V3h-1V2H5v1H4v1H3v2z' fill='${hairColor}'/>",
        "<path d='M3 8h1V5h12v3h1V4h-1V3h-1V2H5v1H4v1H3v4z' fill='${hairColor}'/>",
        "<path d='M2 4v1h1v1h2V4h1V2H4v1H3v1H2zm6-1h2v1h2V3h1V2H8v1zm6 1h1v2h2V5h1V4h-1V3h-1V2h-2v2z' fill-rule='evenodd' fill='${hairColor}'/>",
        "<path d='M3 7h1V5h2V3h8v1h1v1h1v2h1V3h-2V2h-2V1h-1v1h-2V1H9v1H8V1H7v1H5v1H4v1H3v3z' fill='${hairColor}'/>",
        "<path d='M8 2h4v1h-1v1H9V3H8V2z' fill='${hairColor}'/>",
        "<path d='M9 0v1H8v1h4V1h-1V0H9z' fill='${hairColor}'/>",
        "<path d='M3 7h1V5h2V4h2V3h1v1h2v1h2v1h2v1h2V4h-1V3h-1V2H5v1H4v1H3v3z' fill='${hairColor}'/>",
        "<path d='M4 4h12V3h-1V2H5v1H4v1z' fill='${hairColor}'/>",
        "<path d='M2 7h1V5h2V4h1V3h1v1h2V3h4V2h1v1h1v1h1v1h1v2h1V6h1V4h-1V3h-1V2h-1V1h-1V0h-1v1h-2V0h-1v1H9V0H8v1H7V0H5v1H4v1H2v5z' fill='${hairColor}'/>",
        "<path d='M0 7h1v5h1v1h1V9h1V7h1V6h1V4h1V3h7v1h1v1h1v1h1v7h1v-2h1V7h-1V6h1V4h-1v1h-1V3h1V2h-1v1h-1V2h-2V1h-1V0h-1v1H5V0H4v1H3V0H2v1h1v2H2V2H1v1h1v1H1v2H0v1z' fill='${hairColor}'/>",
        "<path d='M5 2v1H4v1H3v3h2V6h1V5h6V4h1V3h1v1h-1v1h1v1h1v1h2V4h-1V3h-1V2H5z' fill='${hairColor}'/>",
    ];
    let selected_hair = g.pick_one(&hair_options);
    s.push_str(&g.pick_a_or_b(0.95, selected_hair, ""));


    
    // Hat (5% chance)
    // 帽子部分 (5% 概率)
    let hat_options = [
        "<path d='M4 0v2H2v2h16V2h-2V0H4z' fill='${hatColor}'/>",
        "<path d='M4 3H2v1h16V3h-2V0H4v3z' fill='${hatColor}'/>",
        "<path d='M2 2v2h16V2h-1V1h-1V0H4v1H3v1H2z' fill='${hatColor}'/>",
        "<path d='M6 0v1H5v1H4v2h14V3h-2V2h-1V1h-1V0H6z' fill='${hatColor}'/>",
        "<path d='M2 4V2h2V0h12v2h2v2H2z' fill='${hatColor}'/><path d='M4 0v2h12V0H4z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M2 4V3h2V0h12v3h2v1H2z' fill='${hatColor}'/><path d='M4 0v3h12V0H4z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M2 4V2h1V1h1V0h12v1h1v1h1v2H2z' fill='${hatColor}'/><path d='M3 1v1h14V1H3zM2 3v1h16V3H2z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M14 0H6v1H5v1H4v2h14V3h-2V2h-1V1h-1V0z' fill='${hatColor}'/><path d='M5 3h1V2h1V1h1V0H7v1H6v1H5v1z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 0v2H2v2h16V2h-2V0H4z' fill='${hatColor}'/><path d='M15 3V0h-1v3h1z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 0v3H2v1h16V3h-2V0H4z' fill='${hatColor}'/><path d='M15 3V0h-1v3h1zm-2-3v2h-1V0h1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M2 2v2h16V2h-1V1h-1V0H4v1H3v1H2z' fill='${hatColor}'/><path d='M15 0v4h-1V0h1zm-2 0v4h-1V0h1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 2H4v2h14V3h-2V2h-1V1h-1V0H6v1H5v1z' fill='${hatColor}'/><path d='M14 2h-3v1h3V2z' fill='#FFF' fill-opacity='.2'/>",
    ];
    let selected_hat = g.pick_one(&hat_options);
    s.push_str(&g.pick_a_or_b(0.05, selected_hat, ""));
    s.push_str("</svg>");
    // 使用替换器替换颜色
    s = replacer.replace_colors(&s);
    s
}

fn female_avatar(seed: u64, mood: &str) -> String {
    let mut g = linear_congruential_generator(seed);

    // 1. 创建基础颜色
    let colors = AvatarColors {
        skin: to_rgb(g.pick_one(&[
            "#FFDBAC", "#F5CFA0", "#EAC393", "#E0B687", "#CB9E6E", "#B68655", "#A26D3D", "#8D5524",
        ])),
        hair: to_rgb(g.pick_one(&[
            "#090806", "#2c222b", "#71635a", "#b7a69e", "#d6c4c2", "#cabfb1", "#dcd0ba", "#fff5e1",
            "#e6cea8", "#e5c8a8", "#debc99", "#b89778", "#a56b46", "#b55239", "#8d4a43", "#91553d",
            "#533d32", "#3b3024", "#554838", "#4e433f", "#504444", "#6a4e42", "#a7856a", "#977961",
        ])),
        eyes: to_rgb(g.pick_one(&[
            "#76778b", "#697b94", "#647b90", "#5b7c8b", "#588387",
        ])),
        clothes: to_rgb(g.pick_one(&[
            "#d11141", "#00b159", "#00aedb", "#f37735", "#ffc425", "#740001", "#ae0001", "#eeba30",
            "#96ceb4", "#ffeead", "#ff6f69", "#ffcc5c", "#88d8b0",
        ])),
        hat: to_rgb(g.pick_one(&[
            "#cc6192", "#2663a3", "#a62116", "#3d8a6b", "#614f8a",
        ])),
        glasses: to_rgb(g.pick_one(&[
            "#5f705c", "#43677d", "#5e172d", "#ffb67a", "#a04b5d", "#191919", "#323232", "#4b4b4b",
        ])),
    };

    // 2. 创建派生颜色
    let derived_colors = DerivedColors {
        hair: colors.hair.brighter_or_darker_than(&colors.skin, 17.0),
        eyebrows: colors.hair.darker_than(&colors.skin, 7.0)
                            .darker_than(&colors.hair, 10.0),
        mustache: to_rgb(g.pick_one(&[
            "#daa520", "#ffd700", "#eee8aa", "#fafad2", "#d3d3d3", "#a9a9a9",
        ])), // 这里用 accessories_color 替代 mustache
        mouth: to_rgb(g.pick_one(&[
            "#dbac98", "#d29985", "#c98276", "#e35d6a", "#e32153", "#de0f0d",
        ])).brighter_or_darker_than(&colors.skin, 10.0),
    };
    
    // 3. 创建颜色替换器
    let replacer = SvgColorReplacer::new(&colors, &derived_colors);

    let mood = if mood.is_empty() {
        g.pick_one(&["sad", "happy", "surprised"])
    } else {
        mood
    };

    let mouth = if mood == "sad" {
        "<path d='M9 11v1H8v1h4v-1h-1v-1H9z' fill='${mouthColor}'/>\
         <path d='M11 11v1H9v1H8v-1h1v-1h2z' fill='${mouthColor}'/>\
         <path d='M9 12h2v1H9v-1z' fill='${mouthColor}'/>\
         <path d='M9 12v1h1v1h1v-2H9z' fill='${mouthColor}'/>"
    } else if mood == "happy" {
        "<path d='M9 11v2h2v-1h-1v-1H9z' fill='${mouthColor}'/><path d='M11 13v-1h-1v-1H9v1h1v1h1z' fill='#FFF' fill-opacity='.2'/>\
         <path d='M10 11v1H9v1h2v-2h-1z' fill='${mouthColor}'/>\
         <path d='M8 11v1h1v1h2v-1h1v-1H8z' fill='${mouthColor}'/>\
         <path d='M9 12v1h2v-1h1v-1h-1v1H9z' fill='${mouthColor}'/>\
         <path d='M8 11v1h1v1h2v-1H9v-1H8z' fill='${mouthColor}'/>\
         <path d='M8 12v1h1v1h2v-1h1v-1h-1v-1H9v1H8z' fill='${mouthColor}'/><path d='M9 12v1h2v-1H9z' fill='#FFF'/>\
         <path d='M8 12v1h1v1h2v-1h1v-1h-1v-1H9v1H8z' fill='${mouthColor}'/><path d='M9 12v1h2v-1H9z' fill='#FFF' fill-opacity='.2'/>"
    } else if mood == "surprised" {
        "<path d='M9 12v1h1v-1H9z' fill='${mouthColor}'/>\
         <path d='M9 11v2h2v-2H9z' fill='${mouthColor}'/>"
    } else {
        ""
    };

    let mut s = String::new();
    s.push_str("<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' style='isolation:isolate' viewBox='0 0 20 20' version='1.1' shape-rendering='crispEdges'>");
    
    // Head
    s.push_str("<path d='M3 20v-3h1v-1h4v-2H6v-1H5v-1H4v-1H3V9H2V7h1V4h1V3h1V2h10v1h1v1h1v3h1v2h-1v2h-1v1h-1v1h-1v1h-2v2h4v1h1v3H3z' fill='${skinColor}'/><path d='M14 14v-1h1v-1h1v-1h1V9h1V7h-1V4h-1V3h-1V2H5v1H4v1H3v3H2v2h1v2h1v1h1v1h1v1h8z' fill='#FFF' fill-opacity='.1'/>");
    
    // Eyes
    s.push_str(&g.pick_one(&[
        "<path d='M5 9V7h3v2H5zm7-2h3v2h-3V7z' fill='#FFF'/><path d='M7 8v1h1V8H7zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill='#FFF'/><path d='M6 8h1v1H6V8zm7 1V8h1v1h-1z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill='#FFF'/><path d='M7 8h1v1H7V8zm5 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M6 7h1v1h1v1H6V7zm6 0h1v1h1v1h-2V7z' fill='#FFF'/><path d='M6 8h1v1H6V8zm6 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 8h2v1H5V8zm7 0h2v1h-2V8z' fill='#FFF'/><path d='M7 8h1v1H7V8zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M6 8h1v1H6V8zm7 0h1v1h-1V8z' fill='#FFF'/><path d='M7 8h1v1H7V8zm5 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7v1h3V7H5zm7 0h3v1h-3V7z' fill='#FFF'/><path d='M5 9V8h1V7h1v1h1v1H5zm7 0V8h1V7h1v1h1v1h-3z' fill='${eyesColor}'/><path d='M5 9V8h1V7h1v1h1v1H7V8H6v1H5zm7 0V8h1V7h1v1h1v1h-1V8h-1v1h-1z' fill='#FFF' fill-opacity='.5'/>",
        "<path d='M5 8h3v1H5V8zm7 0h3v1h-3V8z' fill='#FFF'/><path d='M6 8h1v1H6V8zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill='#FFF'/><path d='M5 8h2v1H5V8zm7 0h2v1h-2V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill='#FFF'/><path d='M6 8h1v1H6V8zm7 0h1v1h-1V8z' fill='${eyesColor}'/>",
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill='#FFF'/><path d='M6 7h2v2H6V7zm7 0h2v2h-2V7z' fill='${eyesColor}'/><path d='M6 7v1h1v1h1V8H7V7H6zm7 0v1h1v1h1V8h-1V7h-1z' fill='#FFF' fill-opacity='.4'/><path d='M7 7v1h1V7H7zm7 0h1v1h-1V7z' fill='#FFF' fill-opacity='.7'/>",
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill='#FFF'/><path d='M5 8h2v1H5V8zm7 0h2v1h-2V8z' fill='${eyesColor}'/><path d='M5 8h1v1H5V8zm7 0h1v1h-1V8z' fill='#FFF' fill-opacity='.7'/>",
        "<path d='M6 7h1v2H5V8h1V7zm7 0h1v2h-2V8h1V7z' fill='#FFF'/><path d='M7 7v1H6v1h2V7H7zm7 0v1h-1v1h2V7h-1z' fill='${eyesColor}'/><path d='M7 7v1h1V7H7zM6 8v1h1V8H6zm8-1v1h1V7h-1zm-1 1v1h1V8h-1z' fill='#FFF' fill-opacity='.5'/>",
    ]));
    
    // Eyebrows
    s.push_str(&g.pick_one(&[
        "<path d='M7 5v1H5v1H4V6h1V5h2zm7 0v1h-2v1h-1V6h1V5h2z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M8 4v1H7v1H5V5h2V4h1zm4 0h1v1h2v1h-2V5h-1V4z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5h3v2H8V6H6V5zm5 0h3v1h-2v1h-1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M5 5h3v1h1v1H8V6H5V5zm10 0h-3v1h-1v1h1V6h3V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5H4v2h1V6h1V5zm8 0h2v2h-1V6h-1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M5 6h2v1H5V6zm8 0h2v1h-2V6z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5h1v1h1v1H5V6h1V5zm7 0h1v1h1v1h-3V6h1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M6 5h2v1h1v1H8V6H6V5zm8 0h-2v1h-1v1h1V6h2V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M12 7V6h1V5h1v1h1v1h-1V6h-1v1h-1zM5 7V6h1V5h1v1h1v1H7V6H6v1H5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M7 5v1H5v1H4V6h1V5h2zm6 0h2v1h1v1h-1V6h-2V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M4 7V5h3v1H5v1H4zm12-2v2h-1V6h-2V5h3z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M7 5h1v1h1v1H8V6H7V5zm6 0v1h-1v1h-1V6h1V5h  fill='${eyebrowsColor}'/>",
        "<path d='M7 5h1v1h1v1H8V6H7V5zm6 0v1h-1v1h-1V6h1V5h1z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
        "<path d='M4 7V6h1V5h1v1H5v1H4zm10-2h1v1h1v1h-1V6h-1V5z' fill-rule='evenodd' fill='${eyebrowsColor}'/>",
    ]));
    
    // Accessories (15% chance)
    let accessortis_options = [
        "<path d='M2 9v1h1V9H2zm15 0v1h1V9h-1z' fill-rule='evenodd' fill='${accessoriesColor}'/>",
        "<path d='M2 9v2h1V9H2zm15 0h1v2h-1V9z' fill-rule='evenodd' fill='${accessoriesColor}'/>",
        "<path d='M2 9v2h1V9H2zm15 0h1v2h-1V9z' fill='${accessoriesColor}'/><path d='M2 9v1h1V9H2zm15 0h1v1h-1V9z' fill='#FFF' fill-opacity='.4'/>",
        "<path d='M1 9v3h3V9H1zm1 1v1h1v-1H2zm14-1v3h3V9h-3zm1 1v1h1v-1h-1z' fill-rule='evenodd' fill='${accessoriesColor}'/>",
    ];
    let selected_accessortis = g.pick_one(&accessortis_options);
    s.push_str(&g.pick_a_or_b(0.25, selected_accessortis, ""));
    
    // Mouth
    s.push_str(&mouth);
    
    // Glasses (25% chance)
    let glasses_options = [
        "<path d='M3 8V7h1V6h2v1h1V6h2v1h2V6h2v1h1V6h2v1h1v1h-1v1h-1v1h-1v1h-1v-1h-1V9h-1V8H9v1H8v1H7v1H6v-1H5V9H4V8H3z' fill='${glassesColor}'/><path d='M3 7v1h1V7h1V6H4v1H3zm5-1v1h1v1h2V7h1V6h-1v1H9V6H8zm7 0v1h1v1h1V7h-1V6h-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 7h3v3H5V7zm7 0h3v3h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 7h1v1H7V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M12 10V7h3v3h-3zm-1-4v1H9V6H4v1H3v1h1v3h5V8h2v3h5V8h1V7h-1V6h-5zm-6 4V7h3v3H5z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7h1v1H3V7zm6 0h2v1H9V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 7h1v1H7V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M5 7v2h3V7H5zM4 6v1H3v1h1v1h1v1h3V9h1V8h2v1h1v1h3V9h1V8h1V7h-1V6h-5v1H9V6H4zm8 1v2h3V7h-3z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7h1v1H3V7zm6 0h2v1H9V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 8h3v1H5V8zm7 0h3v1h-3V8z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 8h1v1H7V8zm7 0h1v1h-1V8z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M5 8v1h3V8H5zM3 7v1h1v1h1v1h3V9h1V8h2v1h1v1h3V9h1V8h1V7H3zm9 1v1h3V8h-3z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7v1h1V7H3zm6 0v1h2V7H9zm7 0v1h1V7h-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 7h1v1H7V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M12 7v2h3V7h-3zM8 6H5v1H3v1h1v1h1v1h3V9h1V8h2v1h1v1h3V9h1V8h1V7h-2V6h-3v1H8V6zM5 7v2h3V7H5z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M3 7h1v1H3V7zm6 0h2v1H9V7zm7 0h1v1h-1V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 8H3V7h1V6h5v1h2V6h5v1h1v1h-1v2h-5V8H9v2H4V8zm1 0V7h3v2H5V8zm7-1v2h3V7h-3z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M5 7h3v2H5V7zm7 0h3v2h-3V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M14 7h1v1h-1V7zM7 7h1v1H7V7z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M3 8V7h1v1H3zm6-1v1h2V7H9zm7 0v1h1V7h-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 8H3V7h14v1h-1v2h-5V8H9v2H4V8zm1 0h3v1H5V8zm7 0h3v1h-3V8z' fill-rule='evenodd' fill='${glassesColor}'/><path d='M5 8h3v1H5V8zm7 0h3v1h-3V8z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M7 8v1h1V8H7zm7 0v1h1V8h-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M3 7v1h1V7H3zm13 0v1h1V7h-1zM9 7v1h2V7H9z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
    ];
    let selected_glasses = g.pick_one(&glasses_options);
    s.push_str(&g.pick_a_or_b(0.25, selected_glasses, ""));
    
    // Clothes
    s.push_str(&g.pick_one(&[
        "<path d='M3 20v-3h1v-1h12v1h1v3H3z' fill='${clothesColor}'/>",
        "<path d='M4 16v4h4v-1H7v-1H6v-1H5v-1H4zm12 0v4h-4v-1h1v-1h1v-1h1v-1h1z' fill-rule='evenodd' fill='${clothesColor}'/>",
        "<path d='M5 16h1v2h1v1h1v1H5v-4zm9 0h1v4h-3v-1h1v-1h1v-2z' fill-rule='evenodd' fill='${clothesColor}'/>",
        "<path d='M4 20v-2h1v-1h1v-1h2v1h1v1h2v-1h1v-1h2v1h1v1h1v2H4z' fill='${clothesColor}'/>",
        "<path d='M3 20v-3h1v-1h4v1h1v1h2v-1h1v-1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M3 20v-3h1v-1h2v1h1v1h1v1h4v-1h1v-1h1v-1h2v1h1v3H3z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h5v1h2v-1h5v1h1v3H3z' fill='${clothesColor}'/><path d='M3 20v-2h1v1h3v1H3zm14 0v-2h-1v1h-3v1h4z' fill-rule='evenodd' fill='#FFF' fill-opacity='.4'/><path d='M7 16H4v1H3v1h1v1h3v1h6v-1h3v-1h1v-1h-1v-1h-3v1H7v-1z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h4v1h4v-1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M15 20h2v-3h-1v-1h-4v1H8v-1H4v1H3v3h2v-2h10v2z' fill='#FFF' fill-opacity='.4'/>",
        "<path d='M3 20v-3h1v-1h4v1h1v1h2v-1h1v-1h4v1h1v3H3z' fill='${clothesColor}'/><path d='M6 16v1h1v1h1v1h4v-1h1v-1h1v-1h-2v1h-1v1H9v-1H8v-1H6z' fill='#FFF' fill-opacity='.4'/><path d='M13 20v-1h2v1h-2zm1-4v1h2v-1h-2zm-8 0H4v1h2v-1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M3 20v-3h1v-1h3v1h1v1h1v1h2v-1h1v-1h1v-1h3v1h1v3H3z' fill='${clothesColor}'/>",
        "<path d='M3 20v-3h1v-1h2v1h1v1h1v1h1v1H3zm14 0v-3h-1v-1h-2v1h-1v1h-1v1h-1v1h6z' fill-rule='evenodd' fill='${clothesColor}'/>",
        "<path d='M4 16v4h4v-1H7v-1H6v-2H4zM16 20v-4h-2v2h-1v1h-1v1h4z' fill='${clothesColor}'/>",
        "<path d='M3 20v-3h1v-1h3v1h1v1h1v1h2v-1h1v-1h1v-1h3v1h1v3H3z' fill='${clothesColor}'/><path d='M6 16v1h1v1h1v1h1v1h2v-1h1v-1h1v-1h1v-1h-1v1h-1v1h-1v1H9v-1H8v-1H7v-1H6z' fill='#FFF' fill-opacity='.4'/><path d='M15 16v1h-1v1h-1v1h-1v1h-1v-1h1v-1h1v-1h1v-1h1zM5 16v1h1v1h1v1h1v1h1v-1H8v-1H7v-1H6v-1H5z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/><path d='M3 20h1v-3h1v1h1v1h1v1h1v-1H7v-1H6v-1H5v-1H4v1H3v3zm14 0v-3h-1v-1h-1v1h-1v1h-1v1h-1v1h1v-1h1v-1h1v-1h1v3h1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.1'/>",
        "<path d='M3 20v-3h1v-1h5v1h2v-1h5v1h1v3H3z' fill='${clothesColor}'/><path d='M3 17h14v1H3v-1zm0 2v1h14v-1H3z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
    ]));
    
    // Hair
    s.push_str(&g.pick_one(&[
        "<path d='M2 9v6h2v-4H3V9H2zm0-2h2V4h12v3h2V3h-1V2H3v1H2v4zm15 2h1v6h-2v-4h1V9z' fill-rule='evenodd' fill='${hairColor}'/>",
        "<path d='M4 12h1v1H3V4h1V3h1V2h10v1h1v1h1v9h-2v-1h1V5H4v7z' fill='${hairColor}'/>",
        "<path d='M2 17h2v-1h4v-2H6v-1H5v-1H4V4h1V3h1v1h1V3h1v1h1V3h6v1h1v8h-1v1h-1v1h-2v2h4v1h2V3h-1V2h-1V1H4v1H3v1H2v14z' fill='${hairColor}'/>",
        "<path d='M2 13V9h1v2h1v1h1v1H2zm15-4h1v4h-3v-1h1v-1h1V9zM2 7h1V4h1V3h1V2h10v1h1v1h1v3h1V3h-1V2h-1V1H4v1H3v1H2v4z' fill-rule='evenodd' fill='${hairColor}'/>",
        "<path d='M2 14h2V4h12v10h2V3h-1V2h-1V1H4v1H3v1H2v11z' fill='${hairColor}'/>",
        "<path d='M2 14h2V7h1V6h1V5h1V4h1V3h6v1h1v1h1v9h2V3h-1V2h-1V1H4v1H3v1H2v11z' fill='${hairColor}'/>",
        "<path d='M3 4v3h1V4h2v1h1V4h1v1h1V4h7v3h1V4h1V2h-1V1h-2v1H5V1H3v1H2v2h1z' fill='${hairColor}'/>",
        "<path d='M2 10h1V8h1V7h1V6h1V5h1V4h7v1h3V4h-1V3h-1V2H4v1H3v1H2v6z' fill='${hairColor}'/>",
        "<path d='M1 17h3v-1h4v-2H6v-1H5v-1H4v-1H3V9H2V7h1V5h1V4h7V3h1v1h1V3h1v1h1V3h1v2h1v2h1v2h-1v2h-1v1h-1v1h-1v1h-2v2h4v1h3V4h-1V3h-1V2h-1V1H4v1H3v1H2v1H1v13z' fill='${hairColor}'/>",
        "<path d='M2 13h3v-1H4v-1H3V9H2v4zm13 0h3V9h-1v2h-1v1h-1v1zm2-6h1V3h-1V2h-1V1H4v1H3v1H2v4h1V5h1V4h1V3h1V2h8v1h1v1h1v1h1v2z' fill-rule='evenodd' fill='${hairColor}'/>",
        "<path d='M2 9v3h1v1H2v2H1v1h1v-1h1v-1h1v-1h1v-1H4v-1H3V9H2zm0-2h1V6h1V5h2V4h9v1h1v1h1v1h1V3h-1V2h-1V1H4v1H3v1H2v4z' fill-rule='evenodd' fill='${hairColor}'/>",
        "<path d='M1 15h5v-2H5v-1H4v-1H3V6h2V5h2V4h1V3h6v1h1v1h1v1h1v5h-1v1h-1v1h-1v2h5V4h-1V3h-1V2h-1V1H4v1H3v1H2v1H1v11z' fill='${hairColor}'/>",
        "<path d='M2 9v3h1v1H2v2H1v1h1v-1h1v-1h1v-1h1v-1H4v-1H3V9H2zm16 0v3h-1v1h1v2h1v1h-1v-1h-1v-1h-1v-1h-1v-1h1v-1h1V9h1zm-1-2h1V3h-1V2h-1V1H4v1H3v1H2v4h1V6h1V5h2V4h9v1h1v1h1v1z' fill-rule='evenodd' fill='${hairColor}'/>",
    ]));
    
    // Hat (5% chance)
    let hat_options = [
        "<path d='M4 0v2H2v2h16V2h-2V0H4z' fill='${hatColor}'/>",
        "<path d='M4 3H2v1h16V3h-2V0H4v3z' fill='${hatColor}'/>",
        "<path d='M2 2v2h16V2h-1V1h-1V0H4v1H3v1H2z' fill='${hatColor}'/>",
        "<path d='M6 0v1H5v1H4v2h14V3h-2V2h-1V1h-1V0H6z' fill='${hatColor}'/>",
        "<path d='M2 4V2h2V0h12v2h2v2H2z' fill='${hatColor}'/><path d='M4 0v2h12V0H4z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M2 4V3h2V0h12v3h2v1H2z' fill='${hatColor}'/><path d='M4 0v3h12V0H4z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M2 4V2h1V1h1V0h12v1h1v1h1v2H2z' fill='${hatColor}'/><path d='M3 1v1h14V1H3zM2 3v1h16V3H2z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M14 0H6v1H5v1H4v2h14V3h-2V2h-1V1h-1V0z' fill='${hatColor}'/><path d='M5 3h1V2h1V1h1V0H7v1H6v1H5v1z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 0v2H2v2h16V2h-2V0H4z' fill='${hatColor}'/><path d='M15 3V0h-1v3h1z' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M4 0v3H2v1h16V3h-2V0H4z' fill='${hatColor}'/><path d='M15 3V0h-1v3h1zm-2-3v2h-1V0h1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M2 2v2h16V2h-1V1h-1V0H4v1H3v1H2z' fill='${hatColor}'/><path d='M15 0v4h-1V0h1zm-2 0v4h-1V0h1z' fill-rule='evenodd' fill='#FFF' fill-opacity='.2'/>",
        "<path d='M5 2H4v2h14V3h-2V2h-1V1h-1V0H6v1H5v1z' fill='${hatColor}'/><path d='M14 2h-3v1h3V2z' fill='#FFF' fill-opacity='.2'/>",
    ];
    let selected_hat = g.pick_one(&hat_options);
    s.push_str(&g.pick_a_or_b(0.05, selected_hat, ""));
    
    s.push_str("</svg>");
    
    // 最后使用替换器替换颜色
    s = replacer.replace_colors(&s);
    
    s
}
