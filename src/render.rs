use image::{DynamicImage, GrayAlphaImage, GrayImage, ImageBuffer, RgbImage, Rgba, RgbaImage};
use numpy::PyArray3;
use pyo3::prelude::*;

#[pyclass]
pub struct PdfRenderer {
    background_image: Option<DynamicImage>,
}

#[pymethods]
impl PdfRenderer {
    #[new]
    fn new() -> Self {
        PdfRenderer {
            background_image: None,
        }
    }

    #[args(img)]
    fn set_background(&mut self, img: &PyArray3<u8>) {
        let img = convert_numpy_array_to_image(img);
        img.save("background.png").unwrap();
        self.background_image = Some(img);
    }

    #[args(
        text,
        position,
        font_size,
        alignment,
        font_family,
        style,
        color,
        vertical
    )]
    fn add_text(
        &self,
        text: &str,
        position: (f64, f64, f64, f64),
        font_size: u32,
        alignment: &str,
        font_family: &str,
        style: &str,
        color: u32,
        vertical: bool,
    ) {
        // Add text to the PDF with the provided parameters
        // ...

        // Convert color to RGBA
        let red = ((color >> 16) & 0xFF) as u8;
        let green = ((color >> 8) & 0xFF) as u8;
        let blue = (color & 0xFF) as u8;
        let alpha = ((color >> 24) & 0xFF) as u8;
        let rgba_color = Rgba([red, green, blue, alpha]);
        println!("rgba_color: {:?}", rgba_color)

        // Use the converted color
        // ...
    }

    #[args(file_name)]
    fn save(&self, file_name: &str) {
        // Save the PDF file with the provided file name
        // ...
    }
}

fn convert_numpy_array_to_image(img: &PyArray3<u8>) -> DynamicImage {
    let img_data = img.readonly();
    let shape = img_data.shape();
    let height = shape[0];
    let width = shape[1];
    let channels = shape[2];
    if channels == 1 {
        DynamicImage::from(
            GrayImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap()).unwrap(),
        )
    } else if channels == 2 {
        DynamicImage::from(
            GrayAlphaImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap())
                .unwrap(),
        )
    } else if channels == 3 {
        DynamicImage::from(
            RgbImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap()).unwrap(),
        )
    } else if channels == 4 {
        DynamicImage::from(
            RgbaImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap()).unwrap(),
        )
    } else {
        panic!("Unsupported number of channels: {}", channels);
    }
}
