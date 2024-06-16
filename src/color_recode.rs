use image::{GenericImage, DynamicImage};
use bitvec::prelude::*;

/// Recodes the original picture color info into 3-bit color representation scheme
pub fn recode_to_3bits(image: &DynamicImage) -> BitVec {
	let mut ret = BitVec::new();
    let rgb_bytes = image.as_bytes();

    for byte in rgb_bytes {
    	if *byte > 127 {
    		ret.push(true);
    	} else {
    		ret.push(false);
    	}
    }

    ret
}

/// Recode the bits in the 3-bit color representation scheme back to RGB DynamicImage
pub fn recode_to_rgb(bits: &BitVec, width: u32, height: u32) -> DynamicImage {
    let mut image = DynamicImage::new_rgb8(width, height);
    let mut x = 0;
    let mut y = 0;

    let mut r = 0;
    let mut g = 0;
    for (i, bit) in bits.iter().enumerate() {
        if *bit.as_ref() {
            match i % 3 {
                0 => { r = 255; }
                1 => { g = 255; }
                2 => {
                    let b = 255;
                    image.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                    y += if x + 1 == width { 1 } else { 0 };
                    x = (x + 1) % width;
                }
                _ => panic!("Impossible value of i % 3")
            };
        } else {
            match i % 3 {
                0 => { r = 0; }
                1 => { g = 0; }
                2 => {
                    let b = 0;
                    image.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                    y += if x + 1 == width { 1 } else { 0 };
                    x = (x + 1) % width;
                }
                _ => panic!("Impossible value of i % 3")
            };
        }
    }

    image
}