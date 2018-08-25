//! The "lobby" actor here maintains a roster of connected clients, and supports a simple messaging
//! system for clients to relay messages to each other.

use std::collections::HashMap;

use actix::prelude::*;

use names;
use client::{ClientMessage,Roster,RosterClient};

const MAX_CLIENTS: usize = 10;

/// A single connected client being tracked in the roster.
pub struct ConnectedClient {
    /// The randomly generated name for the client.
    pub name: String,
    /// The address of the client's websocket actor.
    pub addr: Recipient<ClientMessage>,
    /// The IP address and port number of the client.
    pub peer: Option<String>,
    /// The client's user agent
    pub user_agent: Option<String>,
}

/// The lobby actor only needs to maintain a roster of connected clients.
pub struct Lobby {
    pub clients: HashMap<String, ConnectedClient>,
}

impl Lobby {
    pub fn new() -> Lobby {
        Lobby {
            clients: HashMap::new(),
        }
    }

    /// Build a serializable version of the roster to send to clients.
    fn roster(&self, name: &str) -> Roster {
        let mut clients = vec![];
        for (_, client) in &self.clients {
            clients.push(RosterClient {
                name: client.name.clone(),
                peer: client.peer.clone(),
                user_agent: client.user_agent.clone(),
            });
        }
        clients.sort_by(|a,b| a.name.cmp(&b.name));
        Roster {
            name: name.to_string(),
            clients,
        }
    }

    /// Send an updated roster to every connected client.  This happens when a client connects or
    /// disconnects.
    fn broadcast_roster(&self, except: Option<&str>) {
        for client in self.clients.values() {
            if let Some(except) = except {
                if except == client.name {
                    continue;
                }
            }
            match client.addr.do_send(ClientMessage::Roster(self.roster(&client.name))) {
                Ok(_) => {},
                Err(e) => error!("Lobby: Unable to send roster to client {:?}: {}", client.name, e),
            }
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

////////////////////////////////////////////////////////////////////////

/// When a new client connects, this message is sent to the lobby actor.
#[derive(Message)]
#[rtype(ConnectResponse)]
pub struct Connect {
    pub addr: Recipient<ClientMessage>,
    pub peer: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug)]
pub struct ConnectResponse {
    pub roster: Option<Roster>,
}

impl<A,M> actix::dev::MessageResponse<A,M> for ConnectResponse
where
    A: Actor,
    M: Message<Result = ConnectResponse>,
{
    fn handle<R: actix::dev::ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ConnectResponse;

    /// Handle a new client connection.
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // Restrict the lobby to a maximum number of clients.
        if self.clients.len() >= MAX_CLIENTS {
            return ConnectResponse { roster: None };
        }

        // Create a new random name for this connection.
        let mut name = names::generate();
        let mut count = 0usize;
        while self.clients.contains_key(&name) {
            if count > MAX_CLIENTS {
                // We somehow can't come up with an original name, and don't want to loop forever.
                return ConnectResponse { roster: None };
            }
            name = names::generate();
            count += 1;
        }

        // Add the new client to the lobby.
        self.clients.insert(name.clone(), ConnectedClient {
            name: name.clone(),
            addr: msg.addr,
            peer: msg.peer,
            user_agent: msg.user_agent,
        });

        // Broadcast the updated roster to all clients
        self.broadcast_roster(Some(&name));

        // Return the current roster to the newly connected client.
        ConnectResponse {
            roster: Some(self.roster(&name))
        }
    }
}

////////////////////////////////////////////////////////////////////////

/// When a client disconnects, this message is sent to the lobby actor.
#[derive(Message)]
pub struct Disconnect {
    pub name: String,
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    /// Handle a client disconnection.
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        info!("Lobby: Disconnecting {:?}.", msg.name);
        self.clients.remove(&msg.name);

        // Broadcast the updated roster to all clients
        self.broadcast_roster(None);
    }
}

////////////////////////////////////////////////////////////////////////

/// This message is an envelope for an incoming client message.
#[derive(Debug,Message)]
pub struct RecvClientMessage {
    pub name: String,
    pub message: ClientMessage,
}

impl Handler<RecvClientMessage> for Lobby {
    type Result = ();

    /// Handle messages received from clients.  Currently the only supported message type is
    /// "relay".
    fn handle(&mut self, msg: RecvClientMessage, _: &mut Context<Self>) -> Self::Result {
        info!("Lobby recv message: {:?}", msg);
        let RecvClientMessage { name: mut sender, message } = msg;
        match message {
            ClientMessage::Relay(mut relay) => {
                // Handle relay messages by changing the name to the source, and forwarding the
                // message onward.

                // Swap sender and receiver names
                ::std::mem::swap(&mut relay.name, &mut sender);
                let receiver = sender;
                // Send to destination
                match self.clients.get(&receiver) {
                    Some(client) => {
                        match client.addr.do_send(ClientMessage::Relay(relay)) {
                            Ok(_) => {},
                            Err(e) => error!("Lobby: Unable to send roster to client {:?}: {}", client.name, e),
                        }
                    },
                    None => warn!("Cannot relay to unknown client \"{}\".", receiver),
                };
            },
            _ => {
                warn!("Discarding unexpected message from \"{}\": {:?}", sender, message);
            }
        }
    }
}
