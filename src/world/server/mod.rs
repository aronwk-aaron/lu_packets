//! All packets a world server can receive.
mod gm;

use std::io::{Read, Write};
use std::io::Result as Res;

use endio::{Deserialize, LERead, LEWrite, Serialize};
use endio::LittleEndian as LE;
use lu_packets_derive::ServiceMessage;

use crate::common::{err, ObjId, LuWStr33, LuWStr42, ServiceId, ZoneId};
use crate::chat::server::ChatMessage;
use self::gm::SubjectGameMessage;

pub use crate::general::server::GeneralMessage;

pub type Message = crate::raknet::server::Message<LuMessage>;

#[derive(Debug, Deserialize)]
#[non_exhaustive]
#[repr(u16)]
pub enum LuMessage {
	General(GeneralMessage) = ServiceId::General as u16,
	World(WorldMessage) = ServiceId::World as u16,
}

#[derive(Debug, ServiceMessage)]
#[repr(u32)]
pub enum WorldMessage {
	ClientValidation(ClientValidation) = 1,
	CharacterListRequest = 2,
	CharacterCreateRequest(CharacterCreateRequest) = 3,
	CharacterLoginRequest(CharacterLoginRequest) = 4,
	SubjectGameMessage(SubjectGameMessage) = 5,
	CharacterDeleteRequest(CharacterDeleteRequest) = 6,
	GeneralChatMessage(GeneralChatMessage) = 14,
	LevelLoadComplete(LevelLoadComplete) = 19,
	RouteMessage(RouteMessage) = 21,
	StringCheck(StringCheck) = 25,
	RequestFreeTrialRefresh = 32,
	UgcDownloadFailed(UgcDownloadFailed) = 120,
}

#[derive(Debug)]
pub struct ClientValidation {
	pub username: LuWStr33,
	pub session_key: LuWStr33,
	pub fdb_checksum: [u8; 32],
}

impl<R: Read+LERead> Deserialize<LE, R> for ClientValidation
	where   u8: Deserialize<LE, R>,
	  LuWStr33: Deserialize<LE, R> {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let username         = LERead::read(reader)?;
		let session_key      = LERead::read(reader)?;
		let mut fdb_checksum = [0; 32];
		std::io::Read::read(reader, &mut fdb_checksum)?;
		// garbage byte because the devs messed up the null terminator
		let _ : u8           =  LERead::read(reader)?;
		Ok(Self {
			username,
			session_key,
			fdb_checksum,
		})
	}
}

impl<'a, W: Write+LEWrite> Serialize<LE, W> for &'a ClientValidation
	where       u8: Serialize<LE, W>,
	  &'a LuWStr33: Serialize<LE, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		LEWrite::write(writer, &self.username)?;
		LEWrite::write(writer, &self.session_key)?;
		Write::write(writer, &self.fdb_checksum)?;
		// garbage byte because the devs messed up the null terminator
		LEWrite::write(writer, 0u8)
	}
}

#[derive(Debug)]
pub struct CharacterCreateRequest {
	pub char_name: LuWStr33,
	pub predef_name_ids: (u32, u32, u32),
	pub shirt_color: u32,
	pub pants_color: u32,
	pub hair_style: u32,
	pub hair_color: u32,
	pub eyebrow_style: u32,
	pub eye_style: u32,
	pub mouth_style: u32,
}

impl<R: LERead> Deserialize<LE, R> for CharacterCreateRequest
	where   u8: Deserialize<LE, R>,
	       u32: Deserialize<LE, R>,
	  LuWStr33: Deserialize<LE, R> {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let char_name = reader.read()?;
		let name_id_1 = reader.read()?;
		let name_id_2 = reader.read()?;
		let name_id_3 = reader.read()?;
		let predef_name_ids = (name_id_1, name_id_2, name_id_3);
		let _unused: u8   = reader.read()?;
		let _unused: u32  = reader.read()?;
		let _unused: u32  = reader.read()?;
		let shirt_color   = reader.read()?;
		let _unused: u32  = reader.read()?;
		let pants_color   = reader.read()?;
		let hair_style    = reader.read()?;
		let hair_color    = reader.read()?;
		let _unused: u32  = reader.read()?;
		let _unused: u32  = reader.read()?;
		let eyebrow_style = reader.read()?;
		let eye_style     = reader.read()?;
		let mouth_style   = reader.read()?;
		let _unused: u8   = reader.read()?;

		Ok(Self {
			char_name,
			predef_name_ids,
			shirt_color,
			pants_color,
			hair_style,
			hair_color,
			eyebrow_style,
			eye_style,
			mouth_style,
		})
	}
}

