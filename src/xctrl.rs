extern crate hex;

pub struct XctrlMeter {
  pub id: u8,
  pub level: u8
}

impl XctrlMeter {
  pub fn as_bytes(&self) -> [u8; 4] {
    let normalised_level: u8 = (self.level / 2) + (self.id * 0x10);
    return [0xf0, 0xd0, normalised_level, 0xf7];
  }

  pub fn as_str(&self) -> String {
    return hex::encode(self.as_bytes());
  }
}

pub struct XctrlFader {
  pub id: u8,
  pub level: u16
}

impl XctrlFader {
  pub fn as_bytes(&self) -> [u8; 5] {
    let upper: u8 = (self.level & 0x00ff) as u8;
    let lower: u8 = (self.level >> 8) as u8;
    return [0xf0, 0xe0 + self.id, upper, lower, 0xf7];
  }

  pub fn as_str(&self) -> String {
    return hex::encode(self.as_bytes());
  }
}

#[derive(Copy, Clone)]
pub enum XctrlButtonType {
  Rec = 0x00, // 8
  Solo = 0x08, // 8
  Mute = 0x10, // 8
  Select = 0x18, // 8
  Encoder = 0x20, // 8
  Track = 0x28,
  Send = 0x29,
  Pan = 0x2a,
  PlugIn = 0x2b,
  Eq = 0x2c,
  Inst = 0x2d,
  FaderBank = 0x2e, // 2
  ChannelBank = 0x30, // 2
  Flip = 0x32,
  GlobalView = 0x33,
  Display = 0x34,
  Reserved = 0x35,
  Function = 0x36, // 8
  MidiTracks = 0x3e,
  Inputs = 0x3f,
  AudioTracks = 0x40,
  AudioInst = 0x41,
  Aux = 0x42,
  Buses = 0x43,
  Outputs = 0x44,
  User = 0x45,
  Shift = 0x46,
  Option = 0x47,
  Control = 0x48,
  Alt = 0x49,
  ReadOff = 0x4a,
  Write = 0x4b,
  Trim = 0x4c,
  Touch = 0x4d,
  Latch = 0x4e,
  Group = 0x4f,
  Save = 0x50,
  Undo = 0x51,
  Cancel = 0x52,
  Enter = 0x53
}

pub struct XctrlButton {
  pub id: u8,
  pub state: u8
}

impl XctrlButton {
  pub fn as_bytes(&self) -> [u8; 5] {
    return [0xf0, 0x90, self.id, self.state, 0xf7];
  }

  pub fn as_str(&self) -> String {
    return hex::encode(self.as_bytes());
  }
}

pub enum XctrlDisplayColor {
  Off = 0x00,
  Red = 0x01,
  Green = 0x02,
  Yellow = 0x03,
  Blue = 0x04,
  Pink = 0x05,
  Cyan = 0x06,
  White = 0x07,
  RedInv = 0x41,
  GreenInv = 0x42,
  YellowInv = 0x43,
  BlueInv = 0x44,
  PinkInv = 0x45,
  CyanInv = 0x46,
  WhiteInv = 0x47
}

pub struct XctrlDisplay {
  pub id: u8,
  pub color: u8,
  pub top_text: [u8; 7],
  pub bottom_text: [u8; 7]
}

impl XctrlDisplay {
  pub fn new(id: u8, color: XctrlDisplayColor, top_text: &[u8], bottom_text: &[u8]) -> XctrlDisplay {
      let mut top: [u8; 7] = [0; 7];
      let mut bottom: [u8; 7] = [0; 7];

      for i in 0..7 {
          top[i] = top_text[i];
          bottom[i] = bottom_text[i];
      }

      return XctrlDisplay {
          id: id,
          color: color as u8,
          top_text: top,
          bottom_text: bottom
      }
  }

  pub fn as_bytes(&self) -> [u8; 22] {
      let mut out = [0; 22];
      out[0] = 0xf0;
      out[3] = 0x66;
      out[4] = 0x58;
      out[5] = self.id + 0x20;
      out[6] = self.color;
      out[21] = 0xf7;

      for i in 0..7 {
          out[i + 7] = self.top_text[i];
          out[i + 14] = self.bottom_text[i];
      }

      return out;
  }

  pub fn as_str(&self) -> String {
    return hex::encode(self.as_bytes());
  }

}

pub struct XctrlState {
    pub displays: [XctrlDisplay; 8],
    pub meters: [XctrlMeter; 8],
    pub faders: [XctrlFader; 9],
    pub buttons: [XctrlButton; 84]
}

