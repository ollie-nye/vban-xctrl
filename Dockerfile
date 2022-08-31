FROM rust:1.61.0

WORKDIR /usr/src/vban_xctrl
COPY . .

RUN cargo install --path .

CMD vban_xctrl $XTOUCH_IP $VBAN_IP
