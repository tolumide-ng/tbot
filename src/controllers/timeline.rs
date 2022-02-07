use http::{Request, StatusCode, Method};
use hyper::Body;
use redis::{Client as RedisClient};
use serde_json::Value;

use crate::{helpers::{request::HyperClient, response::{TResult, ApiBody, ResponseBuilder, make_request}}, middlewares::request_builder::RequestBuilder, errors::twitter_errors::TwitterResponseError, interceptor::handle_request::TwitterInterceptor};




pub async fn get_timeline(request: Request<Body>, hyper_client: HyperClient, redis_client: RedisClient) 
 -> TResult<ApiBody>
{
    let mut con = redis_client.get_async_connection().await?;
    
    let user_id: String = redis::cmd("GET").arg(&["tolumide_userid"]).query_async(&mut con).await?;
    let access_token = redis::cmd("GET").arg(&["tolumide_test_access"]).query_async(&mut con).await?;

    let req = RequestBuilder::new
        (Method::GET, format!("https://api.twitter.com/2/users/{}/tweets", user_id))
        .with_query("max_results", "100")
        .with_access_token(access_token).build_request();

    let res = TwitterInterceptor::intercept(make_request(req, hyper_client.clone()).await);

    if let Err(e) = res {
        return ResponseBuilder::new("Error".into(), Some(e.0), e.1).reply()
    }

    let parsed = res.unwrap().separate_tweets_from_rts(true);

    println!("PARSED \n\n {:#?} \n\n ", parsed);

    ResponseBuilder::new("Ok".into(), Some(""), StatusCode::OK.as_u16()).reply()
}