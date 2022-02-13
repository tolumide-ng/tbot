use hyper::{StatusCode, Method};

use crate::{
    helpers::{response::{TResult, ApiBody, ResponseBuilder, make_request, TwitterResponseVecData}}, 
    middlewares::request_builder::{RequestBuilder, AuthType}, interceptor::handle_request::Interceptor, setup::variables::SettingsVars, app::server::AppState
};


const MAX_TWEETS: &'static str = "100";

pub async fn get_timeline(app_state: AppState) -> TResult<ApiBody> {
    let AppState {redis, hyper, env_vars, ..} = app_state;
    let SettingsVars { twitter_url, ..} = env_vars;
    
    let mut con = redis.get_async_connection().await?;

    let user_id: String = redis::cmd("GET").arg(&["userid"]).query_async(&mut con).await?;
    let access_token: String = redis::cmd("GET").arg(&["access_token"]).query_async(&mut con).await?;

    let get_tweets_and_rts = RequestBuilder::new
        (Method::GET, format!("{}/2/users/{}/tweets", twitter_url, user_id))
        .with_query("max_results", MAX_TWEETS)
        .with_auth(AuthType::Bearer, access_token.clone()).build_request();

    let get_likes = RequestBuilder::new(Method::GET, format!("{}/2/users/{}/liked_tweets", twitter_url, user_id))
        .with_auth(AuthType::Bearer, access_token).build_request();

    let res = Interceptor::intercept(make_request(get_tweets_and_rts, hyper).await);

    if let Err(e) = res {
        return ResponseBuilder::new("Error".into(), Some(e.0), e.1).reply()
    }

    let body: TwitterResponseVecData = serde_json::from_value(res.unwrap()).unwrap();

    let parsed = body.separate_tweets_from_rts(true);

    ResponseBuilder::new("Ok".into(), Some(parsed), StatusCode::OK.as_u16()).reply()
}