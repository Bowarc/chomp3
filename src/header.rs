use crate::{constants::*, utils::*};
use bitvec::prelude::*;

#[derive(Debug)]

pub struct Header {
    // Sync (12 bits)
    // This is the synchronization word described above. All 12 bits must be set, i.e.
    // ‘1111 1111 1111’.
    pub sync: BitVec<u8>,

    // Id (1 bit)
    // Specifies the MPEG version. A set bit means that the frame is encoded with the MPEG-1
    // standard, if not MPEG-2 is used

    // Some add-on standards only use 11 bits for the sync word in order to dedicate 2 bits for the id.
    // In this case Table 5.1 is applied.
    pub id: MPEG_Version,

    // Layer (2 bits)
    // See Table 5.2
    pub layer: Layer,

    // Protection Bit (1 bit)
    // If the protection bit is set, the CRC field will be used.
    protection_bit: Protected,

    // Bitrate (4 bits)
    // These four bits tells the decoder in what bitrate the frame is encoded. This value will be the
    // same for all frames if the stream is encoded using CBR. Table 5.3 shows the defined bit
    // values.
    bitrate: Bitrate,

    // Frequency (2 bits)
    // 2 bits that give the sampling frequency, see Table 5.4.
    frequency: Frequency,

    // Padding bit (1 bit)
    // An encoded stream with bitrate 128 kbit/s and sampling frequency of 44100 Hz will create
    // frames of size 417 bytes. To exactly fit the bitrate some of these frames will have to be 418
    // bytes. These frames set the padding bit.
    padding: BitVec<u8>,

    // Private bit (1 bit)
    // One bit for application-specific triggers.
    private_bit: BitVec<u8>,

    // Mode (2 bits)
    // Specifies what channel mode is used according to Table 5.5.
    mode: Mode,

    // Copyright Bit (1 bit)
    // If this bit is set it means that it is illegal to copy the contents.
    copyright_bit: Copyright,

    // Home (Original Bit) (1 bit)
    // The original bit indicates, if it is set, that the frame is located on its original media.
    home: Home,

    // Emphasis (2 bits)
    // The emphasis indication is used to tell the decoder that the file must be de-emphasized, i.e.
    // the decoder must 're-equalize' the sound after a Dolby- like noise supression. It is rarely used.
    emphasis: Emphasis,
}

#[derive(Debug)]
pub enum MPEG_Version {
    Reserved,
    One,
    Two,
    TwoPointFive,
}

#[derive(Debug)]
pub enum Layer {
    Reserved,
    Three,
    Two,
    One,
}

#[derive(Debug)]
pub enum Protected {
    Yes,
    No,
}

#[derive(Debug)]
pub struct Bitrate(usize);

#[derive(Debug)]
// In Hz
pub struct Frequency(usize);

#[derive(Debug)]
pub enum Mode {
    Stereo,
    JointStereo,
    DualChannel,
    SingleChannel,
}

#[derive(Debug)]
pub enum Copyright {
    On,
    Off,
}

#[derive(Debug)]
pub enum Home {
    On,
    Off,
}

#[derive(Debug)]
pub enum Emphasis {
    On,
    Off,
}

#[derive(Debug)]
pub struct RawHeader {
    pub sync: BitVec<u8>,
    pub id: BitVec<u8>,
    pub layer: BitVec<u8>,
    pub protection_bit: BitVec<u8>,
    pub bitrate: BitVec<u8>,
    pub frequency: BitVec<u8>,
    pub padding_bit: BitVec<u8>,
    pub private_bit: BitVec<u8>,
    pub mode: BitVec<u8>,
    pub mode_extension: BitVec<u8>,
    pub copyright_bit: BitVec<u8>,
    pub home: BitVec<u8>,
    pub emphasis: BitVec<u8>,
}

impl RawHeader {
    pub fn new(array: &mut BitSlice<u8>) -> Self {
        Self {
            sync: access(array, SYNC_SIZE),
            id: access(array, ID_SIZE),
            layer: access(array, LAYER_SIZE),
            protection_bit: access(array, PROTECTION_BIT_SIZE),
            bitrate: access(array, BITRATE_SIZE),
            frequency: access(array, FREQUENCY_SIZE),
            padding_bit: access(array, PADDING_BIT_SIZE),
            private_bit: access(array, PRIVATE_BIT_SIZE),
            mode: access(array, MODE_SIZE),
            mode_extension: access(array, MODE_EXTENSION_SIZE),
            copyright_bit: access(array, COPYRIGHT_BIT_SIZE),
            home: access(array, HOME_SIZE),
            emphasis: access(array, EMPHASIS_SIZE),
        }
    }
}

