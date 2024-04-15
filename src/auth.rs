use std::collections::HashMap;

use eyre::{OptionExt, Result};
use reqwest::{redirect::Policy, Url};
use twitch_api::twitch_oauth2::{
    AccessToken, AppAccessToken, ClientId, ClientSecret, Scope, UserToken, UserTokenBuilder,
};

pub async fn get_app_access_token(
    client_id: String,
    client_secret: String,
) -> Result<AppAccessToken> {
    let client = reqwest::ClientBuilder::new()
        .redirect(Policy::none())
        .build()?;
    let client_id = ClientId::new(client_id);
    let client_secret = ClientSecret::new(client_secret);
    let token =
        AppAccessToken::get_app_access_token(&client, client_id, client_secret, vec![]).await?;

    Ok(token)
}

pub async fn get_user_token(
    client_id: String,
    client_secret: String,
    redirect_url: &str,
) -> Result<UserToken> {
    let client = reqwest::ClientBuilder::new()
        .redirect(Policy::none())
        .build()?;
    let client_id = ClientId::new(client_id);
    let client_secret = ClientSecret::new(client_secret);
    let redirect_url = twitch_api::twitch_oauth2::url::Url::parse(redirect_url)?;
    let mut token_builder = UserTokenBuilder::new(client_id, client_secret, redirect_url)
        .set_scopes(vec![
            Scope::ChannelReadRedemptions,
            Scope::ChannelManageRedemptions,
        ])
        .force_verify(true);
    let (url, _) = token_builder.generate_url();

    println!("Authenticate by navigating to {url}");

    let input = rpassword::prompt_password("Paste in entire URL address:")?;
    let response_url = twitch_api::twitch_oauth2::url::Url::parse(&input)?;
    let params = response_url.query_pairs().collect::<HashMap<_, _>>();
    let state = params.get("state").ok_or_eyre("missing state")?;
    let code = params.get("code").ok_or_eyre("missing code")?;
    let token = token_builder.get_user_token(&client, state, code).await?;

    Ok(token)
}
