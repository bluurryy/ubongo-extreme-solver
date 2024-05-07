const COLORS: &[&str] = &[
    "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f", "#bcbd22", "#17becf",
];

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn hex_to_u8s(hex: &str) -> [u8; 3] {
    [0, 1, 2].map(|i| u8::from_str_radix(&hex[(1 + i * 2)..(1 + (i + 1) * 2)], 16).unwrap())
}

fn lerp_hex(a: &str, b: &str, t: f32) -> String {
    let a = hex_to_u8s(a).map(|i| i as f32);
    let b = hex_to_u8s(b).map(|i| i as f32);
    let [r, g, b] = [0, 1, 2].map(|i| lerp(a[i], b[i], t)).map(|f| f.round() as u8);
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[test]
fn colors_lerped() {
    print!("const COLORS_OFF: &[&str] = &[");
    for color in COLORS {
        let lerped = lerp_hex(color, "#ffffff", 0.5);
        print!("\"{lerped}\",");
    }
    print!("];");
}
