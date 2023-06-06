use image::{DynamicImage, GrayAlphaImage, GrayImage, ImageFormat, RgbImage, RgbaImage};
use image_writer::input::{
    Alignments, Background, Data, Font, HorizontalAlignment, Mode, OutputMode, Pos2, ReadDirection,
    Rgb, Rgba, Size2, Styling, Text, VerticalAlignment, Wrap,
};
use image_writer::save::OutputError;
use numpy::PyArray3;
use pyo3::prelude::*;
use std::io::Cursor;

#[pyclass]
pub struct PangoRenderer {
    data: Data,
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
            images: vec![],
        }
    }

    #[pyo3(signature = (img))]
    fn set_background(&mut self, img: &PyArray3<u8>) {
        let img = convert_numpy_array_to_image(img);
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Png).unwrap();
        self.data.background = Some(buffer.into_inner());
    }

    #[pyo3(signature = (
    text,
    position,
    font_size,
    halignment,
    valignment,
    color,
    ))]
    fn add_text(
        &mut self,
        text: &str,
        position: (f64, f64, f64, f64),
        font_size: u32,
        halignment: &str,
        valignment: &str,
        color: u32,
    ) {
        let red = ((color >> 16) & 0xFF) as u8;
        let green = ((color >> 8) & 0xFF) as u8;
        let blue = (color & 0xFF) as u8;
        //let alpha = ((color >> 24) & 0xFF) as u8;
        let rgba_color = Rgb::new(red as f64, green as f64, blue as f64);
        let text = Text {
            mode: Mode::Text,
            value: text.to_string(),
            pos: Pos2::new(position.0, position.1),
            size: Size2::new(position.2, position.3),
            font_size: font_size as f64,
            font_color: rgba_color,
            outline_color: Rgba::new(0.0, 0.0, 1.0, 0.5),
            font_stroke: 5.0,
            background: Background::Rgb(Rgb::new(1.0, 0.0, 0.0)),
            style: None,
            align: Some(Alignments {
                ha: match halignment {
                    "left" => HorizontalAlignment::Left,
                    "center" => HorizontalAlignment::Center,
                    "right" => HorizontalAlignment::Right,
                    _ => panic!("Unsupported alignment: {}", halignment),
                },
                va: match valignment {
                    "top" => VerticalAlignment::Top,
                    "center" => VerticalAlignment::Center,
                    "bottom" => VerticalAlignment::Bottom,
                    _ => panic!("Unsupported alignment: {}", valignment),
                },
            }),
        };
        self.data.items.push(text);
    }

    #[pyo3(signature = (file_name, width, height, format))]
    fn save(&mut self, file_name: &str, width: f64, height: f64, format: String) {
        let mut idents = format.split(' ').collect::<Vec<&str>>();
        let om = match idents.remove(0) {
            "pdf" => OutputMode::Pdf(idents.remove(0).parse().unwrap()),
            "png" => OutputMode::Png(idents.remove(0).parse().unwrap()),
            "svg" => OutputMode::Svg,
            "ps" => OutputMode::Ps,
            "jpeg" => OutputMode::Jpeg(idents.remove(0).parse().unwrap()),
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
        let painter = self.data.painter(&om, width, height).unwrap();
        match image_writer::save::output(om, file_name.into(), painter) {
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
