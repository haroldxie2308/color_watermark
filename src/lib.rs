pub mod dct;
pub mod qim;
pub mod color_recode;
pub mod colorspace;

use bitvec::prelude::*;
use image::{GenericImage, DynamicImage};

use dct::BLOCK_WIDTH;


/// Load the image indicated by the path
fn load_image(path: &str) -> DynamicImage {
    image::open(path).expect("Failed to load image")
}


/// Reconstruct a width * height RGB DynamicImage from rgb blocks (Vec of 8 * 8 f32)
fn reconstruct_image_from_rgb(
    blocks_r: &Vec<Vec<f32>>,
    blocks_g: &Vec<Vec<f32>>,
    blocks_b: &Vec<Vec<f32>>,
    width: u32,
    height: u32
) -> DynamicImage {
    let mut image = DynamicImage::new_rgb8(width, height);

    for (block_idx, ((block_r, block_g), block_b)) in blocks_r.iter().zip(blocks_g.iter()).zip(blocks_b.iter()).enumerate() {
        let x = (block_idx % (width as usize / BLOCK_WIDTH)) * BLOCK_WIDTH;
        let y = (block_idx / (width as usize/ BLOCK_WIDTH)) * BLOCK_WIDTH;

        for j in 0..BLOCK_WIDTH {
            for i in 0..BLOCK_WIDTH {
                let r = block_r[j * BLOCK_WIDTH + i] as u8;
                let g = block_g[j * BLOCK_WIDTH + i] as u8;
                let b = block_b[j * BLOCK_WIDTH + i] as u8;
                let a = 255_u8;
                image.put_pixel(x as u32 + i as u32, y as u32 + j as u32, image::Rgba([r, g, b, a]));
            }
        }
    }

    image
}

pub fn reconstruct_from_bitvec(bits: &BitVec, width: u32, height: u32) -> DynamicImage {
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

pub fn print_block(block: &Vec<f32>) {
    println!("*********************************");
    for line in block.chunks(8) {
        println!("{:?} ", line);
    }
    println!("*********************************");
}


#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    #[test]
    fn test_2d_dct() {
        let image_path = "test_images/pepper.tiff";
        let image = load_image(image_path);
        let (width, height) = image.dimensions();

        let (mut blocks_r, mut blocks_g, mut blocks_b) = dct::split_image_into_blocks(&image);

        dct::apply_2d_dct(&mut blocks_r);
        dct::apply_2d_dct(&mut blocks_g);
        dct::apply_2d_dct(&mut blocks_b);

        let transformed_image = reconstruct_image_from_rgb(&blocks_r, &blocks_g, &blocks_b, width, height);
        transformed_image.save("pepper_2ddct.tiff").expect("Failed to save image");

        dct::apply_2d_idct(&mut blocks_r);
        dct::apply_2d_idct(&mut blocks_g);
        dct::apply_2d_idct(&mut blocks_b);

        let unchanged_image = reconstruct_image_from_rgb(&blocks_r, &blocks_g, &blocks_b, width, height);

        unchanged_image.save("pepper_unchanged_dct.tiff").expect("Failed to save unchanged_image");
    }

    #[test]
    fn test_rgb_toforth_ycrcb() {
        let image_path = "test_images/pepper.tiff";
        let image = load_image(image_path);
        let (width, height) = image.dimensions();

        let mut y_plane: Vec<u8> = Vec::with_capacity((width * height) as usize);
        let mut cr_plane: Vec<u8> = Vec::with_capacity((width * height) as usize);
        let mut cb_plane: Vec<u8> = Vec::with_capacity((width * height) as usize);


        colorspace::convert_to_YCrCb(&image, y_plane.as_mut_slice(), cr_plane.as_mut_slice(), cb_plane.as_mut_slice());

        let rgb_img = colorspace::convert_to_RGB(width, height, y_plane.as_slice(), cr_plane.as_slice(), cb_plane.as_slice());

        rgb_img.save("pepper_unchanged_color.tiff").unwrap();
    }

    #[test]
    fn test_complete_workflow() {
        let image_path = "test_images/pepper.tiff";
        let image = load_image(image_path);
        let (width, height) = image.dimensions();

        let mut y_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
        let mut cr_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
        let mut cb_plane: Vec<u8> = vec![0_u8; (width * height) as usize];

        // Converting to YCrCb colorspace
        colorspace::convert_to_YCrCb(
            &image,
            y_plane.as_mut_slice(),
            cr_plane.as_mut_slice(),
            cb_plane.as_mut_slice()
        );

        // DCT only on Y-plane
        let mut blocks = dct::split_into_blocks(&mut y_plane, width as usize, height as usize);
        assert!(blocks.len() == 64 * 64 && blocks[0].len() == 8 * 8);

        dct::apply_2d_dct(&mut blocks);


        // Generate color-mapped array from the input watermark
        let wm_path = "test_images/small_wm.png";
        let wm_image = load_image(&wm_path);
        // let (wm_width, wm_height) = wm_image.dimensions();

        let wm_bits = color_recode::color_recode_to_3bits(&wm_image);
        // let recoded_wm = reconstruct_from_bitvec(&wm_bits, wm_width, wm_height);
        // recoded_vm.save("recoded_wm.png").unwrap();
        assert!(wm_bits.len() == 128 * 128 * 3);


        // QIM-DM
        let key = 0123456_u64;
        // Generate 2 dither arrays,
        // step_size of 1.0 by using round() as base quantizer
        let step_size = 1.0;
        let dithers = qim::generate_dither_signal(12, step_size, key);

        // for (i, bits) in wm_bits.chunks(12).enumerate() {
        //     qim::embed_watermark(&mut blocks[i], &bits.to_bitvec(), &dithers);
        // }

        // let mut extracted_wm: BitVec<usize, Lsb0> = BitVec::new();
        // for block in blocks.iter() {
        //     let tmp = qim::extract_watermark(&block, &dithers, step_size);
        //     for bit in tmp {
        //         extracted_wm.push(bit);
        //     }
        // }

        // Apply idct to y_plane
        dct::apply_2d_idct(&mut blocks);

        // Reconstruct the image
        let y_plane: Vec<u8> = blocks.into_iter()
                                        .flatten()
                                        .map(|elem| elem as u8)
                                        .collect();
        let wmd_image = colorspace::convert_to_RGB(
                                                width,
                                                height,
                                                &y_plane,
                                                &cr_plane,
                                                &cb_plane
                                            );

        wmd_image.save("watermarked_img.tiff").unwrap();

    }
    
}
