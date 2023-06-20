use image::{DynamicImage, GrayAlphaImage, GrayImage, ImageFormat, RgbImage, RgbaImage};
use image_writer::input::{
    Alignments, Background, Data, Font, HorizontalAlignment, Mode, OutputMode, Pos2, ReadDirection,
    Rgb, Rgba, Size2, Styling, Text, VerticalAlignment, Wrap,
};
use image_writer::save::OutputError;
use numpy::PyArray3;
use pyo3::prelude::*;
use std::io::Cursor;
use pyo3::exceptions::PyException;

#[pyclass]
pub struct PangoRenderer {
    data: Data,
    img_dim: [usize; 2],
    images: Vec<Vec<u8>>,
}

#[pymethods]
impl PangoRenderer {
    #[new]
    fn new(style: &str, vertical: bool) -> Self {
        let font = match style {
            "" => None,
            _ => Some(Font {
                font_family: style.to_string(),
                variant: Default::default(),
                stretch: Default::default(),
                weight: Default::default(),
                style: Default::default(),
            }),
        };
        let data = Data {
            items: vec![],
            global_style: Styling {
                spacing: None,
                line_spacing: None,
                ellipsize: Default::default(),
                wrap: Wrap::Word,
                indent: None,
                single_paragraph_mode: true,
                auto_dir: false,
                read_direction: match vertical {
                    false => ReadDirection::WeakLR,
                    true => ReadDirection::RL,
                },
                vertical,
                font,
                justiy: false,
                justify_last_line: false,
            },
            global_align: Alignments {
                ha: HorizontalAlignment::Left,
                va: VerticalAlignment::Top,
            },
            background: None,
        };
        PangoRenderer {
            data,
            img_dim: [0;2],
            images: vec![],
        }
    }

    #[pyo3(signature = (img))]
    fn add_image(&mut self, img: &PyArray3<u8>) -> PyResult<()> {
        let (width, height, img) = convert_numpy_array_to_image(img).map_err(PyException::new_err)?;

        self.data.background = Some(get_bytes(img));
        self.img_dim = [width, height];
        Ok(())
    }

    #[pyo3(signature = (width, height ))]
    fn set_size(&mut self, width: usize, height: usize) -> PyResult<()> {
        self.img_dim = [width, height];
    }

    #[pyo3(signature = (
    text,
    position,
    font_size, alignment,
    colors,
    bg
    ))]
    fn add_text(
        &mut self,
        text: &str,
        position: (f64, f64, f64, f64),
        font_size: u32,
        // horizontal_alignment,vertical_alignment,
        alignment: (&str, &str),
        //foreground, background
        colors: (u32, u32),
        bg: Option<&PyArray3<u8>>
    ) -> PyResult<()>{
        let color_to_rgb = |color: u32| {
            let red = ((color >> 16) & 0xFF) as u8;
            let green = ((color >> 8) & 0xFF) as u8;
            let blue = (color & 0xFF) as u8;
            let alpha = ((color >> 24) & 0xFF) as u8;
            Rgba::new(red as f64, green as f64, blue as f64, alpha as f64)
        };
        let bg = if let Some(bg) = bg {
            let (_,_,bg) = convert_numpy_array_to_image(bg).map_err(PyException::new_err)?;
            Background::Bytes(get_bytes(bg))
        }else {
            Background::Rgb(Rgb::new(255., 255., 255.))
        };
        let fg_color_rgba = color_to_rgb(colors.0);
        let fg_color = Rgb::new(fg_color_rgba.r, fg_color_rgba.g, fg_color_rgba.b);
        let bg_color = color_to_rgb(colors.1);
        let text = Text {
            mode: Mode::Text,
            value: text.to_string(),
            pos: Pos2::new(position.0, position.1),
            size: Size2::new(position.2, position.3),
            font_size: font_size as f64,
            font_color: fg_color,
            outline_color: bg_color,
            font_stroke: 5.0,
            background: bg,
            style: None,
            align: Some(Alignments {
                ha: match alignment.0 {
                    "left" => HorizontalAlignment::Left,
                    "center" => HorizontalAlignment::Center,
                    "right" => HorizontalAlignment::Right,
                    _ => panic!("Unsupported alignment: {}", alignment.0),
                },
                va: match alignment.1 {
                    "top" => VerticalAlignment::Top,
                    "center" => VerticalAlignment::Center,
                    "bottom" => VerticalAlignment::Bottom,
                    _ => panic!("Unsupported alignment: {}", alignment.1),
                },
            }),
        };
        self.data.items.push(text);
        Ok(())
    }

    #[pyo3(signature = (file_name, format, add_ext))]
    fn save(&mut self, file_name: &str, format: String, add_ext: bool) {
        let mut idents = format.split(' ').collect::<Vec<&str>>();
        let om = match idents.remove(0) {
            "pdf" => OutputMode::Pdf(idents.remove(0).parse().unwrap_or(false)),
            "png" => OutputMode::Png(idents.remove(0).parse().unwrap_or(false)),
            "svg" => OutputMode::Svg,
            "ps" => OutputMode::Ps,
            "jpeg" => OutputMode::Jpeg(idents.remove(0).parse().unwrap_or(100)),
            "ico" => OutputMode::Ico,
            "bmp" => OutputMode::Bmp,
            "farbfeld" => OutputMode::Farbfeld,
            "tga" => OutputMode::Tga,
            "openexr" => OutputMode::OpenExr,
            "tiff" => OutputMode::Tiff,
            "avif" => OutputMode::Avif,
            "qoi" => OutputMode::Qoi,
            "webp" => OutputMode::WebP,
            _ => panic!("Unsupported format: {}", format),
        };
        let painter = self.data.painter(&om, self.img_dim[0] as f64, self.img_dim[1] as f64).unwrap();
        match image_writer::save::output(om, file_name.into(), painter, add_ext) {
            Ok(v) => {
                if let Some(v) = v {
                    self.images.push(v);
                }
            }
            Err(e) => match e {
                OutputError::Custom(v) => {
                    println!("Error: {}", v);
                }
                OutputError::Io(e) => {
                    println!("Io Error: {}", e);
                }
                OutputError::ImageError(e) => {
                    println!("Image Error: {}", e);
                }
                OutputError::CompressionError(e) => {
                    println!("Compression Error: {}", e);
                }
            },
        }
    }
}

fn convert_numpy_array_to_image(img: &PyArray3<u8>) -> Result<(usize, usize, DynamicImage), String> {
    let img_data = img.readonly();
    let shape = img_data.shape();
    let height = shape[0];
    let width = shape[1];
    let channels = shape[2];
    if channels == 1 {
        Ok((width, height, DynamicImage::from(
            GrayImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap()).unwrap(),
        )))
    } else if channels == 2 {
        Ok((width, height, DynamicImage::from(
            GrayAlphaImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap())
                .unwrap(),
        )))
    } else if channels == 3 {
        Ok((width, height, DynamicImage::from(
            RgbImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap()).unwrap(),
        )))
    } else if channels == 4 {
        Ok((width, height, DynamicImage::from(
            RgbaImage::from_raw(width as u32, height as u32, img_data.to_vec().unwrap()).unwrap(),
        )))
    } else {
        Err(format!("Unsupported number of channels: {}", channels))
    }
}

fn get_bytes(img: DynamicImage) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    img.write_to(&mut buffer, ImageFormat::Png).unwrap();
    buffer.into_inner()
}