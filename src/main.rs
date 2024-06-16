use img_watermarking::*;

fn main() {
	let k = 21307;
	let ss = 100.0;
	let wmkd_img = embed_watermark("test_images/pepper.tiff", "test_images/wm_img.png", k, ss);
	wmkd_img.save("watermarked_image_pepper.tiff").unwrap();

	let (_, wmk_img) = extract_watermark("watermarked_image_pepper.tiff", k, ss);
	wmk_img.save("reconstructed_wm_from_pepper.tiff").unwrap();
}


