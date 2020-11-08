//! Client-received world messages.
use std::io::Result as Res;

use endio::{Deserialize, LERead, LEWrite, Serialize};
use endio::LittleEndian as LE;
use lu_packets_derive::{FromVariants, VariantTests};

use crate::common::{ObjId, LuString33, LuWString33, LVec};
use super::{Lot, lnv::LuNameValue, Vector3, ZoneId};
use super::gm::client::SubjectGameMessage;

/// All LU messages that can be received by a client from a world server.
pub type LuMessage = crate::general::client::LuMessage<ClientMessage>;
/// All messages that can be received by a client from a world server.
pub type Message = crate::raknet::client::Message<LuMessage>;

impl From<ClientMessage> for Message {
	fn from(msg: ClientMessage) -> Self {
		LuMessage::Client(msg).into()
	}
}

/// All client-received world messages.
#[derive(Debug, Deserialize, PartialEq, Serialize, FromVariants, VariantTests)]
#[non_exhaustive]
#[post_disc_padding=1]
#[repr(u32)]
pub enum ClientMessage {
	LoadStaticZone(LoadStaticZone) = 2,
	CreateCharacter(CreateCharacter) = 4,
	CharacterListResponse(CharacterListResponse) = 6,
	CharacterCreateResponse(CharacterCreateResponse) = 7,
	CharacterDeleteResponse(CharacterDeleteResponse) = 11,
	SubjectGameMessage(SubjectGameMessage) = 12,
	TransferToWorld(TransferToWorld) = 14,
	BlueprintLoadItemResponse(BlueprintLoadItemResponse) = 23,
	AddFriendRequest(AddFriendRequest) = 27,
	TeamInvite(TeamInvite) = 35,
	MinimumChatModeResponse(MinimumChatModeResponse) = 57,
	MinimumChatModeResponsePrivate(MinimumChatModeResponsePrivate) = 58,
	UpdateFreeTrialStatus(UpdateFreeTrialStatus) = 62,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[repr(u32)]
pub enum InstanceType {
	Public,
	Single,
	Team,
	Guild,
	Match,
}

/**
	Tells the client to load a zone.

	### Trigger
	May be sent at any time. However, in a typical server instance architecture, this message will usually be sent as the first message directly after the client has validated itself with [`ClientValidation`](super::server::ClientValidation).

	### Handling
	Load the zone specified in [`zone_id`](Self::zone_id), whatever that may entail for your client implementation.

	### Response
	Respond with [`LevelLoadComplete`](super::server::LevelLoadComplete) once you're done loading.

	### Notes
	Server instances are usually statically assigned to host a "parallel universe" of a certain zone (world), which means that this message will be sent directly after client validation. However, other instance architectures are theoretically possible:

	- Dynamic changing of the instance's zone, in which case additional [`LoadStaticZone`] messages could be sent (when the zone is changed).

	- Shared/overlapping instances, where the instance connection changes as the player moves around in the world, or where instances take over from others (e.g. in the event of a reboot), with mobs and all other state being carried over. In this case the client would be instructed to connect to the new instance via [`TransferToWorld`], but would not receive a [`LoadStaticZone`] afterwards. If done correctly, the player wouldn't even notice the transfer at all.

	However, these are quite advanced architectures, and for now it is unlikely that any server project will actually pull these off.
*/
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct LoadStaticZone {
	/// ID of the zone to be loaded.
	pub zone_id: ZoneId,
	/// Checksum on the map on the server side. The original LU client will refuse to load any map where the client side checksum doesn't match the server checksum, to prevent inconsistencies and cheating.
	pub map_checksum: u32,
	// editor enabled and editor level, unused
	#[padding=2]
	/// The position of the player in the new world, likely used to be able to load the right part of the world.
	pub player_position: Vector3,
	/// The instance type of the zone being loaded.
	pub instance_type: InstanceType,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateCharacter {
	pub data: LuNameValue,
}

/**
	Provides the list of characters of the client's account.

	### Trigger
	Receipt of [`CharacterListRequest`](super::server::WorldMessage::CharacterListRequest). Also sent in response to [`CharacterCreateRequest`](super::server::CharacterCreateRequest) after [`CharacterCreateResponse`] if the creation is successful.

	### Handling
	Display the characters to the user for selection.

	### Response
	None.

	### Notes
	The LU client can't handle sending more than four characters.
*/
#[derive(Debug, PartialEq)]
pub struct CharacterListResponse {
	/// Index into the list of characters below, specifying which character was used last.
	pub selected_char: u8,
	/// The list of characters.
	pub chars: Vec<CharListChar>,
}

impl<R: LERead> Deserialize<LE, R> for CharacterListResponse
	where       u8: Deserialize<LE, R>,
	  CharListChar: Deserialize<LE, R> {
	fn deserialize(reader: &mut R) -> Res<Self>	{
		let len: u8 = reader.read()?;
		let selected_char = reader.read()?;
		let mut chars = Vec::with_capacity(len as usize);
		for _ in 0..len {
			chars.push(reader.read()?);
		}
		Ok(Self { selected_char, chars } )
	}
}

impl<'a, W: LEWrite> Serialize<LE, W> for &'a CharacterListResponse
	where           u8: Serialize<LE, W>,
	  &'a CharListChar: Serialize<LE, W> {
	fn serialize(self, writer: &mut W) -> Res<()>	{
		writer.write(self.chars.len() as u8)?;
		writer.write(self.selected_char)?;
		for chr in self.chars.iter() {
			writer.write(chr)?;
		}
		Ok(())
	}
}

/// A character from the [`CharacterListResponse`] message.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct CharListChar {
	pub obj_id: ObjId,
	#[padding=4]
	pub char_name: LuWString33,
	pub pending_name: LuWString33,
	pub requires_rename: bool,
	pub is_free_trial: bool,
	#[padding=10]
	pub torso_color: u32,
	#[padding=4]
	pub legs_color: u32,
	pub hair_style: u32,
	pub hair_color: u32,
	#[padding=8]
	pub eyebrow_style: u32,
	pub eye_style: u32,
	pub mouth_style: u32,
	#[padding=4]
	pub last_location: ZoneId,
	#[padding=8]
	pub equipped_items: LVec<Lot, u16>,
}

