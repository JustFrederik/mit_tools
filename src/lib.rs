use crate::file_finder::get_images;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::PyResult;
use std::path::PathBuf;

use crate::resize::{get_filter, scale_down_rust, sha256_rust};
use crate::translate::{Data, Translate};

mod file_finder;
mod resize;
mod translate;

#[pymodule]
fn mit_tools(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(scale_down, m)?)?;
    m.add_function(wrap_pyfunction!(sha256, m)?)?;
    m.add_function(wrap_pyfunction!(sha256_scale, m)?)?;
    m.add_function(wrap_pyfunction!(get_imgs, m)?)?;
    m.add_class::<Translate>()?;
    m.add_class::<Data>()?;
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
    ending: String,
) -> PyResult<Vec<(String, String)>> {
    let root = PathBuf::from(root);
    let output_dir = match output_dir.as_str() {
        "" => None,
        _ => Some(PathBuf::from(output_dir)),
    };
    let ending = match ending.as_str() {
        "" => None,
        _ => Some(ending),
    };
    let file_types = file_types.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    get_images(&root, output_dir, file_types, &ending)
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

#[pymethods]
impl Translate {
    #[pyo3(signature = (text, data))]
    fn translate_py(&self, text: String, data: &mut Data) -> PyResult<String> {
       self.translate(text, data).map_err(PyException::new_err)
    }
    #[pyo3(signature = (text, data))]
    fn translate_vec_py(&self, text: Vec<String>, data: &mut Data) -> PyResult<Vec<String>> {
        self.translate_vec(text, data).map_err(PyException::new_err)
    }
}

#[pymethods]
impl Data {
    #[new]
    fn new_py(chat_gpt_context: Option<String>) -> PyResult<Self> {
        Self::new(chat_gpt_context).map_err(PyException::new_err)
    }
    #[pyo3(signature = (target, default_translator, translators))]
    fn generate_selector_selective_py(&mut self, target: &str, default_translator: &str, translators: Vec<(&str, &str)>) -> PyResult<()> {
        self.generate_selector_selective(target, default_translator, translators).map_err(PyException::new_err)
    }

    #[pyo3(signature = (translators))]
    pub fn generate_chain_py(&mut self, translators: Vec<(&str, &str)>) -> PyResult<()>{
        self.generate_chain(translators).map_err(PyException::new_err)
    }

    #[pyo3(signature = (translators, default_target, default_translator))]
    pub fn generate_selective_chain_py(&mut self, translators: Vec<(&str, &str, &str)>, default_target: &str, default_translator: &str) -> PyResult<()> {
        self.generate_selective_chain(translators, default_target, default_translator).map_err(PyException::new_err)
    }

    #[pyo3(signature = (translators))]
    pub fn generate_list_py(&mut self, translators: Vec<(&str, &str)>) -> PyResult<()>{
        self.generate_list(translators).map_err(PyException::new_err)
    }

    #[pyo3(signature = ())]
    pub fn get_new_translator_instance_py(&self) -> PyResult<Translate>{
        self.get_new_translator_instance().map_err(PyException::new_err)
    }
}