impl<'a, W: LEWrite> Serialize<LE, W> for &'a CharacterCreateRequest
	where      u8: Serialize<LE, W>,
	          u32: Serialize<LE, W>,
	 &'a LuWStr33: Serialize<LE, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write(&self.char_name)?;
		writer.write(self.predef_name_ids.0)?;
		writer.write(self.predef_name_ids.1)?;
		writer.write(self.predef_name_ids.2)?;
		writer.write(0u8)?;
		writer.write(0u32)?;
		writer.write(0u32)?;
		writer.write(self.shirt_color)?;
		writer.write(0u32)?;
		writer.write(self.pants_color)?;
		writer.write(self.hair_style)?;
		writer.write(self.hair_color)?;
		writer.write(0u32)?;
		writer.write(0u32)?;
		writer.write(self.eyebrow_style)?;
		writer.write(self.eye_style)?;
		writer.write(self.mouth_style)?;
		writer.write(0u8)
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterLoginRequest {
	pub char_id: ObjId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterDeleteRequest {
	pub char_id: ObjId,
}

#[derive(Debug)]
pub struct GeneralChatMessage {
	pub chat_channel: u8, // todo: type?
	pub source_id: u16,
	pub message: String,
}

impl<R: Read+LERead> Deserialize<LE, R> for GeneralChatMessage
	where u8: Deserialize<LE, R>,
	     u16: Deserialize<LE, R>,
	     u32: Deserialize<LE, R> {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let chat_channel    = LERead::read(reader)?;
		let source_id       = LERead::read(reader)?;
		let string_len: u32 = LERead::read(reader)?;
		let mut string = vec![0; (string_len*2) as usize];
		let mut taken = Read::take(reader, (string_len*2) as u64);
		Read::read(&mut taken, &mut string)?;
		let string_slice: &[u16] = unsafe { std::slice::from_raw_parts(string.as_ptr() as *const u16, string_len as usize - 1) };
		let message = String::from_utf16_lossy(string_slice);

		Ok(Self { chat_channel, source_id, message })
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LevelLoadComplete {
	pub zone_id: ZoneId,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RouteMessage {
	Chat(ChatMessage),
}

impl<R: LERead> Deserialize<LE, R> for RouteMessage
	where     u32: Deserialize<LE, R>,
	    ServiceId: Deserialize<LE, R>,
	  ChatMessage: Deserialize<LE, R> {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let _packet_size: u32 = reader.read()?;
		let service_id: ServiceId = reader.read()?;
		Ok(match service_id {
			ServiceId::Chat => {
				Self::Chat(reader.read()?)
			}
			_ => {
				return err("route service id", service_id);
			}
		})
	}
}

impl<'a, W: LEWrite> Serialize<LE, W> for &'a RouteMessage
	where         u32: Serialize<LE, W>,
	    &'a ServiceId: Serialize<LE, W>,
	  &'a ChatMessage: Serialize<LE, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		writer.write(0u32)?; // packet size, unused in this server's impl
		writer.write(&ServiceId::Chat)?;
		match self {
			RouteMessage::Chat(msg) => { writer.write(msg)?; }
		}
		Ok(())
	}
}

#[derive(Debug)]
pub struct StringCheck {
	pub chat_mode: u8, // todo: type?
	pub chat_channel: u8, // todo: type?
	pub recipient_name: LuWStr42,
	pub string: String,
}

impl<R: Read+LERead> Deserialize<LE, R> for StringCheck
	where   u8: Deserialize<LE, R>,
	       u16: Deserialize<LE, R>,
	  LuWStr42: Deserialize<LE, R> {
	fn deserialize(reader: &mut R) -> Res<Self> {
		let chat_mode       = LERead::read(reader)?;
		let chat_channel    = LERead::read(reader)?;
		let recipient_name  = LERead::read(reader)?;
		let string_len: u16 = LERead::read(reader)?;
		let mut string = vec![0; (string_len*2) as usize];
		let mut taken = Read::take(reader, (string_len*2) as u64);
		Read::read(&mut taken, &mut string)?;
		let string_slice: &[u16] = unsafe { std::slice::from_raw_parts(string.as_ptr() as *const u16, string_len as usize) };
		let string = String::from_utf16_lossy(string_slice);

		Ok(Self { chat_mode, chat_channel, recipient_name, string })
	}
}

impl<'a, W: Write+LEWrite> Serialize<LE, W> for &'a StringCheck
	where   u8: Serialize<LE, W>,
	       u16: Serialize<LE, W>,
	  &'a LuWStr42: Serialize<LE, W> {
	fn serialize(self, writer: &mut W) -> Res<()> {
		LEWrite::write(writer, self.chat_mode)?;
		LEWrite::write(writer, self.chat_channel)?;
		LEWrite::write(writer, &self.recipient_name)?;
		let utf16_str: Vec<u16> = self.string.encode_utf16().collect();
		LEWrite::write(writer, utf16_str.len() as u16)?;
		let utf16_str_slice: &[u8] = unsafe { std::slice::from_raw_parts(utf16_str.as_ptr() as *const u8, utf16_str.len()*2) };
		Write::write(writer, utf16_str_slice)?;
		Ok(())
	}
}

#[derive(Debug, Deserialize, Serialize)]
#[repr(u32)]
pub enum UgcResType {
	Lxfml,
	Nif,
	Hkx,
	Dds,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UgcDownloadFailed {
	pub res_type: UgcResType,
	pub blueprint_id: ObjId,
	pub status_code: u32,
	pub char_id: ObjId,
}