impl XctrlState {
  pub fn new() -> Self {
    return XctrlState {
      displays: [
        XctrlDisplay::new(0, XctrlDisplayColor::White, &[0; 7], &[0; 7]),
        XctrlDisplay::new(1, XctrlDisplayColor::White, &[0; 7], &[0; 7] ),
        XctrlDisplay::new(2, XctrlDisplayColor::White, &[0; 7], &[0; 7] ),
        XctrlDisplay::new(3, XctrlDisplayColor::White, &[0; 7], &[0; 7] ),
        XctrlDisplay::new(4, XctrlDisplayColor::White, &[0; 7], &[0; 7] ),
        XctrlDisplay::new(5, XctrlDisplayColor::White, &[0; 7], &[0; 7] ),
        XctrlDisplay::new(6, XctrlDisplayColor::White, &[0; 7], &[0; 7] ),
        XctrlDisplay::new(7, XctrlDisplayColor::White, &[0; 7], &[0; 7] ),
      ],
      meters: [
        XctrlMeter { id: 0, level: 0 },
        XctrlMeter { id: 1, level: 0 },
        XctrlMeter { id: 2, level: 0 },
        XctrlMeter { id: 3, level: 0 },
        XctrlMeter { id: 4, level: 0 },
        XctrlMeter { id: 5, level: 0 },
        XctrlMeter { id: 6, level: 0 },
        XctrlMeter { id: 7, level: 0 },
      ],
      faders: [
        XctrlFader { id: 0, level: 0 },
        XctrlFader { id: 1, level: 0 },
        XctrlFader { id: 2, level: 0 },
        XctrlFader { id: 3, level: 0 },
        XctrlFader { id: 4, level: 0 },
        XctrlFader { id: 5, level: 0 },
        XctrlFader { id: 6, level: 0 },
        XctrlFader { id: 7, level: 0 },
        XctrlFader { id: 8, level: 0 },
      ],
      buttons: [
        XctrlButton { id: XctrlButtonType::Rec as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::Rec as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::Rec as u8 + 2, state: 0 },
        XctrlButton { id: XctrlButtonType::Rec as u8 + 3, state: 0 },
        XctrlButton { id: XctrlButtonType::Rec as u8 + 4, state: 0 },
        XctrlButton { id: XctrlButtonType::Rec as u8 + 5, state: 0 },
        XctrlButton { id: XctrlButtonType::Rec as u8 + 6, state: 0 },
        XctrlButton { id: XctrlButtonType::Rec as u8 + 7, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 2, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 3, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 4, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 5, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 6, state: 0 },
        XctrlButton { id: XctrlButtonType::Solo as u8 + 7, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 2, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 3, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 4, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 5, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 6, state: 0 },
        XctrlButton { id: XctrlButtonType::Mute as u8 + 7, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 2, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 3, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 4, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 5, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 6, state: 0 },
        XctrlButton { id: XctrlButtonType::Select as u8 + 7, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 2, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 3, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 4, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 5, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 6, state: 0 },
        XctrlButton { id: XctrlButtonType::Encoder as u8 + 7, state: 0 },
        XctrlButton { id: XctrlButtonType::Track as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Send as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Pan as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::PlugIn as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Eq as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Inst as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::FaderBank as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::FaderBank as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::ChannelBank as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::ChannelBank as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::Flip as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::GlobalView as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Display as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Reserved as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 0, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 1, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 2, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 3, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 4, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 5, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 6, state: 0 },
        XctrlButton { id: XctrlButtonType::Function as u8 + 7, state: 0 },
        XctrlButton { id: XctrlButtonType::MidiTracks as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Inputs as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::AudioTracks as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::AudioInst as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Aux as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Buses as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Outputs as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::User as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Shift as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Option as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Control as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Alt as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::ReadOff as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Write as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Trim as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Touch as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Latch as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Group as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Save as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Undo as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Cancel as u8, state: 0 },
        XctrlButton { id: XctrlButtonType::Enter as u8, state: 0 },
      ]
    };
  }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum XctrlInterface {
  Encoder = 0xb,
  Button = 0x9,
  Fader = 0xe,
  Unknown = 0x00
}

impl XctrlInterface {
  pub fn from(val: u8) -> Self {
      match val {
          0xb => return XctrlInterface::Encoder,
          0x9 => return XctrlInterface::Button,
          0xe => return XctrlInterface::Fader,
          _ => return XctrlInterface::Unknown,
      };
  }
}


#[derive(Clone, Copy)]
pub struct XctrlStateUpdate {
  pub interface_type: XctrlInterface,
  pub id: u8,
  pub value: u16,
  pub raw_message: [u8; 3]
}
