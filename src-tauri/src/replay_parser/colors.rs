use crate::replay_parser::ParseError;

use nom::{
    bytes::complete::take,
    number::complete::{le_f32, le_u32},
};

#[derive(Debug, Clone)]
pub struct PlayerColor {
    pub name: String,
    pub rgb: u32,
}

impl PlayerColor {
    pub fn new(name: String, rgb: u32) -> Self {
        Self { name, rgb }
    }

    pub fn hex_string(&self) -> String {
        format!("#{:06x}", self.rgb)
    }

    pub fn r(&self) -> u8 {
        ((self.rgb >> 16) & 0xff) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.rgb >> 8) & 0xff) as u8
    }

    pub fn b(&self) -> u8 {
        (self.rgb & 0xff) as u8
    }
}

pub fn get_colors() -> Vec<PlayerColor> {
    vec![
        PlayerColor {
            name: "Red".to_string(),
            rgb: 0xf40404,
        },
        PlayerColor {
            name: "Blue".to_string(),
            rgb: 0x0c48cc,
        },
        PlayerColor {
            name: "Teal".to_string(),
            rgb: 0x2cb494,
        },
        PlayerColor {
            name: "Purple".to_string(),
            rgb: 0x88409c,
        },
        PlayerColor {
            name: "Orange".to_string(),
            rgb: 0xf88c14,
        },
        PlayerColor {
            name: "Brown".to_string(),
            rgb: 0x703014,
        },
        PlayerColor {
            name: "White".to_string(),
            rgb: 0xcce0d0,
        },
        PlayerColor {
            name: "Yellow".to_string(),
            rgb: 0xfcfc38,
        },
        PlayerColor {
            name: "Green".to_string(),
            rgb: 0x088008,
        },
        PlayerColor {
            name: "Pale Yellow".to_string(),
            rgb: 0xfcfc7c,
        },
        PlayerColor {
            name: "Tan".to_string(),
            rgb: 0xecc4b0,
        },
        PlayerColor {
            name: "Aqua".to_string(),
            rgb: 0x4068d4,
        },
        PlayerColor {
            name: "Pale Green".to_string(),
            rgb: 0x74a47c,
        },
        PlayerColor {
            name: "Blueish Grey".to_string(),
            rgb: 0x9090b8,
        },
        PlayerColor {
            name: "Pale Yellow2".to_string(),
            rgb: 0xfcfc7c,
        },
        PlayerColor {
            name: "Cyan".to_string(),
            rgb: 0x00e4fc,
        },
        PlayerColor {
            name: "Pink".to_string(),
            rgb: 0xffc4e4,
        },
        PlayerColor {
            name: "Olive".to_string(),
            rgb: 0x787800,
        },
        PlayerColor {
            name: "Lime".to_string(),
            rgb: 0xd2f53c,
        },
        PlayerColor {
            name: "Navy".to_string(),
            rgb: 0x0000e6,
        },
        PlayerColor {
            name: "Dark Aqua".to_string(),
            rgb: 0x4068d4,
        },
        PlayerColor {
            name: "Magenta".to_string(),
            rgb: 0xf032e6,
        },
        PlayerColor {
            name: "Grey".to_string(),
            rgb: 0x808080,
        },
        PlayerColor {
            name: "Black".to_string(),
            rgb: 0x3c3c3c,
        },
    ]
}

pub fn parse_colors_section(input: &[u8]) -> Result<(&[u8], Vec<Option<PlayerColor>>), ParseError> {
    let (input, _section_size) = le_u32(input)?;

    let (input, _crc) = le_u32(input)?;
    let (input, num_chunks) = le_u32(input)?;

    let mut chunks = Vec::new();
    let mut input = input;

    for _ in 0..num_chunks {
        let (new_input, chunk_size) = le_u32(input)?;
        let (new_input, chunk_data) = take(chunk_size)(new_input)?;
        chunks.push(chunk_data);
        input = new_input;
    }

    // Decompress and concatenate chunks
    let mut combined_data = Vec::new();
    for chunk in chunks {
        match decompress_chunk(chunk) {
            Ok(decompressed) => combined_data.extend_from_slice(&decompressed),
            Err(_) => {
                // If decompression fails, try as uncompressed
                combined_data.extend_from_slice(chunk);
            }
        }
    }

    let colors = parse_colors_data(&combined_data)?;

    Ok((input, colors))
}

/// Parse colors data from decompressed buffer
fn parse_colors_data(data: &[u8]) -> Result<Vec<Option<PlayerColor>>, ParseError> {
    let mut colors = Vec::new();
    let mut input = data;

    // Parse 12 color entries
    for _ in 0..12 {
        let (new_input, color) = parse_color_entry(input)?;
        colors.push(color);
        input = new_input;
    }

    Ok(colors)
}

/// Parse a single color entry
fn parse_color_entry(input: &[u8]) -> Result<(&[u8], Option<PlayerColor>), ParseError> {
    // Each color entry is 4 floats (R, G, B, A)
    let (input, r) = le_f32(input)?;
    let (input, g) = le_f32(input)?;
    let (input, b) = le_f32(input)?;
    let (input, a) = le_f32(input)?;

    // Convert floats to bytes
    let r_byte = (r * 255.0).round() as u8;
    let g_byte = (g * 255.0).round() as u8;
    let b_byte = (b * 255.0).round() as u8;
    let _a_byte = (a * 255.0).round() as u8;

    // Try to find matching predefined color
    let colors = get_colors();
    let color = colors
        .iter()
        .find(|color| color.r() == r_byte && color.g() == g_byte && color.b() == b_byte)
        .cloned();

    Ok((input, color))
}

/// Decompress a zlib-compressed chunk  
fn decompress_chunk(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