impl From<BitVec<u8>> for MPEG_Version {
    fn from(bits: BitVec<u8>) -> MPEG_Version {
        match bits.len() {
            1 => match bits.into_vec()[..] {
                [0] => MPEG_Version::Two,
                [1] => MPEG_Version::One,
                _ => unreachable!(),
            },
            2 => match bits.into_vec()[..] {
                [0, 0] => MPEG_Version::TwoPointFive,
                [0, 1] => panic!("reserved"),
                [1, 0] => MPEG_Version::Two,
                [1, 1] => MPEG_Version::One,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl From<BitVec<u8>> for Layer {
    fn from(bits: BitVec<u8>) -> Layer {
        assert_eq!(bits.len(), LAYER_SIZE);

        match bits.into_vec()[..] {
            [0, 0] => panic!("reserved"),
            [0, 1] => Layer::Three,
            [1, 0] => Layer::Two,
            [1, 1] => Layer::One,
            _ => unreachable!(),
        }
    }
}

impl From<BitVec<u8>> for Protected {
    fn from(bits: BitVec<u8>) -> Protected {
        assert_eq!(bits.len(), PROTECTION_BIT_SIZE);

        match bits.into_vec()[..] {
            [0] => Protected::No,
            [1] => Protected::Yes,
            _ => unreachable!(),
        }
    }
}

impl Bitrate {
    pub fn from_bitvecu8(bits: BitVec<u8>, version: MPEG_Version, layer: Layer) -> Self {
        assert_eq!(bits.len(), BITRATE_SIZE);
        match bits.into_vec()[..] {
            [0, 0, 0, 0] => unreachable!(),
            [0, 0, 0, 1] => match (version, layer) {
                (MPEG_Version::One, Layer::One)
                | (MPEG_Version::One, Layer::Two)
                | (MPEG_Version::One, Layer::Three)
                | (MPEG_Version::Two, Layer::One)
                | (MPEG_Version::Two, Layer::Two) => Bitrate(32),
                (MPEG_Version::Two, Layer::Three) => Bitrate(8),

                _ => unreachable!(),
            },
            [0, 0, 1, 0] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(64),
                (MPEG_Version::One, Layer::Two) => Bitrate(48),
                (MPEG_Version::One, Layer::Three) => Bitrate(40),
                (MPEG_Version::Two, Layer::One) => Bitrate(64),
                (MPEG_Version::Two, Layer::Two) => Bitrate(48),
                (MPEG_Version::Two, Layer::Three) => Bitrate(16),
                _ => unreachable!(),
            },
            [0, 0, 1, 1] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(96),
                (MPEG_Version::One, Layer::Two) => Bitrate(56),
                (MPEG_Version::One, Layer::Three) => Bitrate(48),
                (MPEG_Version::Two, Layer::One) => Bitrate(96),
                (MPEG_Version::Two, Layer::Two) => Bitrate(56),
                (MPEG_Version::Two, Layer::Three) => Bitrate(24),
                _ => unreachable!(),
            },
            [0, 1, 0, 0] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(128),
                (MPEG_Version::One, Layer::Two) => Bitrate(64),
                (MPEG_Version::One, Layer::Three) => Bitrate(56),
                (MPEG_Version::Two, Layer::One) => Bitrate(128),
                (MPEG_Version::Two, Layer::Two) => Bitrate(64),
                (MPEG_Version::Two, Layer::Three) => Bitrate(32),
                _ => unreachable!(),
            },
            [0, 1, 0, 1] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(160),
                (MPEG_Version::One, Layer::Two) => Bitrate(80),
                (MPEG_Version::One, Layer::Three) => Bitrate(64),
                (MPEG_Version::Two, Layer::One) => Bitrate(160),
                (MPEG_Version::Two, Layer::Two) => Bitrate(80),
                (MPEG_Version::Two, Layer::Three) => Bitrate(64),
                _ => unreachable!(),
            },
            [0, 1, 1, 0] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(192),
                (MPEG_Version::One, Layer::Two) => Bitrate(96),
                (MPEG_Version::One, Layer::Three) => Bitrate(80),
                (MPEG_Version::Two, Layer::One) => Bitrate(192),
                (MPEG_Version::Two, Layer::Two) => Bitrate(96),
                (MPEG_Version::Two, Layer::Three) => Bitrate(80),
                _ => unreachable!(),
            },
            [0, 1, 1, 1] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(224),
                (MPEG_Version::One, Layer::Two) => Bitrate(112),
                (MPEG_Version::One, Layer::Three) => Bitrate(96),
                (MPEG_Version::Two, Layer::One) => Bitrate(224),
                (MPEG_Version::Two, Layer::Two) => Bitrate(112),
                (MPEG_Version::Two, Layer::Three) => Bitrate(56),
                _ => unreachable!(),
            },
            [1, 0, 0, 0] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(256),
                (MPEG_Version::One, Layer::Two) => Bitrate(128),
                (MPEG_Version::One, Layer::Three) => Bitrate(112),
                (MPEG_Version::Two, Layer::One) => Bitrate(256),
                (MPEG_Version::Two, Layer::Two) => Bitrate(128),
                (MPEG_Version::Two, Layer::Three) => Bitrate(64),
                _ => unreachable!(),
            },
            [1, 0, 0, 1] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(288),
                (MPEG_Version::One, Layer::Two) => Bitrate(160),
                (MPEG_Version::One, Layer::Three) => Bitrate(128),
                (MPEG_Version::Two, Layer::One) => Bitrate(288),
                (MPEG_Version::Two, Layer::Two) => Bitrate(160),
                (MPEG_Version::Two, Layer::Three) => Bitrate(128),
                _ => unreachable!(),
            },
            [1, 0, 1, 0] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(320),
                (MPEG_Version::One, Layer::Two) => Bitrate(192),
                (MPEG_Version::One, Layer::Three) => Bitrate(160),
                (MPEG_Version::Two, Layer::One) => Bitrate(320),
                (MPEG_Version::Two, Layer::Two) => Bitrate(192),
                (MPEG_Version::Two, Layer::Three) => Bitrate(160),
                _ => unreachable!(),
            },
            [1, 0, 1, 1] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(352),
                (MPEG_Version::One, Layer::Two) => Bitrate(224),
                (MPEG_Version::One, Layer::Three) => Bitrate(192),
                (MPEG_Version::Two, Layer::One) => Bitrate(352),
                (MPEG_Version::Two, Layer::Two) => Bitrate(224),
                (MPEG_Version::Two, Layer::Three) => Bitrate(112),
                _ => unreachable!(),
            },
            [1, 1, 0, 0] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(384),
                (MPEG_Version::One, Layer::Two) => Bitrate(256),
                (MPEG_Version::One, Layer::Three) => Bitrate(224),
                (MPEG_Version::Two, Layer::One) => Bitrate(384),
                (MPEG_Version::Two, Layer::Two) => Bitrate(256),
                (MPEG_Version::Two, Layer::Three) => Bitrate(128),
                _ => unreachable!(),
            },
            [1, 1, 0, 1] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(416),
                (MPEG_Version::One, Layer::Two) => Bitrate(320),
                (MPEG_Version::One, Layer::Three) => Bitrate(256),
                (MPEG_Version::Two, Layer::One) => Bitrate(416),
                (MPEG_Version::Two, Layer::Two) => Bitrate(320),
                (MPEG_Version::Two, Layer::Three) => Bitrate(256),
                _ => unreachable!(),
            },
            [1, 1, 1, 0] => match (version, layer) {
                (MPEG_Version::One, Layer::One) => Bitrate(448),
                (MPEG_Version::One, Layer::Two) => Bitrate(384),
                (MPEG_Version::One, Layer::Three) => Bitrate(320),
                (MPEG_Version::Two, Layer::One) => Bitrate(448),
                (MPEG_Version::Two, Layer::Two) => Bitrate(384),
                (MPEG_Version::Two, Layer::Three) => Bitrate(320),
                _ => unreachable!(),
            },
            [1, 1, 1, 1] => unreachable!(),

            _ => unreachable!(),
        }
    }
}

