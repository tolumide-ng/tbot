use http::{Method, StatusCode, Request};
use hyper::Body;
use redis::{Client as RedisClient};


use crate::{helpers::{
    request::HyperClient, 
    response::{
        ResponseBuilder, TResult, ApiBody, make_request, TwitterResponseHashData}
    }, middlewares::request_builder::RequestBuilder, interceptor::handle_request::TwitterInterceptor, setup::variables::SettingsVars
};


// use this endpoint to verify the validity of the username when they want to request for their timeline
pub async fn user_lookup(request: Request<Body>, hyper_client: HyperClient, redis_client: RedisClient) -> TResult<ApiBody> {
    // todo!() move this to params once route management is migrated to routerify
    let SettingsVars {twitter_v2, ..} = SettingsVars::new();
    
    let username = request.uri().query().unwrap().split("=").collect::<Vec<_>>()[1];
    let mut con = redis_client.get_async_connection().await?;

    let access_token = redis::cmd("GET").arg(&["access_token"]).query_async(&mut con).await?;

    let req = RequestBuilder::new(Method::GET, format!("{}/users/by/username/{}", twitter_v2, username))
        .with_access_token("Bearer", access_token).build_request();

    let res= TwitterInterceptor::intercept(make_request(req, hyper_client.clone()).await);

    if let Err(e) = res {
        return ResponseBuilder::new("Error".into(), Some(e.0), e.1).reply();
    }

    let body: TwitterResponseHashData = serde_json::from_value(res.unwrap()).unwrap();
    let user = body.into_one_dict();
    let user_id = user.get("id").unwrap();

    redis::cmd("SET").arg(&["tolumide_userid", &user_id]).query_async(&mut con).await?;
    ResponseBuilder::new("Ok".into(), Some(""), StatusCode::OK.as_u16()).reply()
}