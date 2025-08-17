use crate::replay_parser::ParseError;

use nom::{
    bytes::complete::take,
    number::complete::{le_u16, le_u32, le_u8},
    IResult,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct GameInfo {
    pub engine: i8,
    pub frames: u32,
    pub start_time: SystemTime,
    pub title: String,
    pub map_width: u16,
    pub map_height: u16,
    pub available_slots_count: u8,
    pub speed: u8,
    pub game_type: u16,
    pub sub_type: u16,
    pub host: String,
    pub map: String,
    pub player_structs: Vec<PlayerStruct>,
    pub player_colors: Vec<PlayerColorInfo>,
}

#[derive(Debug)]
pub struct PlayerStruct {
    pub slot_id: u16,
    pub id: u8,
    pub player_type: u8,
    pub race: Race,
    pub team: u8,
    pub name: String,
}

#[derive(Debug)]
pub enum Race {
    Zerg,
    Terran,
    Protoss,
    Unknown(u8),
}

impl From<u8> for Race {
    fn from(value: u8) -> Self {
        match value {
            0 => Race::Zerg,
            1 => Race::Terran,
            2 => Race::Protoss,
            _ => Race::Unknown(value),
        }
    }
}

#[derive(Debug)]
pub struct PlayerColorInfo {
    pub color: u32,
}

pub fn parse_game_info_section(input: &[u8]) -> Result<(&[u8], GameInfo), ParseError> {
    let (input, _crc) = le_u32(input)?;
    let (input, num_chunks) = le_u32(input)?;

    if num_chunks != 1 {
        return Err(ParseError::InvalidData(format!(
            "Expected 1 chunk for game info, got {}",
            num_chunks
        )));
    }

    let (input, chunk_size) = le_u32(input)?;
    let (input, chunk_data) = take(chunk_size)(input)?;

    let decompressed_data = match decompress_chunk(chunk_data) {
        Ok(data) => data,
        Err(_) => chunk_data.to_vec(),
    };

    let (_, game_info) = parse_game_info_data(&decompressed_data)?;

    Ok((input, game_info))
}

fn parse_game_info_data(input: &[u8]) -> Result<(&[u8], GameInfo), ParseError> {
    let (input, engine) = nom::number::complete::i8(input)?;
    let (input, frames) = le_u32(input)?;
    let (input, _) = take(3usize)(input)?;
    let (input, start_time_unix) = le_u32(input)?;
    let (input, _) = take(12usize)(input)?;

    let (input, title_bytes) = take(28usize)(input)?;
    let title = parse_null_terminated_string(title_bytes);

    let (input, map_width) = le_u16(input)?;
    let (input, map_height) = le_u16(input)?;
    let (input, _) = take(1usize)(input)?;
    let (input, available_slots_count) = le_u8(input)?;
    let (input, speed) = le_u8(input)?;
    let (input, _) = take(1usize)(input)?;
    let (input, game_type) = le_u16(input)?;
    let (input, sub_type) = le_u16(input)?;
    let (input, _) = take(8usize)(input)?;

    let (input, host_bytes) = take(24usize)(input)?;
    let host = parse_null_terminated_string(host_bytes);

    let (input, _) = take(1usize)(input)?;

    let (input, map_bytes) = take(26usize)(input)?;
    let map = parse_null_terminated_string(map_bytes);

    let (input, _) = take(38usize)(input)?;

    let mut player_structs = Vec::new();
    let mut input = input;
    for _ in 0..12 {
        let (new_input, player_struct) = parse_player_struct(input)?;
        player_structs.push(player_struct);
        input = new_input;
    }

    let mut player_colors = Vec::new();
    for _ in 0..8 {
        let (new_input, player_color) = parse_player_color(input)?;
        player_colors.push(player_color);
        input = new_input;
    }

    let start_time = UNIX_EPOCH + std::time::Duration::from_secs(start_time_unix as u64);

    Ok((
        input,
        GameInfo {
            engine,
            frames,
            start_time,
            title,
            map_width,
            map_height,
            available_slots_count,
            speed,
            game_type,
            sub_type,
            host,
            map,
            player_structs,
            player_colors,
        },
    ))
}

fn parse_player_struct(input: &[u8]) -> IResult<&[u8], PlayerStruct> {
    let (input, slot_id) = le_u16(input)?;
    let (input, _) = take(2usize)(input)?;
    let (input, id) = le_u8(input)?;
    let (input, _) = take(3usize)(input)?;
    let (input, player_type) = le_u8(input)?;
    let (input, race_value) = le_u8(input)?;
    let (input, team) = le_u8(input)?;
    let (input, name_bytes) = take(25usize)(input)?;

    let race = Race::from(race_value);
    let name = parse_null_terminated_string(name_bytes);

    Ok((
        input,
        PlayerStruct {
            slot_id,
            id,
            player_type,
            race,
            team,
            name,
        },
    ))
}

fn parse_player_color(input: &[u8]) -> IResult<&[u8], PlayerColorInfo> {
    let (input, color) = le_u32(input)?;
    Ok((input, PlayerColorInfo { color }))
}

fn parse_null_terminated_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

fn decompress_chunk(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
