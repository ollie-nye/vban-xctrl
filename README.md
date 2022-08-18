# VBAN -> XCtrl translator

This serves as a network based translator for interfacing the Behringer X-Touch
with VoiceMeeter Potato. It is designed to run on a machine separate to the one
running VoiceMeeter, either as a Docker image, VM, or distinct machine.

Behringer X-Touch: https://www.behringer.com/product.html?modelCode=P0B1X
VoiceMeeter Potato: https://vb-audio.com/Voicemeeter/potato.htm

## Installing

This utility is built using Rust, you may need to install that first:
https://www.rust-lang.org/tools/install

After pulling the repository, build an executable for your OS with
`cargo build --release`.

Then, from the `./target/release` directory, run

```
./vban_xctrl <machine ip> <xtouch ip> <vban ip>
```

This will register the program with VoiceMeeter running on the `<vban ip>`
machine, push any VoiceMeeter changes out to the X-Touch at `<xtouch ip>`,
receive control changes from the X-Touch, and send them as virtual MIDI messages
back into VoiceMeeter.

It supports two virtual pages, controlled by the 'Fader Bank' buttons on the
surface. The first page mirrors VoiceMeeter inputs, and the second shows the
outputs.
