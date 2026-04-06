use macros::num_enum;

use crate::prelude::*;
use crate::util::init::num_enum_from;
use crate::wad::elements::game_object::event::EventSubtype;

/// The subtype for an [`OtherEvent`].
#[num_enum(u32)]
pub enum Other {
    /// The outside room event.
    OutsideRoom = 0,

    /// The intersect boundary event.
    IntersectBoundary = 1,

    /// The game start event.
    GameStart = 2,

    /// The game end event.
    GameEnd = 3,

    /// The room start event.
    RoomStart = 4,

    /// The room end event.
    RoomEnd = 5,

    /// The "No More Lives" event. Only used in GameMaker: Studio 1 and earlier.
    NoMoreLives = 6,

    /// The animation end event.
    AnimationEnd = 7,

    /// The path ended event.
    EndOfPath = 8,

    /// The "No More Health" event. Only used in GameMaker: Studio 1 and
    /// earlier.
    NoMoreHealth = 9,

    // USER EVENTS
    /// The User 0 event.
    User0 = 10,

    /// The User 1 event.
    User1 = 11,

    /// The User 2 event.
    User2 = 12,

    /// The User 3 event.
    User3 = 13,

    /// The User 4 event.
    User4 = 14,

    /// The User 5 event.
    User5 = 15,

    /// The User 6 event.
    User6 = 16,

    /// The User 7 event.
    User7 = 17,

    /// The User 8 event.
    User8 = 18,

    /// The User 9 event.
    User9 = 19,

    /// The User 10 event.
    User10 = 20,

    /// The User 11 event.
    User11 = 21,

    /// The User 12 event.
    User12 = 22,

    /// The User 13 event.
    User13 = 23,

    /// The User 14 event.
    User14 = 24,

    /// The User 15 event.
    User15 = 25,

    /// The User 16 event.
    User16 = 26,

    // VIEW EVENTS
    /// The Outside View 0 event.
    OutsideView0 = 40,

    /// The Outside View 1 event.
    OutsideView1 = 41,

    /// The Outside View 2 event.
    OutsideView2 = 42,

    /// The Outside View 3 event.
    OutsideView3 = 43,

    /// The Outside View 4 event.
    OutsideView4 = 44,

    /// The Outside View 5 event.
    OutsideView5 = 45,

    /// The Outside View 6 event.
    OutsideView6 = 46,

    /// The Outside View 7 event.
    OutsideView7 = 47,

    /// The Intersect View 0 Boundary event.
    BoundaryView0 = 50,

    /// The Intersect View 1 Boundary event.
    BoundaryView1 = 51,

    /// The Intersect View 2 Boundary event.
    BoundaryView2 = 52,

    /// The Intersect View 3 Boundary event.
    BoundaryView3 = 53,

    /// The Intersect View 4 Boundary event.
    BoundaryView4 = 54,

    /// The Intersect View 5 Boundary event.
    BoundaryView5 = 55,

    /// The Intersect View 6 Boundary event.
    BoundaryView6 = 56,

    /// The Intersect View 7 Boundary event.
    BoundaryView7 = 57,

    /// The animation Update event for Skeletal Animation functions.
    AnimationUpdate = 58,

    /// The animation event for Skeletal Animation functions.
    AnimationEvent = 59,

    // ASYNC EVENTS
    /// The async image loaded event.
    AsyncImageLoaded = 60,

    /// The async sound loaded event.
    AsyncSoundLoaded = 61,

    /// The async http event.
    AsyncHTTP = 62,

    /// The async dialog event.
    AsyncDialog = 63,

    /// The async in-app purchase event.
    AsyncIAP = 66,

    /// The async cloud event.
    AsyncCloud = 67,

    /// The async networking event.
    AsyncNetworking = 68,

    /// The async Steam event.
    AsyncSteam = 69,

    /// The async social event.
    AsyncSocial = 70,

    /// The async push notification event.
    AsyncPushNotification = 71,

    /// The async save/load event.
    AsyncSaveAndLoad = 72,

    /// The async audio recording event.
    AsyncAudioRecording = 73,

    /// The async audio playback event.
    AsyncAudioPlayback = 74,

    /// The async system event.
    AsyncSystem = 75,
}

impl EventSubtype for Other {
    fn parse(subtype: u32) -> Result<Self> {
        num_enum_from(subtype)
    }

    fn build(self) -> u32 {
        self.into()
    }
}

impl Other {
    /// Creates a new `Other::UserXX` based on the given ID.
    /// This function will fail for IDs greater than 16.
    pub fn from_user_id(id: u32) -> Result<Self> {
        Ok(match id {
            0 => Self::User0,
            1 => Self::User1,
            2 => Self::User2,
            3 => Self::User3,
            4 => Self::User4,
            5 => Self::User5,
            6 => Self::User6,
            7 => Self::User7,
            8 => Self::User8,
            9 => Self::User9,
            10 => Self::User10,
            11 => Self::User11,
            12 => Self::User12,
            13 => Self::User13,
            14 => Self::User14,
            15 => Self::User15,
            16 => Self::User16,
            _ => bail!("User Event ID {id} is too high; maximum is 16"),
        })
    }

    /// The User Event ID of this event, if this is a `Other::UserXX` event.
    #[must_use]
    pub const fn user_id(self) -> Option<u32> {
        Some(match self {
            Self::User0 => 0,
            Self::User1 => 1,
            Self::User2 => 2,
            Self::User3 => 3,
            Self::User4 => 4,
            Self::User5 => 5,
            Self::User6 => 6,
            Self::User7 => 7,
            Self::User8 => 8,
            Self::User9 => 9,
            Self::User10 => 10,
            Self::User11 => 11,
            Self::User12 => 12,
            Self::User13 => 13,
            Self::User14 => 14,
            Self::User15 => 15,
            Self::User16 => 16,
            _ => return None,
        })
    }
}
