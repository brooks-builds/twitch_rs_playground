use eyre::{bail, Result};
use std::{net::TcpStream, ops::Deref};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{self, protocol::WebSocketConfig, Message},
    MaybeTlsStream, WebSocketStream,
};
use twitch_api::{
    eventsub::{self, Event, SessionData},
    helix::users::User,
    twitch_oauth2::UserToken,
    HelixClient, TWITCH_EVENTSUB_WEBSOCKET_URL,
};

pub type BBHelixClient<'a> = HelixClient<'a, reqwest::Client>;

pub struct WebsocketClient {}

impl WebsocketClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>> {
        let config = WebSocketConfig {
            max_message_size: Some(64 << 20),
            max_frame_size: Some(16 << 20),
            accept_unmasked_frames: false,
            ..WebSocketConfig::default()
        };
        let (socket, _) =
            connect_async_with_config(TWITCH_EVENTSUB_WEBSOCKET_URL.deref(), Some(config), false)
                .await?;

        Ok(socket)
    }

    pub async fn run<'a>(
        mut self,
        client: &'a BBHelixClient<'a>,
        token: &UserToken,
        user: &User,
    ) -> Result<()> {
        let mut stream = self.connect().await?;

        loop {
            tokio::select! (
                Some(message) = futures::StreamExt::next(&mut stream) => {
                    let message = match message {
                        Err(tungstenite::Error::Protocol(tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,)) => {
                            self.connect().await?;
                            continue;
                        }
                        _ => message?,
                    };
                    self.process_message(message, client, token, user).await?;
                }

            )
        }
    }

    async fn process_message<'a>(
        &mut self,
        message: Message,
        helix_client: &'a BBHelixClient<'a>,
        token: &UserToken,
        user: &User,
    ) -> Result<()> {
        match message {
            Message::Text(message_text) => match Event::parse_websocket(&message_text)? {
                twitch_api::eventsub::EventsubWebsocketData::Welcome { metadata, payload } => {
                    self.process_welcome_message(payload.session, helix_client, token, user)
                        .await?;
                }
                twitch_api::eventsub::EventsubWebsocketData::Keepalive { metadata, payload } => (),
                twitch_api::eventsub::EventsubWebsocketData::Notification { metadata, payload } => {
                    match payload {
                        Event::ChannelPointsCustomRewardAddV1(reward) => {
                            println!("channel point redeemed: {:?}", reward.message)
                        }
                        _ => (),
                    }
                }
                twitch_api::eventsub::EventsubWebsocketData::Revocation { metadata, payload } => {
                    bail!("revocating!");
                }
                twitch_api::eventsub::EventsubWebsocketData::Reconnect { metadata, payload } => {
                    self.process_welcome_message(payload.session, helix_client, token, user)
                        .await?;
                }
                _ => (),
            },
            Message::Binary(_) => eprintln!("Message::Binary"),
            Message::Ping(_) => eprintln!("Message::Ping"),
            Message::Pong(_) => eprintln!("Message::Pong"),
            Message::Close(_) => eprintln!("Message::Close"),
            Message::Frame(_) => eprintln!("Message::Frame"),
        }
        Ok(())
    }

    async fn process_welcome_message<'a>(
        &mut self,
        data: SessionData<'_>,
        helix_client: &'a BBHelixClient<'a>,
        token: &UserToken,
        user: &User,
    ) -> Result<()> {
        let transport = eventsub::Transport::websocket(data.id);

        helix_client
            .create_eventsub_subscription(
                eventsub::channel::ChannelPointsCustomRewardAddV1::broadcaster_user_id(
                    user.id.clone(),
                ),
                transport.clone(),
                token,
            )
            .await?;
        Ok(())
    }
}
