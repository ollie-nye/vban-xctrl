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
  Rec = 0,
  Solo = 8,
  Mute = 16,
  Select = 24,
  FaderBank = 46,
  ChannelBank = 48
}

pub struct XctrlButton {
  pub button_type: XctrlButtonType,
  pub id: u8,
  pub state: u8
}

impl XctrlButton {
  pub fn as_bytes(&self) -> [u8; 5] {
    let button_id: u8 = self.id + self.button_type as u8;
    return [0xf0, 0x90, button_id, self.state, 0xf7];
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
    pub recs: [XctrlButton; 8],
    pub solos: [XctrlButton; 8],
    pub mutes: [XctrlButton; 8],
    pub selects: [XctrlButton; 8]
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
      recs: [
        XctrlButton { button_type: XctrlButtonType::Rec, id: 0, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Rec, id: 1, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Rec, id: 2, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Rec, id: 3, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Rec, id: 4, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Rec, id: 5, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Rec, id: 6, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Rec, id: 7, state: 0 },
      ],
      solos: [
        XctrlButton { button_type: XctrlButtonType::Solo, id: 0, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Solo, id: 1, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Solo, id: 2, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Solo, id: 3, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Solo, id: 4, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Solo, id: 5, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Solo, id: 6, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Solo, id: 7, state: 0 },
      ],
      mutes: [
        XctrlButton { button_type: XctrlButtonType::Mute, id: 0, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Mute, id: 1, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Mute, id: 2, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Mute, id: 3, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Mute, id: 4, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Mute, id: 5, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Mute, id: 6, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Mute, id: 7, state: 0 },
      ],
      selects: [
        XctrlButton { button_type: XctrlButtonType::Select, id: 0, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Select, id: 1, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Select, id: 2, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Select, id: 3, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Select, id: 4, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Select, id: 5, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Select, id: 6, state: 0 },
        XctrlButton { button_type: XctrlButtonType::Select, id: 7, state: 0 },
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



  // Fader = 1,
  // RecButton = 2,
  // SoloButton = 3,
  // MuteButton = 4,
  // SelectButton = 5,
  // FaderBankButton = 6,
  // ChannelBankButton = 7,
  // FunctionButton = 8,
  // AutomationButton = 9
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
