use img_watermarking::{color_recode, colorspace, dct, qim, reconstruct_from_bitvec};
use image::{GenericImage, DynamicImage, GenericImageView};
use bitvec::prelude::*;

fn main() {
	let key = 0123456_u64;
	let step_size = 100.0;

	// Embed the watermark
	let image_path = "test_images/pepper.tiff";
	let image = image::open(image_path).unwrap();
	let (width, height) = image.dimensions();

	println!("Original RGB value: ");
	let imgbytes = image.as_bytes();
	for i in 0..24 {
		print!("{:?} ", imgbytes[i]);
	}
	println!();

	let mut y_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
	let mut cr_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
	let mut cb_plane: Vec<u8> = vec![0_u8; (width * height) as usize];

	colorspace::convert_to_YCrCb(
	    &image,
	    y_plane.as_mut_slice(),
	    cr_plane.as_mut_slice(),
	    cb_plane.as_mut_slice()
	);

	let mut y_blocks = dct::split_into_blocks(
										&mut y_plane,
										width as usize,
										height as usize
									);

	dct::apply_2d_dct(&mut y_blocks);

	// println!("Unwatermarked y_block: ");
	// print_block(&y_blocks[1]);

	let wm_path = "test_images/small_wm.png";
	let wm_image = image::open(wm_path).unwrap();
	let wm_bits = color_recode::color_recode_to_3bits(&wm_image);


	// QIM-DM
	let dithers = qim::generate_dither_signal(12, step_size, key);

	for (i, bits) in wm_bits.chunks(12).enumerate() {
	    qim::embed_watermark(&mut y_blocks[i], &bits.to_bitvec(), &dithers, step_size);
	}

	// println!("Watermarked y_block: ");
	// print_block(&y_blocks[1]);

	let mut extracted_wm: BitVec<usize, Lsb0> = BitVec::new();
    for block in y_blocks.iter() {
        let tmp = qim::extract_watermark(&block, &dithers, step_size);
        for bit in tmp {
            extracted_wm.push(bit);
        }
    }

    reconstruct_from_bitvec(&extracted_wm, 128, 128).save("inbetween.tiff").unwrap();

	dct::apply_2d_idct(&mut y_blocks);

	let watermarked_y_plane: Vec<u8> = dct::merge_into_plane(
										&y_blocks,
										width as usize,
										height as usize
									);

	let wmd_image = colorspace::convert_to_RGB(
	                                        width,
	                                        height,
	                                        &watermarked_y_plane,
	                                        &cr_plane,
	                                        &cb_plane
	                                    );

	println!("Watermarked RGB value: ");
	let imgbytes = wmd_image.as_bytes();
	for i in 0..24 {
		print!("{:?} ", imgbytes[i]);
	}
	println!();

	wmd_image.save("watermarked_img.tiff").unwrap();


	// Extract the watermark, with pre-shared `key` and `step_size`
	let wmkd_image = image::open("watermarked_img.tiff").unwrap();

	let (width, height) = wmkd_image.dimensions();

	println!("Read RGB value: ");
	let imgbytes = wmkd_image.as_bytes();
	for i in 0..24 {
		print!("{:?} ", imgbytes[i]);
	}
	println!();

	let mut wmkd_y_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
	let mut cr_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
	let mut cb_plane: Vec<u8> = vec![0_u8; (width * height) as usize];

	colorspace::convert_to_YCrCb(
	    &wmkd_image,
	    wmkd_y_plane.as_mut_slice(),
	    cr_plane.as_mut_slice(),
	    cb_plane.as_mut_slice()
	);

	let mut wmkd_y_blocks = dct::split_into_blocks(
										&mut wmkd_y_plane,
										width as usize,
										height as usize
									);

	dct::apply_2d_dct(&mut wmkd_y_blocks);

	// println!("Read wmkd block from image:");
	// print_block(&wmkd_y_blocks[1]);

	let mut extracted_wm: BitVec<usize, Lsb0> = BitVec::new();
	for block in wmkd_y_blocks.iter() {
	    let tmp = qim::extract_watermark(&block, &dithers, step_size);
	    for bit in tmp {
	        extracted_wm.push(bit);
	    }
	}

	let reconstructed_wm = reconstruct_from_bitvec(&extracted_wm, 128, 128);
	reconstructed_wm.save("reconstructed_wm.tiff").unwrap();
}


