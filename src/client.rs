//! WebSocket communication with clients

use actix::prelude::*;
use actix_web::{self, HttpRequest, HttpResponse};
use actix_web::ws;
use serde_json;

use std::borrow::Cow;

use lobby;
use server::AppState;

////////////////////////////////////////////////////////////////////////
// client-server messages
////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
pub struct RosterClient {
    pub name: String,
    pub peer: Option<String>,
    pub user_agent: Option<String>,
}

/// Messages of this type will be used to inform clients of their name and the other clients
/// currently connected.
#[derive(Serialize, Deserialize, Debug)]
pub struct Roster {
    pub name: String,
    pub clients: Vec<RosterClient>,
}

/// Messages of this type will be relayed between clients.
#[derive(Serialize, Deserialize, Debug)]
pub struct Relay {
    pub name: String,
    pub json: String,
}

/// Encapsulate all possible messages between the client and server.
#[derive(Serialize, Deserialize, Debug)]
#[derive(Message)]
pub enum ClientMessage {
    Relay(Relay),
    Roster(Roster),
}

////////////////////////////////////////////////////////////////////////

/// Start an actor for this new websocket.
pub fn ws_index(req: &HttpRequest<AppState>) -> Result<HttpResponse, actix_web::Error> {
    let peer = req.connection_info().remote().map(|v| v.to_string());
    let user_agent = req
        .headers()
        .get(actix_web::http::header::USER_AGENT)
        .map(|v| String::from_utf8_lossy(v.as_bytes()))
        .map(|s| simplify_user_agent(s))
        .map(|s| s.into_owned());
    ws::start(req, Client {
        name: None,
        peer,
        user_agent,
    })
}

/// Attempt to parse the client's user agent string, and reduce it to a simplified form.
fn simplify_user_agent<'a>(user_agent: Cow<'a,str>) -> Cow<'a,str>{
    let parser = ::woothee::parser::Parser::new();
    if let Some(ua) = parser.parse(&user_agent) {
        Cow::Owned(format!("{} {} {}\n{} {}", ua.vendor, ua.name, ua.version, ua.os, ua.os_version))
    } else {
        user_agent
    }
}

/// Represent a single client websocket connection.
struct Client {
    name: Option<String>,
    peer: Option<String>,
    user_agent: Option<String>,
}

impl Client {
    fn display_name(&self) -> &str {
        self.name.as_ref().map(|s|&s[..]).unwrap_or("none")
    }
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("New client connecting.");

        ctx.state().lobby.send(lobby::Connect {
            addr: ctx.address().recipient(),
            peer: self.peer.clone(),
            user_agent: self.user_agent.clone(),
        })
            .into_actor(self)
            .then(|res, act, ctx| {
                info!("Connection result: {:?}", res);
                match res {
                    Ok(res) => {
                        match res.roster {
                            Some(roster) => {
                                act.name = Some(roster.name.clone());
                                let msg = ClientMessage::Roster(roster);
                                ctx.text(json!(msg).to_string());
                            }
                            None => {
                                // The lobby did not accept the connection.
                                ctx.stop();
                            }
                        }
                    },
                    Err(_) => ctx.stop(),
                }
                actix::fut::ok(())
            })
        .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        info!("[{}] stopping.", self.display_name());
        if let Some(name) = self.name.take() {
            ctx.state().lobby.do_send(lobby::Disconnect {name});
        }
        Running::Stop
    }
}

// Handler for ws::Message messages
impl StreamHandler<ws::Message, ws::ProtocolError> for Client {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        info!("[{}] recv msg: {:?}", self.display_name(), msg);

        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                // Deserialize the message and send it to the lobby.
                let message = match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(message) => message,
                    Err(e) => {
                        info!("[{}] cannot parse incoming message: {:?}", self.display_name(), e);
                        return;
                    }
                };
                let name = match self.name {
                    Some(ref name) => name.clone(),
                    None => {
                        info!("[{}] message received before name assignment", self.display_name());
                        return;
                    }
                };
                ctx.state().lobby.do_send(lobby::RecvClientMessage {
                    name,
                    message
                });
            },
            _ => (),
        }
    }
}

impl Handler<ClientMessage> for Client {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut <Self as Actor>::Context) -> Self::Result {
        info!("[{}] send msg: {:?}", self.display_name(), msg);
        ctx.text(json!(msg).to_string());
    }
}
