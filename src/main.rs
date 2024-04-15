use std::arch::aarch64::vgetq_lane_p64;
use std::env;
use twitch_api::client::Body;
use twitch_api::eventsub::channel::ChannelPointsCustomRewardRedemptionAddV1;
use twitch_api::eventsub::{EventSubscription, WebsocketTransport};
use twitch_api::helix::eventsub::CreateEventSubSubscriptionRequest;
use twitch_api::helix::{HelixClient, RequestPost};
use twitch_api::twitch_oauth2::{AccessToken, UserToken};
use twitch_rs_playground::websocket::WebsocketClient;
use twitch_rs_playground::{auth, websocket};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let twitch_client_id = std::env::var("TWITCH_CLIENT_ID").unwrap();
    let twitch_client_secret = std::env::var("TWITCH_CLIENT_SECRET").unwrap();
    let twitch_redirect_url = std::env::var("TWITCH_REDIRECT_URL").unwrap();
    let twitch_username = std::env::var("TWITCH_USERNAME").unwrap();
    let user_token =
        auth::get_user_token(twitch_client_id, twitch_client_secret, &twitch_redirect_url)
            .await
            .unwrap();
    let helix_client: HelixClient<reqwest::Client> = HelixClient::default();
    let user = helix_client
        .get_user_from_login(&twitch_username, &user_token)
        .await
        .unwrap()
        .unwrap();

    let mut websocket = WebsocketClient::new();

    websocket
        .run(&helix_client, &user_token, &user)
        .await
        .unwrap();
}
