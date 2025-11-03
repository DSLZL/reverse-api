use crate::grok::error::Result;
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::f64::consts::PI;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Signature;

impl Signature {
    fn h(x: f64, param: f64, c: f64, e: bool) -> f64 {
        let f = ((x * (c - param)) / 255.0) + param;
        if e {
            f.floor()
        } else {
            let rounded = (f * 100.0).round() / 100.0;
            if rounded == 0.0 || rounded == -0.0 {
                0.0
            } else {
                rounded
            }
        }
    }

    fn cubic_bezier_eased(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let bezier = |u: f64| -> (f64, f64) {
            let omu = 1.0 - u;
            let b1 = 3.0 * omu * omu * u;
            let b2 = 3.0 * omu * u * u;
            let b3 = u * u * u;
            let x = b1 * x1 + b2 * x2 + b3;
            let y = b1 * y1 + b2 * y2 + b3;
            (x, y)
        };

        let mut lo = 0.0;
        let mut hi = 1.0;
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if bezier(mid).0 < t {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let u = 0.5 * (lo + hi);
        bezier(u).1
    }

    fn xa(svg: &str) -> Vec<Vec<i32>> {
        let substr = &svg[9..];
        let parts: Vec<&str> = substr.split('C').collect();
        let mut out = Vec::new();

        for part in parts {
            let re = Regex::new(r"[^\d]+").unwrap();
            let cleaned = re.replace_all(part, " ");
            let cleaned = cleaned.trim();

            let nums: Vec<i32> = if cleaned.is_empty() {
                vec![0]
            } else {
                cleaned
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect()
            };
            out.push(nums);
        }
        out
    }

    fn tohex(num: f64) -> String {
        let rounded = (num * 100.0).round() / 100.0;
        if rounded == 0.0 || rounded == -0.0 {
            return "0".to_string();
        }

        let sign = if rounded < 0.0 { "-" } else { "" };
        let absval = rounded.abs();
        let intpart = absval.floor() as i64;
        let frac = absval - intpart as f64;

        if frac == 0.0 {
            return format!("{}{:x}", sign, intpart);
        }

        let mut frac_digits = Vec::new();
        let mut f = frac;
        for _ in 0..20 {
            f *= 16.0;
            let digit = (f + 1e-12).floor() as i64;
            frac_digits.push(format!("{:x}", digit));
            f -= digit as f64;
            if f.abs() < 1e-12 {
                break;
            }
        }

        let frac_str: String = frac_digits.into_iter().collect();
        let frac_str = frac_str.trim_end_matches('0');

        if frac_str.is_empty() {
            format!("{}{:x}", sign, intpart)
        } else {
            format!("{}{:x}.{}", sign, intpart, frac_str)
        }
    }

    fn simulate_style(values: &[i32], c: i32) -> (String, String) {
        let duration = 4096;
        let current_time = ((c / 10) * 10) as f64;
        let t = current_time / duration as f64;

        let cp: Vec<f64> = values[7..]
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let param = if i % 2 == 0 { 0.0 } else { -1.0 };
                Self::h(v as f64, param, 1.0, false)
            })
            .collect();

        let eased_y = Self::cubic_bezier_eased(t, cp[0], cp[1], cp[2], cp[3]);

        let start: Vec<f64> = values[0..3].iter().map(|&v| v as f64).collect();
        let end: Vec<f64> = values[3..6].iter().map(|&v| v as f64).collect();

        let r = (start[0] + (end[0] - start[0]) * eased_y).round() as i32;
        let g = (start[1] + (end[1] - start[1]) * eased_y).round() as i32;
        let b = (start[2] + (end[2] - start[2]) * eased_y).round() as i32;
        let color = format!("rgb({}, {}, {})", r, g, b);

        let end_angle = Self::h(values[6] as f64, 60.0, 360.0, true);
        let angle = end_angle * eased_y;
        let rad = angle * PI / 180.0;

        let is_effectively_zero = |val: f64| val.abs() < 1e-7;
        let is_effectively_integer = |val: f64| (val - val.round()).abs() < 1e-7;

        let cosv = rad.cos();
        let sinv = rad.sin();

        let a: String;
        let d: String;
        if is_effectively_zero(cosv) {
            a = "0".to_string();
            d = "0".to_string();
        } else if is_effectively_integer(cosv) {
            a = cosv.round().to_string();
            d = cosv.round().to_string();
        } else {
            a = format!("{:.6}", cosv);
            d = format!("{:.6}", cosv);
        }

        let bval: String;
        let cval: String;
        if is_effectively_zero(sinv) {
            bval = "0".to_string();
            cval = "0".to_string();
        } else if is_effectively_integer(sinv) {
            bval = sinv.round().to_string();
            cval = (-sinv).round().to_string();
        } else {
            bval = format!("{:.7}", sinv);
            cval = format!("{:.7}", -sinv);
        }

        let transform = format!("matrix({}, {}, {}, {}, 0, 0)", a, bval, cval, d);
        (color, transform)
    }

    fn xs(x_bytes: &[u8], svg: &str, x_values: &[usize]) -> String {
        let idx = (x_bytes[x_values[0]] % 16) as usize;
        let c = ((x_bytes[x_values[1]] % 16) as i32 * (x_bytes[x_values[2]] % 16) as i32)
            * (x_bytes[x_values[3]] % 16) as i32;

        let o = Self::xa(svg);
        let vals = &o[idx];
        let (color, transform) = Self::simulate_style(vals, c);

        let concat = format!("{}{}", color, transform);
        let re = Regex::new(r"[\d\.\-]+").unwrap();
        let matches: Vec<&str> = re.find_iter(&concat).map(|m| m.as_str()).collect();

        let converted: Vec<String> = matches
            .iter()
            .map(|m| {
                let num: f64 = m.parse().unwrap_or(0.0);
                Self::tohex(num)
            })
            .collect();

        let joined = converted.join("");
        joined.replace(".", "").replace("-", "")
    }

    pub fn generate_sign(
        path: &str,
        method: &str,
        verification: &str,
        svg: &str,
        x_values: &[usize],
        time_n: Option<u32>,
        random_float: Option<f64>,
    ) -> Result<String> {
        let n = time_n.unwrap_or_else(|| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            now - 1682924400
        });

        let t = n.to_le_bytes();
        let r = general_purpose::STANDARD.decode(verification)?;
        let o = Self::xs(&r, svg, x_values);

        let msg = format!("{}!{}!{}", method, path, n) + "obfiowerehiring" + &o;
        let mut hasher = Sha256::new();
        hasher.update(msg.as_bytes());
        let digest = &hasher.finalize()[..16];

        let prefix_byte = (random_float.unwrap_or_else(rand::random::<f64>) * 256.0).floor() as u8;

        let mut assembled = Vec::new();
        assembled.push(prefix_byte);
        assembled.extend_from_slice(&r);
        assembled.extend_from_slice(&t);
        assembled.extend_from_slice(digest);
        assembled.push(3);

        // XOR with first byte
        if !assembled.is_empty() {
            let first = assembled[0];
            for i in 1..assembled.len() {
                assembled[i] ^= first;
            }
        }

        Ok(general_purpose::STANDARD
            .encode(&assembled)
            .trim_end_matches('=')
            .to_string())
    }
}
