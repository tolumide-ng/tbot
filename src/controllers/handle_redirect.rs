use std::collections::HashMap;

use serde_json::Value;
use http::Method;
use hyper::{StatusCode, Request};
use redis::{Client as RedisClient};
use serde::{Serialize, Deserialize};
use crate::{helpers::{
    response::{TResult, ApiBody, make_request, ResponseBuilder}, 
    request::{HyperClient}, keyval::KeyVal}, 
    setup::variables::SettingsVars, errors::response::{TError}, middlewares::request_builder::RequestBuilder, interceptor::handle_request::TwitterInterceptor
};


#[derive(Debug, Clone)]
pub struct AccessToken {
    pub state: String,
    pub  code: String,
}

impl AccessToken {
    pub fn validate_state(self, local_state: String) -> TResult<Self> {
        if self.state != local_state {
            return Err(TError::InvalidCredentialError("The state value obtained from the redirect uri does not match the local one".into()));
        }

        Ok(self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppAccess {
    token_type: String,
    expires_in: i32,
    access_token: String,
    scope: String,
    refresh_token: String,
}

async fn access_token(hyper_client: HyperClient, redis_client: RedisClient, auth_code: String) -> Result<(), TError> {
    let SettingsVars{client_id, callback_url, client_secret, twitter_v2, ..} = SettingsVars::new();
    let mut con = redis_client.get_async_connection().await.unwrap();


    let req_body = KeyVal::new().add_list_keyval(vec![
        ("code".into(), auth_code.clone()),
        ("grant_type".to_string(), "authorization_code".into()),
        ("client_id".to_string(), client_id.clone()),
        ("redirect_uri".to_string(), callback_url),
        ("code_verifier".to_string(), redis::cmd("GET").arg(&["pkce"]).query_async(&mut con).await?)
    ]).to_urlencode();

    let content_type = "application/x-www-form-urlencoded";

    let request = RequestBuilder::new(Method::POST, format!("{}/oauth2/token", twitter_v2))
        .with_basic_auth(client_id, client_secret)
        .with_body(req_body, content_type).build_request();

    let (_header, body) = make_request(request, hyper_client.clone()).await?;

    // println!("WHAT DOES THIS ACTUALLY LOOK LIKE!!!!! {:#?}", String::from_utf8_lossy(&body));

    // if let Ok(_head, body) = res {
        let body: HashMap<String, Value> = serde_json::from_slice(&body).unwrap();
        let has_access_token = body.get("access_token");
        let has_refresh_token = body.get("refresh_token");

        if has_access_token.is_some() && has_refresh_token.is_some() {
            // let body_string = String::from_utf8_lossy(&body).to_string();
            println!("THE BODY STRING WITH AN OBJ!!!!!!! {:#?}", body);
            println!("!!!!!!!!!!!!!!!!!!!!!!VERIFIED!!!!!!!!!!!!!!!!!!!!!!");


            let a_t = body.get("access_token").unwrap().clone();
            let r_t = body.get("refresh_token").unwrap().clone();
            let access_token: String = serde_json::from_value(a_t).unwrap();
            let refresh_token: String = serde_json::from_value(r_t).unwrap();
            redis::cmd("SET").arg(&["access_token", &access_token]).query_async(&mut con).await?;
            redis::cmd("SET").arg(&["refresh_token", &refresh_token]).query_async(&mut con).await?;
            return Ok(())
        }

        return Err(TError::InvalidCredentialError("Required keys are not present".into()))
}



pub async fn handle_redirect(req: Request<hyper::Body>, hyper_client: HyperClient, redis_client: RedisClient) -> TResult<ApiBody> {
    let mut con = redis_client.get_async_connection().await?;
    let SettingsVars{state, api_key, ..} = SettingsVars::new();
    
    let query_params = KeyVal::query_params_to_keyval(req.uri())?;
    let is_v1_callback = query_params.verify_present(vec!["oauth_token".into(), "oauth_verifier".into()]);

    match is_v1_callback {
         Some(k) => {
            let oauth_token: String = redis::cmd("GET").arg(&["oauth_token"]).query_async(&mut con).await?;
            if k.validate("oauth_token".into(),oauth_token.clone()) {
                let verifier = k.get("oauth_verifier").unwrap();
                redis::cmd("SET").arg(&["oauth_verifier", verifier]).query_async(&mut con).await?;

                let req = RequestBuilder::new(Method::POST, "https://api.twitter.com/oauth/access_token".into())
                    .with_query("oauth_consumer_key", &api_key)
                    .with_query("oauth_token", &oauth_token)
                    .with_query("oauth_verifier", verifier)
                    .build_request();

                let res = make_request(req, hyper_client.clone()).await;

                if let Ok((_header, body)) = res {
                    let body_string = String::from_utf8_lossy(&body).to_string();
                    let params = KeyVal::string_to_keyval(body_string);

                    if let Some(map) = params {
                        redis::cmd("SET").arg(&["oauth_token", map.get("oauth_token").unwrap()]).query_async(&mut con).await?;
                        redis::cmd("SET").arg(&["oauth_token_secret", map.get("oauth_token_secret").unwrap()]).query_async(&mut con).await?;
                        redis::cmd("SET").arg(&["user_id", map.get("user_id").unwrap()]).query_async(&mut con).await?;
                        
                        return ResponseBuilder::new("Access Granted".into(), Some(""), StatusCode::OK.as_u16()).reply();
                    }
                }

            }

        }
        None => {
            // maybe it is a v2 callback
            let is_v2_callback = query_params.verify_present(vec!["code".into(), "state".into()]);

            if let Some(dict) = is_v2_callback {
                if query_params.validate("state".into(), state) {
                    let code = dict.get("code").unwrap().to_string();
                    access_token(hyper_client.clone(), redis_client, code).await?;

                    return ResponseBuilder::new("Access Granted".into(), Some(""), StatusCode::OK.as_u16()).reply();
                }
            }
        }
    }

    if query_params.verify_present(vec!["denied".into()]).is_some() {
        return ResponseBuilder::new("Unauthorized".into(), Some("Permission denied"), StatusCode::UNAUTHORIZED.as_u16()).reply()
    }

    
    ResponseBuilder::new("Bad request".into(), Some(""), StatusCode::BAD_REQUEST.as_u16()).reply()


}
