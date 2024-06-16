use image::{DynamicImage, GenericImageView};

pub mod dct;
pub mod qim;
pub mod color_recode;
pub mod colorspace;

use bitvec::prelude::*;

/// Uses DCT together with QIM-DM to embed the colored watermark image into the host image
/// 
/// Higher `step_size` generally yields better extraction result, but might reduce the imperceptability of the watermark
/// 
/// Panics if the host image is not 512 * 512 or the watermark image is not 128 * 128
pub fn embed_watermark(
    host_image: &str,
    watermark_image: &str,
    key: u64,
    step_size: f32
) -> DynamicImage {
    let host = image::open(host_image).expect("Failed to open host image");
    let (h_width, h_height) = host.dimensions();
    assert!(h_width == 512 && h_height == 512);

    // Set up YCrCb plane buffer
    let mut y_plane: Vec<u8> = vec![0_u8; (h_width * h_height) as usize];
    let mut cr_plane: Vec<u8> = vec![0_u8; (h_width * h_height) as usize];
    let mut cb_plane: Vec<u8> = vec![0_u8; (h_width * h_height) as usize];

    // Convert the image to YCrCb colorspace
    colorspace::convert_to_YCrCb(
        &host,
        y_plane.as_mut_slice(),
        cr_plane.as_mut_slice(),
        cb_plane.as_mut_slice()
    );

    // Split Y plane into 8 * 8 blocks for DCT operation
    let mut y_blocks = dct::split_into_blocks(
                                        &mut y_plane,
                                        h_width as usize,
                                        h_height as usize
                                    );

    // DCT on Y blocks
    dct::apply_2d_dct(&mut y_blocks);

    let wm = image::open(watermark_image).expect("Failed to open watermark image");
    let (wm_width, wm_height) = wm.dimensions();
    assert!(wm_width == 128 && wm_height == 128);

    // Recoding the watermark
    let wm_bits = color_recode::recode_to_3bits(&wm);

    // QIM-DM to embed the watermark with the preset key and step_size
    let dithers = qim::generate_dither_signal(12, step_size, key);
    for (i, bits) in wm_bits.chunks(12).enumerate() {
        qim::embed_watermark(&mut y_blocks[i], &bits.to_bitvec(), &dithers, step_size);
    }

    // IDCT on watermarked Y blocks
    dct::apply_2d_idct(&mut y_blocks);

    // Convert Y blocks back to Y plane
    let watermarked_y_plane = dct::merge_into_plane(
                                                &y_blocks,
                                                h_width as usize,
                                                h_height as usize
                                            );

    // Convert back to RGB colorspace and return the RGB DynamicImage
    colorspace::convert_to_RGB(
                                h_width,
                                h_height,
                                &watermarked_y_plane,
                                &cr_plane,
                                &cb_plane
                            )
}

/// Extract the colored watermark embedded using DCT + QIM-DM watermarking scheme
/// 
/// Returns the original bit stream and the reconstructed RGB DynamicImage
pub fn extract_watermark(watermarked_image: &str, key: u64, step_size: f32) -> (BitVec, DynamicImage) {
    let wmkd_image = image::open(watermarked_image).unwrap();
    let (width, height) = wmkd_image.dimensions();

    // Convert the watermarked image to YCrCb colorspace and DCT on Y blocks
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

    // Extract the watermark from each block
    let dithers = qim::generate_dither_signal(12, step_size, key);
    let mut extracted_wm: BitVec<usize, Lsb0> = BitVec::new();
    for block in wmkd_y_blocks.iter() {
        let tmp = qim::extract_watermark(&block, &dithers, step_size);
        for bit in tmp {
            extracted_wm.push(bit);
        }
    }

    // Reconstruct the image from bits and save the recovered watermark
    let reconstructed_wm_image = color_recode::recode_to_rgb(&extracted_wm, 128, 128);
    (extracted_wm, reconstructed_wm_image)
}


