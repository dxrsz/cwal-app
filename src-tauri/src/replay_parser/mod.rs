use nom::{bytes::complete::take, number::complete::le_u32, IResult};

pub mod colors;
pub mod error;
pub mod frames;
pub mod game_info;
pub mod header;

pub use colors::PlayerColor;
pub use error::ParseError;
pub use frames::{Command, Frame};
pub use game_info::{GameInfo, PlayerStruct};
pub use header::Header;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender_name: String,
    pub message: String,
    pub frame_number: u32,
    pub sender_id: u8,
}

#[derive(Debug)]
pub struct ReplayParser<'a> {
    input: &'a [u8],
}

impl<'a> ReplayParser<'a> {
    /// Create a new parser from raw replay file data
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    pub fn parse(&self) -> Result<ParsedReplay, ParseError> {
        let (remaining, header) = header::parse_header(self.input)?;

        if header.replay_version != "seRS" {
            return Err(ParseError::UnsupportedVersion(header.replay_version));
        }

        let (remaining, game_info) = game_info::parse_game_info_section(remaining)?;

        let (remaining, _) = skip_section(remaining).map_err(|e| {
            ParseError::InvalidData(format!("Failed to skip section size section: {:?}", e))
        })?;

        let (_, frames) = frames::parse_frames_section(remaining)?;

        Ok(ParsedReplay {
            header,
            game_info,
            frames,
            player_colors: vec![],
        })
    }
}

#[derive(Debug)]
pub struct ParsedReplay {
    pub header: Header,
    pub game_info: GameInfo,
    pub frames: Vec<Frame>,
    pub player_colors: Vec<Option<PlayerColor>>,
}

impl ParsedReplay {
    pub fn duration_ms(&self) -> u32 {
        self.game_info.frames * 42 // 42 ms per frame
    }

    pub fn duration_minutes(&self) -> f32 {
        self.duration_ms() as f32 / 1000.0 / 60.0
    }

    pub fn player_apm(&self, player_id: u8) -> u32 {
        let command_count = self
            .frames
            .iter()
            .flat_map(|frame| &frame.commands)
            .filter(|cmd| cmd.player_id == player_id)
            .count();

        if self.duration_minutes() > 0.0 {
            (command_count as f32 / self.duration_minutes()) as u32
        } else {
            0
        }
    }

    pub fn chat_messages(&self) -> Vec<ChatMessage> {
        let mut messages = Vec::new();

        for frame in &self.frames {
            for command in &frame.commands {
                if command.command_type == 0x5c {
                    if let Some(chat_msg) = Self::parse_chat_command(
                        command,
                        &self.game_info.player_structs,
                        frame.frame_number,
                    ) {
                        messages.push(chat_msg);
                    }
                }
            }
        }

        messages
    }

    fn parse_chat_command(
        command: &Command,
        players: &[PlayerStruct],
        frame_number: u32,
    ) -> Option<ChatMessage> {
        if command.command_type != 0x5c {
            return None;
        }

        if command.data.len() < 81 {
            return None;
        }

        let sender_id = command.data[0];
        let message_bytes = &command.data[1..81];

        let null_pos = message_bytes.iter().position(|&b| b == 0).unwrap_or(80);

        let message = match std::str::from_utf8(&message_bytes[..null_pos]) {
            Ok(s) => s.trim().to_string(),
            Err(_) => String::from_utf8_lossy(&message_bytes[..null_pos])
                .trim()
                .to_string(),
        };

        if message.is_empty() {
            return None;
        }

        let sender_name = players
            .iter()
            .find(|p| p.slot_id == sender_id as u16)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {}", sender_id));

        Some(ChatMessage {
            sender_name,
            message,
            frame_number,
            sender_id,
        })
    }
}

fn skip_section(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, _crc) = le_u32(input)?;
    let (input, num_chunks) = le_u32(input)?;

    let mut remaining = input;
    for _ in 0..num_chunks {
        let (new_remaining, chunk_size) = le_u32(remaining)?;
        let (new_remaining, _chunk_data) = take(chunk_size)(new_remaining)?;
        remaining = new_remaining;
    }

    Ok((remaining, ()))
}
