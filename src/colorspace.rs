use image::{DynamicImage, GenericImageView, GenericImage};
use yuvutils_rs::*;

/// Takes an RGB DynamicImage and convert to YCrCb
/// 
/// Return value: `(y_plane, cb_plane, cr_plane)`
#[allow(non_snake_case)]
pub fn convert_to_YCbCr(image: &DynamicImage) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (width, height) = image.dimensions();
    let mut y_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
    let mut cb_plane: Vec<u8> = vec![0_u8; (width * height) as usize];
    let mut cr_plane: Vec<u8> = vec![0_u8; (width * height) as usize];

    let rgb = image.as_bytes();
    let (width, height) = image.dimensions();
    let (rgb_stride, y_stride, cb_stride, cr_stride) = get_strides(width, false);

    rgb_to_yuv444(y_plane.as_mut_slice(), y_stride,
                  cb_plane.as_mut_slice(), cb_stride,
                  cr_plane.as_mut_slice(), cr_stride,
                  &rgb, rgb_stride,
                  width, height, 
                  YuvRange::Full, YuvStandardMatrix::Bt709);

    (y_plane, cb_plane, cr_plane)
}

/// Convert YCrCb to RGB DynamicImage
#[allow(non_snake_case)]
pub fn convert_to_RGB(
    width: u32,
    height: u32,
    y_plane: &[u8],
    cb_plane: &[u8],
    cr_plane: &[u8]
) -> DynamicImage {
    let (rgb_stride, y_stride, cb_stride, cr_stride) = get_strides(width, false);
    let mut rgb = vec![0_u8; (width * height * 3) as usize];

    yuv444_to_rgb(&y_plane, y_stride, 
                  &cb_plane, cb_stride,
                  &cr_plane, cr_stride,
                  rgb.as_mut_slice(), rgb_stride,
                  width, height, 
                  YuvRange::Full, YuvStandardMatrix::Bt709);

    let mut img = DynamicImage::new_rgb8(width, height);

    for x in 0..height {
        for y in 0..width {
            let r = rgb[(x * width * 3 + y * 3) as usize];
            let g = rgb[(x * width * 3 + y * 3 + 1) as usize];
            let b = rgb[(x * width * 3 + y * 3 + 2) as usize];
            let a = 255;
            img.put_pixel(y, x, image::Rgba([r, g, b, a]));
        }
    }

    img
}

/// Calculates and returns the strides needed for colorspace conversion
/// 
/// Return value: `(rgb_stride, y_stride, cb_stride, cr_stride)`
/// 
/// set downsample to true when using 422 conversion, false when using 444
fn get_strides(width: u32, downsample: bool) -> (u32, u32, u32, u32) {
	let rgb_stride = width * 3;  // 3 bytes per pixel for RGB
    let y_stride = width;        // 1 byte per pixel for Y
    let cb_stride = if downsample {(width + 1) / 2} else {width}; // subsampled horizontally
    let cr_stride = if downsample {(width + 1) / 2} else {width}; // subsampled horizontally

    (rgb_stride, y_stride, cb_stride, cr_stride)
}