#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    use bitvec::prelude::*;

    #[test]
    fn test_2d_dct() {
        let image_path = "test_images/pepper.tiff";
        let image = image::open(image_path).unwrap();
        let (width, height) = image.dimensions();

        let (mut blocks_r, mut blocks_g, mut blocks_b) = dct::split_image_into_blocks(&image);

        dct::apply_2d_dct(&mut blocks_r);
        dct::apply_2d_dct(&mut blocks_g);
        dct::apply_2d_dct(&mut blocks_b);

        let transformed_image = dct::reconstruct_image_from_rgb(&blocks_r, &blocks_g, &blocks_b, width, height);
        transformed_image.save("test_results/pepper_2ddct.tiff").expect("Failed to save image");

        dct::apply_2d_idct(&mut blocks_r);
        dct::apply_2d_idct(&mut blocks_g);
        dct::apply_2d_idct(&mut blocks_b);

        let unchanged_image = dct::reconstruct_image_from_rgb(&blocks_r, &blocks_g, &blocks_b, width, height);

        unchanged_image.save("test_results/pepper_unchanged_dct.tiff").expect("Failed to save unchanged_image");
    }

    #[test]
    fn test_rgb_toforth_ycrcb() {
        let image_path = "test_images/pepper.tiff";
        let image = image::open(image_path).unwrap();
        let (width, height) = image.dimensions();

        let mut y_plane: Vec<u8> = Vec::with_capacity((width * height) as usize);
        let mut cr_plane: Vec<u8> = Vec::with_capacity((width * height) as usize);
        let mut cb_plane: Vec<u8> = Vec::with_capacity((width * height) as usize);


        colorspace::convert_to_YCrCb(&image, y_plane.as_mut_slice(), cr_plane.as_mut_slice(), cb_plane.as_mut_slice());

        let rgb_img = colorspace::convert_to_RGB(width, height, y_plane.as_slice(), cr_plane.as_slice(), cb_plane.as_slice());

        rgb_img.save("test_results/pepper_unchanged_color.tiff").unwrap();
    }

    #[test]
    fn test_complete_workflow() {
        let key = 0123456_u64;
        let step_size = 100.0;

        // *********** Embedding the watermark **********
        let image_path = "test_images/pepper.tiff";
        let image = image::open(image_path).unwrap();
        let (width, height) = image.dimensions();

        // Set up YCrCb plane buffer
        let mut y_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
        let mut cr_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
        let mut cb_plane: Vec<u8> = vec![0_u8; (width * height) as usize];

        // Convert the image to YCrCb colorspace
        colorspace::convert_to_YCrCb(
            &image,
            y_plane.as_mut_slice(),
            cr_plane.as_mut_slice(),
            cb_plane.as_mut_slice()
        );

        // Split Y plane into 8 * 8 blocks for DCT operation
        let mut y_blocks = dct::split_into_blocks(
                                            &mut y_plane,
                                            width as usize,
                                            height as usize
                                        );

        // DCT on Y blocks
        dct::apply_2d_dct(&mut y_blocks);

        // Load the watermark image
        let wm_path = "test_images/wm_img.png";
        let wm_image = image::open(wm_path).unwrap();
        // Recoding the watermark
        let wm_bits = color_recode::recode_to_3bits(&wm_image);

        // QIM-DM to embed the watermark with the preset key and step_size
        let dithers = qim::generate_dither_signal(12, step_size, key);
        for (i, bits) in wm_bits.chunks(12).enumerate() {
            qim::embed_watermark(&mut y_blocks[i], &bits.to_bitvec(), &dithers, step_size);
        }

        // In between embedding result test
        let mut extracted_wm: BitVec<usize, Lsb0> = BitVec::new();
        for block in y_blocks.iter() {
            let tmp = qim::extract_watermark(&block, &dithers, step_size);
            for bit in tmp {
                extracted_wm.push(bit);
            }
        }
        color_recode::recode_to_rgb(&extracted_wm, 128, 128).save("test_results/in_between.tiff").unwrap();


        // IDCT on watermarked Y blocks
        dct::apply_2d_idct(&mut y_blocks);

        // Convert Y blocks back to Y plane
        let watermarked_y_plane = dct::merge_into_plane(
                                                    &y_blocks,
                                                    width as usize,
                                                    height as usize
                                                );

        // Convert back to RGB colorspace
        let wmd_image = colorspace::convert_to_RGB(
                                                width,
                                                height,
                                                &watermarked_y_plane,
                                                &cr_plane,
                                                &cb_plane
                                            );

        // Save the watermarked image
        wmd_image.save("test_results/watermarked_img.tiff").unwrap();


        // ************ Extracting the watermark ***************
        let wmkd_image = image::open("test_results/watermarked_img.tiff").unwrap();
        let (width, height) = wmkd_image.dimensions();

        // Convert the watermarked image to YCrCb colorspace and DCT on Y blocks
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

        // Extract the watermark from each block
        let mut extracted_wm: BitVec<usize, Lsb0> = BitVec::new();
        for block in wmkd_y_blocks.iter() {
            let tmp = qim::extract_watermark(&block, &dithers, step_size);
            for bit in tmp {
                extracted_wm.push(bit);
            }
        }

        // Reconstruct the image from bits and save the recovered watermark
        let reconstructed_wm = color_recode::recode_to_rgb(&extracted_wm, 128, 128);
        reconstructed_wm.save("test_results/reconstructed_wm.tiff").unwrap();
    }   
}
