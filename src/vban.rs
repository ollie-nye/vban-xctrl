use packed_struct::prelude::*;

#[derive(PrimitiveEnum_u8, Debug, Clone, Copy, PartialEq)]
pub enum VBANProtocol {
    Audio = 0x00,
    Serial = 0x20,
    Txt = 0x40,
    Service = 0x60,
    User = 0xE0
}

#[derive(PrimitiveEnum_u8, Debug, Clone, Copy, PartialEq)]
pub enum VoiceMeeterType {
    Standard = 1,
    Banana = 2,
    Potato = 3
}

#[derive(PackedStruct, Debug, Clone, Copy, PartialEq)]
#[packed_struct(endian="lsb", bit_numbering="msb0")]
pub struct VBANHeader {
    pub vban: [u8; 4],
    pub protocol: u8
}

#[derive(PackedStruct, Debug, Clone, Copy, PartialEq)]
#[packed_struct(endian="lsb", bit_numbering="msb0")]
pub struct VBANServiceHeader {
    #[packed_field(element_size_bytes="5")]
    pub header: VBANHeader,
    pub function: u8,
    pub service: u8,
    pub additional_info: u8,
    pub stream_name: [u8; 16],
    pub frame_id: u32
}

#[derive(PackedStruct, Debug, Clone, Copy, PartialEq)]
#[packed_struct(endian="lsb", bit_numbering="msb0")]
pub struct RegisterRT {
    #[packed_field(element_size_bytes="28")]
    pub header: VBANServiceHeader,
    pub packet_ids: [u8; 128]
}

#[derive(PackedStruct, Debug, Clone, Copy, PartialEq)]
#[packed_struct(endian="lsb", bit_numbering="msb0")]
pub struct RTPacket {
    #[packed_field(element_size_bytes="28")]
    pub header: VBANServiceHeader,

    // packet body
    pub voicemeeter_type: u8,
    pub reserved: u8,
    pub buffer_size: u16,
    voicemeeter_version_raw: [u8; 4],
    pub options: u32,
    pub sample_rate: u32,
    input_levels_raw: [u16; 34],
    output_levels_raw: [u16; 64],
    pub transport: u32,
    pub strip_state: [u32; 8],
    pub bus_state: [u32; 8],
    strip_gain_layer_1_raw: [i16; 8],
    strip_gain_layer_2_raw: [i16; 8],
    strip_gain_layer_3_raw: [i16; 8],
    strip_gain_layer_4_raw: [i16; 8],
    strip_gain_layer_5_raw: [i16; 8],
    strip_gain_layer_6_raw: [i16; 8],
    strip_gain_layer_7_raw: [i16; 8],
    strip_gain_layer_8_raw: [i16; 8],
    bus_gain_raw: [i16; 8],
    strip_labels_raw: [u8; 480],
    bus_labels_raw: [u8; 480],
}

impl RTPacket {
    pub fn voicemeeter_version(&self) -> [u8; 4] {
        let mut arr = self.voicemeeter_version_raw;
        arr.reverse();
        return arr;
    }

    fn normalize_level(level: &u16) -> u16 {
        return ((1 << 16) - 1) as u16 - level;
    }

    fn gains(raw_gains: [i16; 8]) -> [f32; 8] {
      return raw_gains.map(|gain| (gain as f32 * 0.01));
    }

    pub fn input_gains(&self) -> [f32; 8] {
        return Self::gains(self.strip_gain_layer_1_raw);
    }

    pub fn output_gains(&self) -> [f32; 8] {
      return Self::gains(self.bus_gain_raw);
  }

    pub fn input_levels(&self) -> [[u16; 2]; 8] {
        let physicals = &self.input_levels_raw[0..10];
        let virtuals = &self.input_levels_raw[10..34];

        let mut out: [[u16; 2]; 8] = [[0; 2]; 8];
        for i in 0..5 {
            out[i] = [Self::normalize_level(&physicals[i * 2]), Self::normalize_level(&physicals[(i * 2) + 1])];
        }
        for i in 0..3 {
            out[i + 5] = [Self::normalize_level(&virtuals[i * 8]), Self::normalize_level(&virtuals[(i * 8) + 1])];
        }
        return out;
    }