/**
	Reports the result of a character create request.

	### Trigger
	Receipt of [`CharacterCreateRequest`](super::server::CharacterCreateRequest).

	### Handling
	If the variant is not [`Success`](CharacterCreateResponse::Success), display an appropriate error message and let the user try again. If successful, wait for the updated [`CharacterListResponse`] packet to arrive and display the new character list.

	### Response
	None.
*/
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[repr(u8)]
pub enum CharacterCreateResponse {
	/// The character has been successfully created.
	Success,
	/// Something went wrong during creation.
	GeneralFailure,
	/// The selected name is not allowed by the name moderation policy.
	NameNotAllowed,
	/// The ThreePartName is already in use.
	PredefinedNameInUse,
	/// The custom name is already in use.
	CustomNameInUse,
}

/**
	Reports the result of a character delete request.

	### Trigger
	Receipt of [`CharacterDeleteRequest`](super::server::CharacterDeleteRequest).

	### Handling
	Delete the character locally if [`success`](Self::success) is `true`, else display an error message and keep the character.

	### Response
	None.
*/
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct CharacterDeleteResponse {
	/// Whether the deletion was successful.
	pub success: bool,
}

/**
	Tells the client to open a connection to another server instance.

	### Trigger
	The server can send this at any time, but typically does when a launchpad or command is used to go to another world. Other reasons can include the instance shutting down, or exceeding its player limit.

	### Response
	Close the connection after the connection to the other instance has been established.
*/
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct TransferToWorld {
	/// The host to connect to.
	pub redirect_ip: LuString33,
	/// The port to connect to.
	pub redirect_port: u16,
	/// If this is `true`, the original LU client displays a "Mythran dimensional shift succeeded" announcement.
	pub is_maintenance_transfer: bool,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct BlueprintLoadItemResponse {
	pub success: bool,
	pub item_id: ObjId,
	pub dest_item_id: ObjId,
}

/**
	Informs the client that another player has asked them to be their friend.

	### Trigger
	Receipt of `ChatMessage::AddFriendRequest` (todo). Note that friend requests should be supported even if the recipient is on another instance, so a relay infrastructure like a chat server is necessary and needs to be accounted for.

	### Handling
	Display a dialog to the player asking them whether to accept or deny the request.

	### Response
	Respond with [`AddFriendResponse`](crate::chat::server::AddFriendResponse) once the user has made their choice.
*/
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct AddFriendRequest {
	/// Name of the requesting character.
	pub sender_name: LuWString33,
	/// Whether the request is asking to be best friends instead of just normal friends.
	pub is_best_friend_request: bool,
}

/**
	Informs the client that another player has asked them to be their friend.

	### Trigger
	Receipt of `ChatMessage::TeamInvite` (todo). Note that team invites should be supported even if the recipient is on another instance, so a relay infrastructure like a chat server is necessary and needs to be accounted for.

	### Handling
	Display a dialog to the player asking them whether to accept or deny the request.

	### Response
	Respond with [`TeamInviteResponse`](crate::chat::server::TeamInviteResponse) once the user has made their choice.
*/
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct TeamInvite {
	/// Name of the requesting character.
	pub sender_name: LuWString33,
	/// Object ID of the requesting character.
	pub sender_id: ObjId,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct MinimumChatModeResponse {
	pub chat_mode: u8, // todo: type?
	pub chat_channel: u8, // todo: type?
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct MinimumChatModeResponsePrivate {
	pub chat_mode: u8, // todo: type?
	pub chat_channel: u8, // todo: type?
	pub recipient_name: LuWString33,
	pub recipient_gm_level: u8,
}

/**
	Notifies the client that its free trial status has changed.

	### Trigger
	Sent by the server when the status changes.

	### Handling
	Display appropriate UI, celebration, etc.

	### Response
	None.
*/
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct UpdateFreeTrialStatus {
	/// Whether the player is on free trial.
	pub is_free_trial: bool,
}
