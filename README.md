# webrtc-demo-server

This Rust program demonstrates WebRTC data channels with a simple
back-end signalling server.  It is based on Actix and uses websockets to
each connected client.  The main page of the web application shows a
roster of all connected clients, each with a "chat" button allowing the
user to initiate a WebRTC data channel chat with any of them.

### Known issues

Sometimes the Bootstrap modal (used for chat) doesn't open and close
("show" and "hide") when it is supposed to.

### License

This software is distributed under the terms of both the MIT license and
the Apache License (Version 2.0).  See LICENSE-APACHE and LICENSE-MIT
for details.

#### Contributing

Unless you explicitly state otherwise, any contribution you intentionally submit
for inclusion in the work, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.
