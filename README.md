# Color_watermark

## Introduction 

This is a rust implementation of the 2018 paper *A Robust Watermarking Scheme to JPEG Compression for Embedding a Color Watermark into Digital Images* by David-Octavio Muñoz-Ramirez, Volodymyr Ponomaryov, Rogelio Reyes-Reyes, Volodymyr Kyrychenko, Oleksandr Pechenin and Alexander Totsky. Many thanks to the authors for such a wonderful idea! 

## Quickstart

Prepare a 512 * 512 host image and 128 * 128 watermark image in RGB encoding. 

```rust
let key = 123456;
let step_size = 50.0;
let watermarked_img = embed_watermark("path/to/host_image", "path/to/watermark", key, step_size);
watermarked_img.save("path/to/watermarked_img");

let extracted_wm = extract_watermark("path/to/watermarked_img", key, step_size);
extracted_wm.save("path/to/extracted_wm");
```
