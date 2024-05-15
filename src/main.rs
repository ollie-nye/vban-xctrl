use std::net::UdpSocket;
use std::thread;
use std::time;
use std::time::SystemTime;
use std::env;

extern crate hex;
extern crate vban_xctrl;
extern crate packed_struct;

use packed_struct::prelude::*;
use std::convert::TryInto;

pub use vban_xctrl::*;

#[derive(Clone, Copy)]
pub enum StateUpdate {
    Vban(RTPacket),
    Xctrl(XctrlStateUpdate),
    XtouchConnect()
}

pub enum VbanStripFlags {
    Mute = 0b00000001,
    Solo = 0b00000010,
    Mono = 0b00000100,
}

fn xctrl_incoming_thread(queue: vban_xctrl::WorkQueue<String>, socket: UdpSocket) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        loop {
            let mut buf = [0; 32];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    let buf = &mut buf[..amt];
                    let message = hex::encode(buf);
                    queue.add_work(message.clone());
                },
                Err(e) => {
                    eprintln!("Error receiving XCtrl data: {:?}", e);
                }
            }

            std::thread::yield_now();
        }
    });
}

fn vban_incoming_thread(queue: vban_xctrl::WorkQueue<String>, socket: UdpSocket) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        loop {
            let mut buf = [0; 1412];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    let buf = &mut buf[..amt];
                    let message = hex::encode(buf);
                    queue.add_work(message.clone());
                },
                Err(e) => {
                    eprintln!("Error receiving VBAN data: {:?}", e);
                }
            }

            std::thread::yield_now();
        }
    });
}

fn xctrl_outgoing_thread(ip: String, queue: vban_xctrl::WorkQueue<String>, socket: UdpSocket) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        loop {
            if let Some(message) = queue.get_work() {
                let buf = hex::decode(&message).unwrap();
                if let Err(e) = socket.send_to(&buf, &ip) {
                    eprintln!("Error sending XCtrl data: {:?}", e);
                }
                // println!("Sent {} to x-touch", message);
            } else {
                thread::sleep(time::Duration::from_millis(10));
            }

            std::thread::yield_now();
        }
    });
}

fn vban_outgoing_thread(ip: String, queue: vban_xctrl::WorkQueue<String>, socket: UdpSocket) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        loop {
            if let Some(message) = queue.get_work() {
                let buf = hex::decode(&message).unwrap();
                if let Err(e) = socket.send_to(&buf, &ip) {
                    eprintln!("Error sending VBAN data: {:?}", e);
                }
                // println!("Sent {} to vban", message);
            } else {
                thread::sleep(time::Duration::from_millis(10));
            }

            std::thread::yield_now();
        }
    });
}

fn xctrl_processor_thread(incoming: vban_xctrl::WorkQueue<String>, outgoing: vban_xctrl::WorkQueue<String>, state: vban_xctrl::WorkQueue<StateUpdate>) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        loop {
            if let Some(message) = incoming.get_work() {
                if message == "f0002032585400f7" {
                    let response = hex::encode([0xf0, 0x00, 0x00, 0x66, 0x14, 0x00, 0xf7]);
                    outgoing.add_work(response.clone());

                    state.add_work(StateUpdate::XtouchConnect());
                } else if message == "f000006658013031353634303833393344f7" {
                    continue;
                } else {
                    let buf = hex::decode(&message).expect("can't decode string into hex");

                    let command: XctrlInterface = XctrlInterface::from(buf[0] >> 4);
                    let mut id: u8 = 0;
                    let mut value: u16 = 0;

                    match command {
                        XctrlInterface::Fader => {
                            id = buf[0] & 0x0f;
                            value = (((buf[2] as u32) << 8) + buf[1] as u32) as u16;
                        },
                        XctrlInterface::Button => {
                            id = buf[1];
                            value = buf[2] as u16;
                        },
                        XctrlInterface::Encoder => {
                            println!("Processing encoder change");
                        },
                        XctrlInterface::Unknown => {
                            println!("Processing unknown change");
                        }
                    }

                    let raw_message: [u8; 3] = hex::decode(&message).expect("could not decode hex")[0..3].try_into().unwrap();

                    let state_update = XctrlStateUpdate { interface_type: command, id: id, value: value, raw_message: raw_message };
                    state.add_work(StateUpdate::Xctrl(state_update));
                }
            } else {
                thread::sleep(time::Duration::from_millis(10));
            }

            std::thread::yield_now();
        }
    });
}