impl Frequency {
    pub fn from_bitvecu8(bits: BitVec<u8>, version: MPEG_Version) -> Self {
        assert_eq!(bits.len(), FREQUENCY_SIZE);

        match bits.into_vec()[..] {
            [0, 0] => match version {
                MPEG_Version::One => Frequency(44100),
                MPEG_Version::Two => Frequency(22050),
                MPEG_Version::TwoPointFive => Frequency(11025),
                _ => unreachable!(),
            },
            [0, 1] => match version {
                MPEG_Version::One => Frequency(48000),
                MPEG_Version::Two => Frequency(24000),
                MPEG_Version::TwoPointFive => Frequency(12000),
                _ => unreachable!(),
            },
            [1, 0] => match version {
                MPEG_Version::One => Frequency(32000),
                MPEG_Version::Two => Frequency(16000),
                MPEG_Version::TwoPointFive => Frequency(8000),
                _ => unreachable!(),
            },
            [1, 1] => {
                panic!("reserved")
            }
            _ => unreachable!(),
        }
    }
}

impl From<BitVec<u8>> for Mode {
    fn from(bits: BitVec<u8>) -> Mode {
        assert_eq!(bits.len(), MODE_SIZE);

        match bits.into_vec()[..] {
            [0, 0] => Mode::Stereo,
            [0, 1] => Mode::JointStereo,
            [1, 0] => Mode::DualChannel,
            [1, 1] => Mode::SingleChannel,
            _ => unreachable!(),
        }
    }
}

impl From<BitVec<u8>> for Copyright {
    fn from(bits: BitVec<u8>) -> Copyright {
        assert_eq!(bits.len(), COPYRIGHT_BIT_SIZE);

        match bits.into_vec()[..] {
            [0] => Copyright::Off,
            [1] => Copyright::On,

            _ => unreachable!(),
        }
    }
}

impl From<BitVec<u8>> for Home {
    fn from(bits: BitVec<u8>) -> Home {
        assert_eq!(bits.len(), HOME_SIZE);

        match bits.into_vec()[..] {
            [0] => Home::Off,
            [1] => Home::On,

            _ => unreachable!(),
        }
    }
}
impl From<BitVec<u8>> for Emphasis {
    fn from(bits: BitVec<u8>) -> Emphasis {
        assert_eq!(bits.len(), EMPHASIS_SIZE);

        match bits.into_vec()[..] {
            [0] => Emphasis::Off,
            [1] => Emphasis::On,

            _ => unreachable!(),
        }
    }
}
