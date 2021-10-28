use crate::{
    res::{GenericResultCode, ResultCode},
    time::SystemTimestamp,
};
use alloc::str;
use core::convert::TryFrom;
use num_enum::IntoPrimitive;
use safe_transmute::TriviallyTransmutable;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct NatProperties {
    unk1: u8,
    unk2: u8,
    unk3: u32,
}

// This is safe because all fields in the struct can function with any value
unsafe impl TriviallyTransmutable for NatProperties {}

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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct ScrambledFriendCode {
    pub friend_code: u64,
    pub xor_key: u16,
    pub unk: u16,
}

// This is safe because all fields in the struct can function with any value
unsafe impl TriviallyTransmutable for ScrambledFriendCode {}

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

#[derive(Clone, Copy, Debug, PartialEq, Default)]
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

// This is safe because all fields in the struct can function with any value
unsafe impl TriviallyTransmutable for FriendPresence {}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct ExpandedFriendPresence {
    pub join_availability_flag: u32,
    pub match_make_system_type: u32,
    pub join_game_id: u32,
    pub join_game_mode: u32,
    pub owner_principal_id: u32,
    pub join_group_id: u32,
    pub application_arg: [u8; 20],
    pub game_description: [u16; 128],
}

// This is safe because all fields in the struct can function with any value
unsafe impl TriviallyTransmutable for ExpandedFriendPresence {}

impl Default for ExpandedFriendPresence {
    fn default() -> Self {
        Self {
            join_availability_flag: 0,
            match_make_system_type: 0,
            join_game_id: 0,
            join_game_mode: 0,
            owner_principal_id: 0,
            join_group_id: 0,
            application_arg: [0; 20],
            game_description: [0; 128],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct FriendComment([u16; 17]);

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

        FriendComment(result)
    }
}

// This is safe because all fields in the struct can function with any value.
// At some point it may be worth having a validator to ensure a valid value
// is sent to another process.
unsafe impl TriviallyTransmutable for FriendComment {}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct ScreenName([u16; 11]);

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

        ScreenName(result)
    }
}

// This is safe because all fields in the struct can function with any value.
// At some point it may be worth having a validator to ensure a valid value
// is sent to another process.
unsafe impl TriviallyTransmutable for ScreenName {}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mii([u8; 96]);

impl Default for Mii {
    fn default() -> Self {
        Mii([0; 96])
    }
}

impl Mii {
    pub fn new(bytes: [u8; 96]) -> Self {
        Mii(bytes)
    }

    pub fn as_bytes(&self) -> [u8; 96] {
        self.0
    }
}

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for Mii {}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum CharacterSet {
    JapanUsaEuropeAustralia = 0,
    Korea = 1,
    China = 2,
    Taiwan = 3,
    None = 0xff,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrivialCharacterSet(u8);

impl TrivialCharacterSet {
    pub fn get_character_set(&self) -> CharacterSet {
        match self.0 {
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
        TrivialCharacterSet(CharacterSet::None as u8)
    }
}

impl From<CharacterSet> for TrivialCharacterSet {
    fn from(character_set: CharacterSet) -> Self {
        TrivialCharacterSet(character_set as u8)
    }
}

// This is safe because all fields in the struct can function with any value.
// At some point it may be worth having a validator to ensure a valid value
// is sent to another process.
unsafe impl TriviallyTransmutable for TrivialCharacterSet {}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct FriendKey {
    pub principal_id: u32,
    pub padding: u32,
    pub local_friend_code: u64,
}

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for FriendKey {}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct GameKey {
    pub title_id: u64,
    pub version: u32,
    pub unk: u32,
}

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for GameKey {}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct FriendProfile {
    pub region: u8,
    pub country: u8,
    pub area: u8,
    pub language: u8,
    pub platform: u8,
    pub padding: [u8; 3],
}

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for FriendProfile {}

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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct SomeFriendThing {
    pub friend_profile: FriendProfile,
    pub favorite_game: GameKey,
    pub unk2: u32,
    pub comment: FriendComment,
    pub unk3: u16, // padding?
    pub last_online: SystemTimestamp,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

// This is safe because all fields in the struct can function with any value.
// At some point it may be worth having a validator to ensure a valid value
// is sent to another process.
unsafe impl TriviallyTransmutable for FriendInfo {}

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
            _ => Err(GenericResultCode::InvalidValue.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
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

// This is safe because all fields in the struct can function with any value.
// At some point it may be worth having a validator to ensure a valid value
// is sent to another process.
unsafe impl TriviallyTransmutable for NotificationEvent {}
