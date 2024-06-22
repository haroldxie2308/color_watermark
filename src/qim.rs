use bitvec::vec::BitVec;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Generates a Vec for 2 Dither Arrays
/// 
/// length should be 12 for this specific implementation
pub fn generate_dither_signal(length: usize, step_size: f32, seed: u64) -> Vec<(f32, f32)> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let half = step_size / 2.0;
    (0..length).map(|_| {
            let tmp = rng.gen_range(-half..half);
            if tmp > 0.0 {
                (tmp, tmp - half)
            } else {
                (tmp, tmp + half)
            }
        }).collect()
}

fn in_range(i: usize) -> bool {
    // I choose to use those coefficients, just because it's easier
    (i >= 4 && i <= 7) || (i >= 11 && i <= 15) || (i >= 18 && i <= 20)
}

fn round_to_step_size(num: f32, step_size: f32) -> f32 {
    if num % step_size > (step_size / 2.0) {
        (num as u32 / step_size as u32 + 1) as f32 * step_size
    } else {
        (num as u32 / step_size as u32) as f32 * step_size
    }
}

pub fn embed_wm(
    host_signal: &mut Vec<f32>,
    watermark: &BitVec,
    dither_signal: &Vec<(f32, f32)>,
    step_size: f32
) {
    let mut j = 0;
    for (i, h) in host_signal.iter_mut().enumerate() {
        if in_range(i) {
            let d = if watermark[j] { dither_signal[j].1 } else { dither_signal[j].0 };
            *h = round_to_step_size(*h + d, step_size) - d;
            j += 1;
        }
    }
    assert!(j == 12);
}

pub fn extract_wm(
    watermarked_signal: &Vec<f32>,
    dither_signal: &Vec<(f32, f32)>,
    step_size: f32
) -> BitVec {
    let acceptable_range = step_size / 10.0;

    let mut ret = BitVec::new();
    let mut j = 0;
    for (i, wmkd_bit) in watermarked_signal.iter().enumerate() {
        if in_range(i) {
            let tmp = wmkd_bit + dither_signal[j].0;
            if (round_to_step_size(tmp, step_size) - tmp).abs() < acceptable_range {
                ret.push(false);
            } else {
                ret.push(true);
            }
            j += 1;
        }
    }
    assert!(j == 12);
    ret
}