use crate::{
    res::{error, ResultCode},
    time::SystemTimestamp,
};
use alloc::{str, vec::Vec};
use core::{
    convert::{TryFrom, TryInto},
    mem,
};
use no_std_io::{
    Cursor, EndianRead, EndianWrite, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};
use num_enum::IntoPrimitive;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct NatProperties {
    unk1: u8,
    unk2: u8,
    unk3: u32,
}

impl NatProperties {
    pub fn new(unk1: u8, unk2: u8, unk3: u32) -> Self {
        Self { unk1, unk2, unk3 }
    }

    pub fn get_unk1(&self) -> u8 {
        if self.unk1 < 3 {
            self.unk1
        } else {
            0
        }
    }

    pub fn get_unk2(&self) -> u8 {
        if self.unk2 < 3 {
            self.unk2
        } else {
            0
        }
    }

    pub fn get_unk3(&self) -> u32 {
        self.unk3
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct ScrambledFriendCode {
    pub friend_code: u64,
    pub xor_key: u16,
    pub unk: u16,
}

impl ScrambledFriendCode {
    pub fn new(friend_code: u64, xor_key: u16) -> Self {
        Self {
            friend_code: Self::xor_friend_code(friend_code, xor_key),
            xor_key,
            unk: 0,
        }
    }

    fn expand_xor_key(xor_key: u16) -> u64 {
        let _xor_key: u64 = xor_key as u64;
        _xor_key | (_xor_key << 16) | (_xor_key << 32) | (_xor_key << 48)
    }

    fn xor_friend_code(friend_code: u64, xor_key: u16) -> u64 {
        friend_code ^ Self::expand_xor_key(xor_key)
    }

    pub fn get_unscrambled_friend_code(&self) -> u64 {
        Self::xor_friend_code(self.friend_code, self.xor_key)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct FriendPresence {
    pub join_availability_flag: u32,
    pub match_make_system_type: u32,
    pub join_game_id: u32,
    pub join_game_mode: u32,
    pub owner_principal_id: u32,
    pub join_group_id: u32,
    pub application_arg: [u8; 20],
    pub unk: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameDescription {
    raw: [u16; 128],
}

impl Default for GameDescription {
    fn default() -> Self {
        Self { raw: [0; 128] }
    }
}

impl EndianRead for GameDescription {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        let read_size = mem::size_of::<GameDescription>();
        let raw = StreamContainer::new(bytes)
            .into_le_iter()
            .take(128)
            .collect::<Vec<u16>>()
            .try_into()
            .map_err(|_| no_std_io::Error::InvalidSize {
                wanted_size: read_size,
                offset: 0,
                data_len: bytes.len(),
            })?;
        Ok(ReadOutput::new(Self { raw }, read_size))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        unimplemented!()
    }
}

impl EndianWrite for GameDescription {
    fn get_size(&self) -> usize {
        mem::size_of::<GameDescription>()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        let mut stream = StreamContainer::new(dst);

        for short in self.raw.iter() {
            stream.write_stream_le(short)?;
        }

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        unimplemented!()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EndianRead, EndianWrite)]
#[repr(C)]
pub struct ExpandedFriendPresence {
    pub join_availability_flag: u32,
    pub match_make_system_type: u32,
    pub join_game_id: u32,
    pub join_game_mode: u32,
    pub owner_principal_id: u32,
    pub join_group_id: u32,
    pub application_arg: [u8; 20],
    pub game_description: GameDescription,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct FriendComment {
    raw: [u16; 17],
}

impl FriendComment {
    pub fn new(raw: [u16; 17]) -> Self {
        Self { raw }
    }
}

impl EndianRead for FriendComment {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        let read_size = mem::size_of::<FriendComment>();
        let raw = StreamContainer::new(bytes)
            .into_le_iter()
            .take(17)
            .collect::<Vec<u16>>()
            .try_into()
            .map_err(|_| no_std_io::Error::InvalidSize {
                wanted_size: read_size,
                offset: 0,
                data_len: bytes.len(),
            })?;
        Ok(ReadOutput::new(Self { raw }, read_size))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        unimplemented!()
    }
}

impl EndianWrite for FriendComment {
    fn get_size(&self) -> usize {
        mem::size_of::<FriendComment>()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        let mut stream = StreamContainer::new(dst);

        for short in self.raw.iter() {
            stream.write_stream_le(short)?;
        }

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        unimplemented!()
    }
}

impl From<&str> for FriendComment {
    fn from(string: &str) -> Self {
        let mut result: [u16; 17] = [0; 17];
        string
            .encode_utf16()
            .take(16)
            .zip(result.iter_mut())
            .for_each(|(short, result_short)| {
                *result_short = short;
            });

        Self { raw: result }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ScreenName {
    raw: [u16; 11],
}

impl ScreenName {
    pub fn new(raw: [u16; 11]) -> Self {
        Self { raw }
    }
}

impl EndianRead for ScreenName {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        let read_size = mem::size_of::<ScreenName>();
        let raw = StreamContainer::new(bytes)
            .into_le_iter()
            .take(11)
            .collect::<Vec<u16>>()
            .try_into()
            .map_err(|_| no_std_io::Error::InvalidSize {
                wanted_size: read_size,
                offset: 0,
                data_len: bytes.len(),
            })?;
        Ok(ReadOutput::new(Self { raw }, read_size))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        unimplemented!()
    }
}

impl EndianWrite for ScreenName {
    fn get_size(&self) -> usize {
        mem::size_of::<ScreenName>()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        let mut stream = StreamContainer::new(dst);

        for short in self.raw.iter() {
            stream.write_stream_le(short)?;
        }

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        unimplemented!()
    }
}

impl From<&str> for ScreenName {
    fn from(string: &str) -> Self {
        let mut result: [u16; 11] = [0; 11];
        string
            .encode_utf16()
            .take(10)
            .zip(result.iter_mut())
            .for_each(|(short, result_short)| {
                *result_short = short;
            });

        Self { raw: result }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EndianRead, EndianWrite)]
#[repr(C)]
pub struct Mii {
    raw: [u8; 96],
}

impl Default for Mii {
    fn default() -> Self {
        Mii { raw: [0; 96] }
    }
}

impl Mii {
    pub fn new(bytes: [u8; 96]) -> Self {
        Self { raw: bytes }
    }

    pub fn as_bytes(&self) -> [u8; 96] {
        self.raw
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CharacterSet {
    JapanUsaEuropeAustralia = 0,
    Korea = 1,
    China = 2,
    Taiwan = 3,
    None = 0xff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EndianRead, EndianWrite)]
pub struct TrivialCharacterSet {
    raw: u8,
}

impl TrivialCharacterSet {
    pub fn get_character_set(&self) -> CharacterSet {
        match self.raw {
            0 => CharacterSet::JapanUsaEuropeAustralia,
            1 => CharacterSet::Korea,
            2 => CharacterSet::China,
            3 => CharacterSet::Taiwan,
            _ => CharacterSet::None,
        }
    }
}

impl Default for TrivialCharacterSet {
    fn default() -> Self {
        Self {
            raw: CharacterSet::None as u8,
        }
    }
}

impl From<CharacterSet> for TrivialCharacterSet {
    fn from(character_set: CharacterSet) -> Self {
        Self {
            raw: character_set as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct FriendKey {
    pub principal_id: u32,
    pub padding: u32,
    pub local_friend_code: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct GameKey {
    pub title_id: u64,
    pub version: u32,
    pub unk: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct FriendProfile {
    pub region: u8,
    pub country: u8,
    pub area: u8,
    pub language: u8,
    pub platform: u8,
    pub padding: [u8; 3],
}

impl From<FriendProfile> for [u8; 5] {
    fn from(profile: FriendProfile) -> Self {
        [
            profile.region,
            profile.country,
            profile.area,
            profile.language,
            profile.platform,
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EndianRead, EndianWrite)]
#[repr(C)]
pub struct SomeFriendThing {
    pub friend_profile: FriendProfile,
    pub favorite_game: GameKey,
    pub unk2: u32,
    pub comment: FriendComment,
    pub unk3: u16, // padding?
    pub last_online: SystemTimestamp,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, EndianRead, EndianWrite)]
#[repr(C)]
pub struct FriendInfo {
    pub friend_key: FriendKey,
    pub some_timestamp: SystemTimestamp,
    pub friend_relationship: u8,
    pub unk1: [u8; 3], // padding?
    pub unk2: u32,
    pub unk3: SomeFriendThing,
    pub screen_name: ScreenName,
    pub character_set: TrivialCharacterSet,
    pub unk4: u8, // padding?
    pub mii: Mii,
}

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum NotificationType {
    UserWentOnline = 1,
    UserWentOffline = 2,
    FriendWentOnline = 3,
    FriendUpdatedPresence = 4,
    FriendUpdatedMii = 5,
    FriendUpdatedProfile = 6,
    FriendWentOffline = 7,
    FriendRegisteredUser = 8,
    FriendSentInvitation = 9,
}

impl TryFrom<u8> for NotificationType {
    type Error = ResultCode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::UserWentOnline),
            2 => Ok(Self::UserWentOffline),
            3 => Ok(Self::FriendWentOnline),
            4 => Ok(Self::FriendUpdatedPresence),
            5 => Ok(Self::FriendUpdatedMii),
            6 => Ok(Self::FriendUpdatedProfile),
            7 => Ok(Self::FriendWentOffline),
            8 => Ok(Self::FriendRegisteredUser),
            9 => Ok(Self::FriendSentInvitation),
            _ => Err(error::invalid_enum_value()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct NotificationEvent {
    notification_type: u8,
    padding: [u8; 7],
    friend_key: FriendKey,
}

impl NotificationEvent {
    pub fn new(notification_type: NotificationType, friend_key: FriendKey) -> Self {
        Self {
            notification_type: notification_type.into(),
            padding: [0; 7],
            friend_key,
        }
    }

    /// Returns the notification type if it's a recognized notification type, otherwise returns None.
    pub fn get_known_notification_type(&self) -> Option<NotificationType> {
        NotificationType::try_from(self.notification_type).ok()
    }
}
