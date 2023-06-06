use crate::file_finder::get_images;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::PyResult;
use std::path::PathBuf;

use crate::resize::{get_filter, scale_down_rust, sha256_rust};

mod file_finder;
mod resize;

#[pymodule]
fn mit_tools(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(scale_down, m)?)?;
    m.add_function(wrap_pyfunction!(sha256, m)?)?;
    m.add_function(wrap_pyfunction!(sha256_scale, m)?)?;
    m.add_function(wrap_pyfunction!(get_imgs, m)?)?;
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
    sha256_rust(&image_path).map_err(PyException::new_err)
}

#[pyfunction]
pub fn sha256_scale(
    image_path: String,
    output_path: String,
    filter: String,
    scale: f32,
) -> PyResult<Vec<u8>> {
    let sha = sha256_rust(&image_path).map_err(PyException::new_err)?;
    let filter = get_filter(filter.as_str()).map_err(PyException::new_err)?;
    scale_down_rust(&image_path, &output_path, filter, scale).map_err(PyException::new_err)?;
    Ok(sha)
}

#[pyfunction]
pub fn get_imgs(
    root: String,
    output_dir: String,
    file_types: Vec<String>,
) -> PyResult<Vec<(String, String)>> {
    let root = PathBuf::from(root);
    let output_dir = match output_dir.as_str() {
        "" => None,
        _ => Some(PathBuf::from(output_dir)),
    };
    let file_types = file_types.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    get_images(&root, output_dir, file_types)
        .map_err(PyException::new_err)
        .map(|v| {
            v.iter()
                .map(|(a, b)| {
                    (
                        a.to_str().unwrap_or("").to_string(),
                        b.to_str().unwrap_or("").to_string(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .map_err(PyException::new_err)
}
