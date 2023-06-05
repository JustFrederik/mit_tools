use image::imageops::FilterType;
use image::GenericImageView;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::PyResult;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;

#[pymodule]
fn mit_tools(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(scale_down, m)?)?;
    m.add_function(wrap_pyfunction!(sha256, m)?)?;
    m.add_function(wrap_pyfunction!(sha256_scale, m)?)?;
    Ok(())
}

#[pyfunction]
pub fn scale_down(
    image_path: String,
    output_path: String,
    filter: String,
    scale: f32,
) -> PyResult<()> {
    let filter = get_filter(filter.as_str()).map_err(PyException::new_err)?;
    scale_down_rust(&image_path, &output_path, filter, scale).map_err(PyException::new_err)?;
    Ok(())
}

#[pyfunction]
pub fn sha256(image_path: String) -> PyResult<Vec<u8>> {
    sha256_rust(&image_path)
        .map_err(PyException::new_err)
        .map_err(PyException::new_err)
}

#[pyfunction]
pub fn sha256_scale(
    image_path: String,
    output_path: String,
    filter: String,
    scale: f32,
) -> PyResult<Vec<u8>> {
    let sha = sha256_rust(&image_path)
        .map_err(PyException::new_err)
        .map_err(PyException::new_err)?;
    let filter = get_filter(filter.as_str()).map_err(PyException::new_err)?;
    scale_down_rust(&image_path, &output_path, filter, scale).map_err(PyException::new_err)?;
    Ok(sha)
}

fn get_filter(filter: &str) -> Result<FilterType, String> {
    match filter {
        "lanczos3" => Ok(FilterType::Lanczos3),
        "catmullrom" => Ok(FilterType::CatmullRom),
        "gaussian" => Ok(FilterType::Gaussian),
        "nearest" => Ok(FilterType::Nearest),
        "triangle" => Ok(FilterType::Triangle),
        _ => Err("Invalid filter type".to_string()),
    }
}

fn scale_down_rust(
    image_path: &str,
    output_path: &str,
    filter: FilterType,
    scale: f32,
) -> Result<(), String> {
    let image = image::open(image_path).expect("Failed to open image");

    let (original_width, original_height) = image.dimensions();

    let new_width = (original_width as f32 * scale).round() as u32;
    let new_height = (original_height as f32 * scale).round() as u32;

    let resized_image = image.resize_exact(new_width, new_height, filter);

    resized_image
        .save(output_path)
        .map_err(|e| format!("Failed to save image {}", e))?;
    Ok(())
}

fn sha256_rust(image_path: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(image_path).map_err(|e| format!("Failed to open image {}", e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read image {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let hash = hasher.finalize().iter().copied().collect::<Vec<_>>();

    Ok(hash)
}