    pub fn output_levels(&self) -> [[u16; 2]; 8] {
        let physicals = &self.output_levels_raw[0..40];
        let virtuals = &self.output_levels_raw[40..64];

        let mut out: [[u16; 2]; 8] = [[0; 2]; 8];
        for i in 0..5 {
            out[i] = [Self::normalize_level(&physicals[i * 8]), Self::normalize_level(&physicals[(i * 8) + 1])];
        }
        for i in 0..3 {
            out[i + 5] = [Self::normalize_level(&virtuals[i * 8]), Self::normalize_level(&virtuals[(i * 8) + 1])];
        }
        return out;
    }

    fn levels_to_meters(levels: [[u16; 2]; 8]) -> [i16; 8] {
      let mut out: [i16; 8] = [0; 8];

      for i in 0..8 {
          let raw_levels = levels[i];
          let level_sum = raw_levels[0] as f32 + raw_levels[1] as f32;
          let level_avg = level_sum / 2.0;
          let mut level_normalised = level_avg * -0.01;
          if level_normalised < -200.0 {
              level_normalised = 0.0;
          } else if level_normalised < -100.0 {
              level_normalised = -100.0;
          }
          out[i] = (((level_normalised + 100.0) / (15.0 + 100.0)) * 15.0) as i16;
      }

      return out;
    }

    pub fn input_meters(&self) -> [i16; 8] {
      return Self::levels_to_meters(self.input_levels());
    }

    pub fn output_meters(&self) -> [i16; 8] {
      return Self::levels_to_meters(self.output_levels());
    }

    fn format_labels(raw_labels: [u8; 480]) -> [String; 8] {
      let mut out: [String; 8] = ["", "", "", "", "", "", "", ""].map(|s| s.to_string());
      for i in 0..8 {
          let raw_string = &raw_labels[(i * 60)..((i * 60) + 60)];
          let label = std::str::from_utf8(raw_string).expect("invalid utf-8 sequence").to_string();
          out[i] = label;
      }
      return out;
    }

    pub fn strip_labels(&self) -> [String; 8] {
      return Self::format_labels(self.strip_labels_raw);
    }

    pub fn bus_labels(&self) -> [String; 8] {
      return Self::format_labels(self.bus_labels_raw);
    }
}

#[derive(PackedStruct, Debug, Clone, Copy, PartialEq)]
#[packed_struct(endian="lsb", bit_numbering="msb0")]
pub struct VBANMidiHeader {
    #[packed_field(element_size_bytes="5")]
    pub header: VBANHeader,
    pub bitmode: u8,
    pub channels: u8,
    pub data_format: u8, // 0x10 for midi
    pub stream_name: [u8; 16],
    pub frame_id: u32
}

#[derive(PackedStruct, Debug, Clone, Copy, PartialEq)]
#[packed_struct(endian="lsb", bit_numbering="msb0")]
pub struct MidiPacket {
    #[packed_field(element_size_bytes="28")]
    pub header: VBANMidiHeader,
    pub body: [u8; 3]
}

impl MidiPacket {
  pub fn new(packet_data: [u8; 3], frame_id: u32) -> Self {
    return MidiPacket {
      header: VBANMidiHeader {
        header: VBANHeader {
          vban: [0x56, 0x42, 0x41, 0x4e], // "VBAN"
          protocol: VBANProtocol::Serial as u8
        },
        bitmode: 0,
        channels: 0,
        data_format: 0x10, // 0x10 for MIDI with no other format options
        stream_name: [0x4d, 0x49, 0x44, 0x49, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // "MIDI1"
        frame_id: frame_id
      },
      body: packet_data
    };
  }
}
