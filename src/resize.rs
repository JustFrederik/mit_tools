use image::imageops::FilterType;
use image::GenericImageView;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;

pub fn get_filter(filter: &str) -> Result<FilterType, String> {
    match filter {
        "lanczos3" => Ok(FilterType::Lanczos3),
        "catmullrom" => Ok(FilterType::CatmullRom),
        "gaussian" => Ok(FilterType::Gaussian),
        "nearest" => Ok(FilterType::Nearest),
        "triangle" => Ok(FilterType::Triangle),
        _ => Err("Invalid filter type".to_string()),
    }
}

pub fn scale_down_rust(
    image_path: &str,
    output_path: &str,
    filter: FilterType,
    scale: f32,
) -> Result<(), String> {
    let image = image::open(image_path).map_err(|v| format!("Failed to open image: {}", v))?;

    let (original_width, original_height) = image.dimensions();

    let new_width = (original_width as f32 * scale).round() as u32;
    let new_height = (original_height as f32 * scale).round() as u32;

    let resized_image = image.resize_exact(new_width, new_height, filter);

    resized_image
        .save(output_path)
        .map_err(|e| format!("Failed to save image {}", e))?;
    Ok(())
}

pub fn sha256_rust(image_path: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(image_path).map_err(|e| format!("Failed to open image {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read image {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let hash = hasher.finalize().iter().copied().collect::<Vec<_>>();

    Ok(hash)
}
