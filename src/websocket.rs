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
                        Event::ChannelCharityCampaignDonateV1(reward) => println!(
                            "Event::ChannelCharityCampaignDonateV1: {:?}",
                            reward.message
                        ),
                        Event::ChannelCharityCampaignProgressV1(reward) => println!(
                            "Event::ChannelCharityCampaignProgressV1: {:?}",
                            reward.message
                        ),
                        Event::ChannelCharityCampaignStartV1(reward) => {
                            println!("Event::ChannelCharityCampaignStartV1: {:?}", reward.message)
                        }
                        Event::ChannelCharityCampaignStopV1(reward) => {
                            println!("Event::ChannelCharityCampaignStopV1: {:?}", reward.message)
                        }
                        Event::ChannelUpdateV1(reward) => {
                            println!("Event::ChannelUpdateV1: {:?}", reward.message)
                        }
                        Event::ChannelUpdateV2(reward) => {
                            println!("Event::ChannelUpdateV2: {:?}", reward.message)
                        }
                        Event::ChannelFollowV1(reward) => {
                            println!("Event::ChannelFollowV1: {:?}", reward.message)
                        }
                        Event::ChannelFollowV2(reward) => {
                            println!("Event::ChannelFollowV2: {:?}", reward.message)
                        }
                        Event::ChannelSubscribeV1(reward) => {
                            println!("Event::ChannelSubscribeV1: {:?}", reward.message)
                        }
                        Event::ChannelCheerV1(reward) => {
                            println!("Event::ChannelCheerV1: {:?}", reward.message)
                        }
                        Event::ChannelBanV1(reward) => {
                            println!("Event::ChannelBanV1: {:?}", reward.message)
                        }
                        Event::ChannelUnbanV1(reward) => {
                            println!("Event::ChannelUnbanV1: {:?}", reward.message)
                        }
                        Event::ChannelPointsCustomRewardUpdateV1(reward) => println!(
                            "Event::ChannelPointsCustomRewardUpdateV1: {:?}",
                            reward.message
                        ),
                        Event::ChannelPointsCustomRewardRemoveV1(reward) => println!(
                            "Event::ChannelPointsCustomRewardRemoveV1: {:?}",
                            reward.message
                        ),
                        Event::ChannelPointsCustomRewardRedemptionAddV1(reward) => println!(
                            "Event::ChannelPointsCustomRewardRedemptionAddV1: {:?}",
                            reward.message
                        ),
                        Event::ChannelPointsCustomRewardRedemptionUpdateV1(reward) => println!(
                            "Event::ChannelPointsCustomRewardRedemptionUpdateV1: {:?}",
                            reward.message
                        ),
                        Event::ChannelPollBeginV1(reward) => {
                            println!("Event::ChannelPollBeginV1: {:?}", reward.message)
                        }
                        Event::ChannelPollProgressV1(reward) => {
                            println!("Event::ChannelPollProgressV1: {:?}", reward.message)
                        }
                        Event::ChannelPollEndV1(reward) => {
                            println!("Event::ChannelPollEndV1: {:?}", reward.message)
                        }
                        Event::ChannelPredictionBeginV1(reward) => {
                            println!("Event::ChannelPredictionBeginV1: {:?}", reward.message)
                        }
                        Event::ChannelPredictionProgressV1(reward) => {
                            println!("Event::ChannelPredictionProgressV1: {:?}", reward.message)
                        }
                        Event::ChannelPredictionLockV1(reward) => {
                            println!("Event::ChannelPredictionLockV1: {:?}", reward.message)
                        }
                        Event::ChannelPredictionEndV1(reward) => {
                            println!("Event::ChannelPredictionEndV1: {:?}", reward.message)
                        }
                        Event::ChannelRaidV1(reward) => {
                            println!("Event::ChannelRaidV1: {:?}", reward.message)
                        }
                        Event::ChannelShieldModeBeginV1(reward) => {
                            println!("Event::ChannelShieldModeBeginV1: {:?}", reward.message)
                        }
                        Event::ChannelShieldModeEndV1(reward) => {
                            println!("Event::ChannelShieldModeEndV1: {:?}", reward.message)
                        }
                        Event::ChannelShoutoutCreateV1(reward) => {
                            println!("Event::ChannelShoutoutCreateV1: {:?}", reward.message)
                        }
                        Event::ChannelShoutoutReceiveV1(reward) => {
                            println!("Event::ChannelShoutoutReceiveV1: {:?}", reward.message)
                        }
                        Event::ChannelGoalBeginV1(reward) => {
                            println!("Event::ChannelGoalBeginV1: {:?}", reward.message)
                        }
                        Event::ChannelGoalProgressV1(reward) => {
                            println!("Event::ChannelGoalProgressV1: {:?}", reward.message)
                        }
                        Event::ChannelGoalEndV1(reward) => {
                            println!("Event::ChannelGoalEndV1: {:?}", reward.message)
                        }
                        Event::ChannelHypeTrainBeginV1(reward) => {
                            println!("Event::ChannelHypeTrainBeginV1: {:?}", reward.message)
                        }
                        Event::ChannelHypeTrainProgressV1(reward) => {
                            println!("Event::ChannelHypeTrainProgressV1: {:?}", reward.message)
                        }
                        Event::ChannelHypeTrainEndV1(reward) => {
                            println!("Event::ChannelHypeTrainEndV1: {:?}", reward.message)
                        }
                        Event::StreamOnlineV1(reward) => {
                            println!("Event::StreamOnlineV1: {:?}", reward.message)
                        }
                        Event::StreamOfflineV1(reward) => {
                            println!("Event::StreamOfflineV1: {:?}", reward.message)
                        }
                        Event::UserUpdateV1(reward) => {
                            println!("Event::UserUpdateV1: {:?}", reward.message)
                        }
                        Event::UserAuthorizationGrantV1(reward) => {
                            println!("Event::UserAuthorizationGrantV1: {:?}", reward.message)
                        }
                        Event::UserAuthorizationRevokeV1(reward) => {
                            println!("Event::UserAuthorizationRevokeV1: {:?}", reward.message)
                        }
                        Event::ChannelSubscriptionEndV1(reward) => {
                            println!("Event::ChannelSubscriptionEndV1: {:?}", reward.message)
                        }
                        Event::ChannelSubscriptionGiftV1(reward) => {
                            println!("Event::ChannelSubscriptionGiftV1: {:?}", reward.message)
                        }
                        Event::ChannelSubscriptionMessageV1(reward) => {
                            println!("Event::ChannelSubscriptionMessageV1: {:?}", reward.message)
                        }
                        _ => unreachable!(),
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
                eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1::broadcaster_user_id(
                    user.id.clone(),
                ),
                transport.clone(),
                token,
            )
            .await?;
        Ok(())
    }
}
