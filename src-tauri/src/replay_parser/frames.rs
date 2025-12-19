use crate::replay_parser::ParseError;

use nom::{
    bytes::complete::take,
    number::complete::{le_u32, le_u8},
};

#[derive(Debug)]
pub struct Frame {
    pub frame_number: u32,
    pub commands: Vec<Command>,
}

#[derive(Debug)]
pub struct Command {
    pub command_type: u8,
    pub data: Vec<u8>,
}

pub fn parse_frames_section(input: &[u8]) -> Result<(&[u8], Vec<Frame>), ParseError> {
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

    let mut combined_data = Vec::new();
    for chunk in chunks {
        match decompress_zlib_chunk(chunk) {
            Ok(decompressed) => combined_data.extend_from_slice(&decompressed),
            Err(_) => {
                // assume uncompressed
                combined_data.extend_from_slice(chunk);
            }
        }
    }

    let frames = parse_frames_data(&combined_data)?;

    Ok((input, frames))
}

fn parse_frames_data(data: &[u8]) -> Result<Vec<Frame>, ParseError> {
    let mut frames = Vec::new();
    let mut input = data;

    while !input.is_empty() {
        match parse_frame(input) {
            Ok((remaining, frame)) => {
                frames.push(frame);
                input = remaining;
            }
            Err(_) => break,
        }
    }

    Ok(frames)
}

fn parse_frame(input: &[u8]) -> Result<(&[u8], Frame), ParseError> {
    let (input, frame_number) = le_u32(input)?;
    let (input, block_size) = le_u8(input)?;
    let (input, frame_data) = take(block_size)(input)?;

    let commands = parse_commands(frame_data)?;

    Ok((
        input,
        Frame {
            frame_number,
            commands,
        },
    ))
}

fn parse_commands(data: &[u8]) -> Result<Vec<Command>, ParseError> {
    let mut commands = Vec::new();
    let mut input = data;

    while !input.is_empty() {
        match parse_command(input) {
            Ok((remaining, command)) => {
                commands.push(command);
                input = remaining;
            }
            Err(_) => break,
        }
    }

    Ok(commands)
}

fn parse_command(input: &[u8]) -> Result<(&[u8], Command), ParseError> {
    let (input, _player_id) = le_u8(input)?;
    let (input, command_type) = le_u8(input)?;

    let (input, data) = match command_type {
        // Commands with no additional data
        0x18 | 0x19 | 0x10 | 0x11 | 0x08 | 0x5a | 0x5b | 0x1b | 0x1c | 0x1d | 0x27 | 0x2a
        | 0x2e | 0x31 | 0x33 | 0x34 | 0x36 | 0x38 | 0x39 | 0x3c | 0x54 => (input, Vec::new()),
        // Commands with 1 byte
        0x0f | 0x20 | 0x21 | 0x22 | 0x30 | 0x32 | 0x55 | 0x57 => {
            let (input, byte) = le_u8(input)?;
            (input, vec![byte])
        }
        // Commands with 2 bytes
        0x29 | 0x62 => {
            let (input, data) = take(2usize)(input)?;
            (input, data.to_vec())
        }
        0x5c => {
            let (input, sender) = le_u8(input)?;
            let (input, message_data) = take(80usize)(input)?;
            let mut data = vec![sender];
            data.extend_from_slice(message_data);
            (input, data)
        }
        _ => {
            if input.is_empty() {
                (input, Vec::new())
            } else {
                let (input, byte) = le_u8(input)?;
                (input, vec![byte])
            }
        }
    };

    Ok((input, Command { command_type, data }))
}

fn decompress_zlib_chunk(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