fn vban_heartbeat_thread(vban_outgoing: vban_xctrl::WorkQueue<String>) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        let packet: RegisterRT = RegisterRT {
            header: VBANServiceHeader {
                header: VBANHeader {
                    vban: [0x56, 0x42, 0x41, 0x4e], // "VBAN"
                    protocol: VBANProtocol::Service as u8
                },
                function: 0,
                service: 32,
                additional_info: 50,
                stream_name: [0x58, 0x2d, 0x54, 0x6f, 0x75, 0x63, 0x68, 0x20, 0x6d, 0x65, 0x74, 0x65, 0x72, 0x73, 0x00, 0x00], //"X-Touch meters"
                frame_id: 1
            },
            packet_ids: [1; 128]
        };

        let packet_data = packet.pack().expect("couldn't pack the packet");

        loop {
            vban_outgoing.add_work(hex::encode(packet_data));
            thread::sleep(time::Duration::from_millis(10000));
        }
    });
}

fn vban_processor_thread(vban_incoming: vban_xctrl::WorkQueue<String>, state: vban_xctrl::WorkQueue<StateUpdate>) -> thread::JoinHandle<()> {
    return thread::spawn(move || {
        loop {
            if let Some(message) = vban_incoming.get_work() {
                let header: [u8; 5] = hex::decode(&message).expect("could not decode hex")[0..5].try_into().unwrap();

                let packet = VBANHeader::unpack(&header).expect("not a vban packet");
                if packet.vban == "VBAN".as_bytes() {
                    if packet.protocol == VBANProtocol::Service as u8 {
                        let service_buf: [u8; 28] = hex::decode(&message).expect("could not decode hex")[0..28].try_into().unwrap();
                        let service_header = VBANServiceHeader::unpack(&service_buf).expect("packet isn't a service header");
                        if service_header.service == 32 && service_header.additional_info == 1 {
                            println!("VoiceMeeter registered");
                        } else if service_header.stream_name == "Voicemeeter-RTP\0".as_bytes() {
                            let buf: [u8; 1412] = hex::decode(&message).expect("could not decode hex").try_into().unwrap();
                            let rt_packet = RTPacket::unpack(&buf).expect("packet isn't a rt service");

                            state.add_work(StateUpdate::Vban(rt_packet));
                        }
                    }
                } else {
                    println!("Didn't receive VBAN packet :(")
                }
            } else {
                thread::sleep(time::Duration::from_millis(10));
            }

            std::thread::yield_now();
        }
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let bind_ip = &args[1];
    let xtouch_ip = &args[2];
    let vban_ip = &args[3];

    let xctrl_bind = format!("{bind_ip}:10111");
    let vban_bind = format!("{bind_ip}:6980");
    let xtouch_addr = format!("{xtouch_ip}:10111");
    let vban_addr = format!("{vban_ip}:6980");

    let xctrl_incoming: vban_xctrl::WorkQueue<String> = vban_xctrl::WorkQueue::new();
    let xctrl_outgoing: vban_xctrl::WorkQueue<String> = vban_xctrl::WorkQueue::new();
    let state: vban_xctrl::WorkQueue<StateUpdate> = vban_xctrl::WorkQueue::new();
    let xctrl_socket = UdpSocket::bind(xctrl_bind).unwrap();

    let vban_incoming: vban_xctrl::WorkQueue<String> = vban_xctrl::WorkQueue::new();
    let vban_outgoing: vban_xctrl::WorkQueue<String> = vban_xctrl::WorkQueue::new();
    let vban_socket = UdpSocket::bind(vban_bind).unwrap();

    let mut threads = Vec::new();

    println!("Starting tx/rx threads for XCtrl");
    threads.push(xctrl_incoming_thread(xctrl_incoming.clone(), xctrl_socket.try_clone().expect("couldn't clone the socket")));
    threads.push(xctrl_outgoing_thread(xtouch_addr, xctrl_outgoing.clone(), xctrl_socket.try_clone().expect("couldn't clone the socket")));

    println!("Starting tx/rx threads for VBAN");
    threads.push(vban_incoming_thread(vban_incoming.clone(), vban_socket.try_clone().expect("couldn't clone the socket")));
    threads.push(vban_outgoing_thread(vban_addr, vban_outgoing.clone(), vban_socket.try_clone().expect("couldn't clone the socket")));

    println!("Starting processor threads");
    threads.push(xctrl_processor_thread(xctrl_incoming.clone(), xctrl_outgoing.clone(), state.clone()));
    threads.push(vban_processor_thread(vban_incoming.clone(), state.clone()));
    threads.push(vban_heartbeat_thread(vban_outgoing.clone()));

    let mut x_touch_page = 0;
    let mut x_touch_state = [XctrlState::new(), XctrlState::new()];

    let mut last_update_send = SystemTime::now();
    let mut display_string: String;
    let mut controls_string: String;
    let mut buttons_string: String;

    let mut faders_updated = false;
    let mut connection_time = SystemTime::now();

    let mut frame_id: u32 = 0;

    loop {
        if let Some(message) = state.get_work() {
            match message {
                StateUpdate::XtouchConnect() => {
                    let last_connection_diff = SystemTime::now().duration_since(connection_time).expect("SystemTime::duration_since failed");
                    if last_connection_diff.as_secs() > 5 {
                        faders_updated = true;
                        println!("New connection made, updating faders...");
                    }
                    connection_time = SystemTime::now();
                },
                StateUpdate::Xctrl(update) => {
                    match update.interface_type {
                        XctrlInterface::Button => {
                            // if update.id >= XctrlButtonType::Rec as u8 && update.id < XctrlButtonType::Rec as u8 {
                            //     current_surface.recs[update.id - XctrlButtonType::Rec] = update.value;
                            // }
                            if update.value == 127 {
                                if update.id == 47 {
                                    x_touch_page = 1;
                                    faders_updated = true;
                                } else if update.id == 46 {
                                    x_touch_page = 0;
                                    faders_updated = true;
                                }
                            }
                        },
                        _ => {}
                    }
                    frame_id += 1;
                    let mut raw_message = update.raw_message;
                    raw_message[0] = raw_message[0] + (0x08 * x_touch_page as u8);
                    let vban_midi_update: [u8; 31] = MidiPacket::new(raw_message, frame_id).pack().unwrap();
                    vban_outgoing.add_work(hex::encode(vban_midi_update));

                },
                StateUpdate::Vban(update) => {
                    let labels = update.strip_labels();
                    for i in 0..8 {
                        let label: &[u8] = &labels[i].as_bytes();
                        let top: &[u8] = &label[0..7];
                        let bottom: &[u8] = &label[7..14];
                        let mut color = XctrlDisplayColor::Green;
                        if top == [0, 0, 0, 0, 0, 0, 0] && bottom == [0, 0, 0, 0, 0, 0, 0] {
                            color = XctrlDisplayColor::Off;
                        }
                        x_touch_state[0].displays[i] = XctrlDisplay::new(i as u8, color, top, bottom);
                    }

                    let labels = update.bus_labels();
                    for i in 0..8 {
                        let label: &[u8] = &labels[i].as_bytes();
                        let top: &[u8] = &label[0..7];
                        let bottom: &[u8] = &label[7..14];
                        let mut color = XctrlDisplayColor::Blue;
                        if top == [0, 0, 0, 0, 0, 0, 0] && bottom == [0, 0, 0, 0, 0, 0, 0] {
                            color = XctrlDisplayColor::Off;
                        }
                        x_touch_state[1].displays[i] = XctrlDisplay::new(i as u8, color, top, bottom);
                    }

                    let gains = update.input_gains();
                    for i in 0..8 {
                        let scaled_gain = (((gains[i] + 60.0) / (12.0 + 60.0)) * 32767.0) as u16;
                        let fader = XctrlFader { id: i as u8, level: scaled_gain };
                        if x_touch_state[0].faders[i].level != scaled_gain {
                            faders_updated = true;
                        }
                        x_touch_state[0].faders[i] = fader;
                    }

                    let gains = update.output_gains();
                    for i in 0..8 {
                        let scaled_gain = (((gains[i] + 60.0) / (12.0 + 60.0)) * 32767.0) as u16;
                        let fader = XctrlFader { id: i as u8, level: scaled_gain };
                        if x_touch_state[1].faders[i].level != scaled_gain {
                            faders_updated = true;
                        }
                        x_touch_state[1].faders[i] = fader;
                    }

                    let flags = update.strip_state;
                    for i in 0..8 {
                        let flag = flags[i];
                        if (flag & VbanStripFlags::Mute as u32) == VbanStripFlags::Mute as u32 {
                            x_touch_state[0].buttons[XctrlButtonType::Mute as usize + i] = XctrlButton { id: XctrlButtonType::Mute as u8 + i as u8, state: 127 };
                        } else {
                            x_touch_state[0].buttons[XctrlButtonType::Mute as usize + i] = XctrlButton { id: XctrlButtonType::Mute as u8 + i as u8, state: 0 };
                        }
                        if (flag & VbanStripFlags::Solo as u32) == VbanStripFlags::Solo as u32 {
                            x_touch_state[0].buttons[XctrlButtonType::Solo as usize + i] = XctrlButton { id: XctrlButtonType::Solo as u8 + i as u8, state: 127 };
                        } else {
                            x_touch_state[0].buttons[XctrlButtonType::Solo as usize + i] = XctrlButton { id: XctrlButtonType::Solo as u8 + i as u8, state: 0 };
                        }
                        if (flag & VbanStripFlags::Mono as u32) == VbanStripFlags::Mono as u32 {
                            x_touch_state[0].buttons[XctrlButtonType::Rec as usize + i] = XctrlButton { id: XctrlButtonType::Rec as u8 + i as u8, state: 127 };
                        } else {
                            x_touch_state[0].buttons[XctrlButtonType::Rec as usize + i] = XctrlButton { id: XctrlButtonType::Rec as u8 + i as u8, state: 0 };
                        }
                    }

                    let flags = update.bus_state;
                    for i in 0..8 {
                        let flag = flags[i];
                        if (flag & VbanStripFlags::Mute as u32) == VbanStripFlags::Mute as u32 {
                            x_touch_state[1].buttons[XctrlButtonType::Mute as usize + i] = XctrlButton { id: XctrlButtonType::Mute as u8 + i as u8, state: 127 };
                        } else {
                            x_touch_state[1].buttons[XctrlButtonType::Mute as usize + i] = XctrlButton { id: XctrlButtonType::Mute as u8 + i as u8, state: 0 };
                        }
                        if (flag & VbanStripFlags::Solo as u32) == VbanStripFlags::Solo as u32 {
                            x_touch_state[1].buttons[XctrlButtonType::Solo as usize + i] = XctrlButton { id: XctrlButtonType::Solo as u8 + i as u8, state: 127 };
                        } else {
                            x_touch_state[1].buttons[XctrlButtonType::Solo as usize + i] = XctrlButton { id: XctrlButtonType::Solo as u8 + i as u8, state: 0 };
                        }
                        if (flag & VbanStripFlags::Mono as u32) == VbanStripFlags::Mono as u32 {
                            x_touch_state[1].buttons[XctrlButtonType::Rec as usize + i] = XctrlButton { id: XctrlButtonType::Rec as u8 + i as u8, state: 127 };
                        } else {
                            x_touch_state[1].buttons[XctrlButtonType::Rec as usize + i] = XctrlButton { id: XctrlButtonType::Rec as u8 + i as u8, state: 0 };
                        }
                    }

                    let m = update.input_meters();
                    for i in 0..8 {
                        let meter = XctrlMeter { id: i as u8, level: m[i] as u8 };
                        x_touch_state[0].meters[i] = meter;
                    }

                    let m = update.output_meters();
                    for i in 0..8 {
                        let meter = XctrlMeter { id: i as u8, level: m[i] as u8 };
                        x_touch_state[1].meters[i] = meter;
                    }

                    x_touch_state[0].buttons[XctrlButtonType::FaderBank as usize + 0] = XctrlButton { id: XctrlButtonType::FaderBank as u8 + 0, state: 127 };
                    x_touch_state[1].buttons[XctrlButtonType::FaderBank as usize + 1] = XctrlButton { id: XctrlButtonType::FaderBank as u8 + 1, state: 127 };
                },
            }

            if SystemTime::now().duration_since(last_update_send).expect("Time went backwards").as_millis() > 50 {
                let current_surface = &x_touch_state[x_touch_page];

                last_update_send = SystemTime::now();

                display_string = "".to_owned();
                for display in &current_surface.displays {
                    display_string.push_str(&display.as_str());
                }
                xctrl_outgoing.add_work(display_string);

                controls_string = "".to_owned();
                for meter in &current_surface.meters {
                    controls_string.push_str(&meter.as_str());
                }
                if faders_updated {
                    for fader in &current_surface.faders {
                        controls_string.push_str(&fader.as_str());
                    }
                }
                xctrl_outgoing.add_work(controls_string);

                buttons_string = "".to_owned();
                for button in &current_surface.buttons {
                    buttons_string.push_str(&button.as_str());
                }
                xctrl_outgoing.add_work(buttons_string);

                faders_updated = false;
            }
        } else {
            thread::sleep(time::Duration::from_millis(10));
        }
    }

    // for handle in threads {
    //     handle.join().unwrap();
    // }
}
