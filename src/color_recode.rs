use image::{GenericImage, GenericImageView, DynamicImage};
use bitvec::prelude::*;


/// Recodes the original picture color info into 3-bit color representation scheme
pub fn color_recode_to_3bits(image: &DynamicImage) -> BitVec {
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