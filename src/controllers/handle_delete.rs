use std::{collections::HashMap, io::Read};
use http::Method;
use hyper::{Body, Request};
use redis::{Client as RedisClient};
use futures::{stream, StreamExt};
use serde_json::Value;
use tokio;

use crate::{helpers::{request::HyperClient, response::{TResult, ApiBody, ResponseBuilder, make_request}}, middlewares::request_builder::RequestBuilder};

type Ids = HashMap<String, Vec<String>>;

#[derive(Clone, Debug)]
enum TweetType {
    Tweets,
    Rts,
}

impl std::fmt::Display for TweetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tweets => write!(f, "tweets"),
            Self::Rts => write!(f, "rts"),
        }
    }
}

// type ObjectStr = HashMap<String, String>;
#[derive(Debug, Clone)]
struct PostIds(Vec<(String, TweetType)>);


// FIND A BETTER WAY TO HANDLE THIS PARSING, SO THE CODE IS MORE READABLE AND EASIER TO FOLLOW
impl PostIds {
    pub fn parse(s: Ids) -> Self {
        let received_keys = s.keys().cloned().collect::<Vec<String>>();
        let expected_keys = [TweetType::Rts, TweetType::Tweets];

        if received_keys.contains(&TweetType::Rts.to_string()) && received_keys.contains(&&TweetType::Tweets.to_string()) {
            let mut total = s.get(&TweetType::Rts.to_string()).unwrap().len() + s.get(&TweetType::Tweets.to_string()).unwrap().len();

            if total > 50 {
                panic!("Total tweets and rts cannot be more than 50")
            }

            let mut all_ids: Vec<(String, TweetType)> = vec![];

            for key in expected_keys {
                let ids = s.get(&key.to_string()).unwrap();

                let duplicates = ids.iter()
                    .find(|x| ids.iter().filter(|y| x == y).count() >= 2);

                let empty_string = ids.iter().find(|x| x.len() < 1);

                if duplicates.is_some() || empty_string.is_some() {
                    // detail = "Ids must contains atleast one post id"
                    panic!("{} must be an array of ids (string type) or an empty array", key)
                }

                // all_ids.push(());
                let ids = s.get(&key.to_string()).unwrap()
                    .iter().map(|k| (*k, key)).collect::<Vec<(String, TweetType)>>();

                all_ids.extend(ids);



            }
            return Self(all_ids)
        }

        panic!("request object must contain rts and tweets with an array of string as values")
    }
}


// rename this module to destory which then contains destory RTs and destory Posts
pub async fn handle_delete(request: Request<Body>, hyper_client: HyperClient, redis_client: RedisClient) -> TResult<ApiBody> {
    let mut con = redis_client.get_async_connection().await?;
    let access_token: String = redis::cmd("GET").arg(&["access_token"]).query_async(&mut con).await?;

    // req body for the ids must be a vector of strings(id of tweets)
    let req_body = request.into_body();
    let  byte_body = hyper::body::to_bytes(req_body).await?.to_owned();
    let body: Ids = serde_json::from_slice(&byte_body)?;

    let post_ids = PostIds::parse(body).0;
    let parallel_requests = post_ids.len();

    let bodies = stream::iter(post_ids)
        .map(|id: (String, TweetType)| {
            let client = hyper_client.clone();
            let token = access_token.clone();

            // let mut api_id 

            tokio::spawn(async move {
                let request = RequestBuilder::new(Method::DELETE, format!("/tweets/{}", id))
                    .with_access_token("Bearer", token).build_request();

                println!("THE REQUEST {:#?}", request);

                let response = make_request(request, client).await.unwrap();
                println!("THE HEADER {:#?}", response.0);
                // Ok(response.1);
                // Ok(body)
                response.1
                // response.1.bytes().await
                // HANDLE RESPONSE HERE ---- IT IS TIME TO CREATE THE INTERCEPTOR THAT HANDLES THE RESPONSE OF THE MAKE_REQUEST CALL
                // let res = make_request(request, client);
            })
        }).buffer_unordered(parallel_requests);

    bodies
        .for_each(|res| async {
            match res {
                Ok(body) => {
                    let body: Value = serde_json::from_slice(&body).unwrap();
                    println!("THE BODY {:#?}", body)
                }
                Err(e) => {
                    eprintln!("ERROR {:#?}", e)
                }
            }
        }).await;




    
    // let req = RequestBuilder::new(Method::POST, format!("https://api.twitter.com/1.1/statuses/destroy/{}.json"))
    return ResponseBuilder::new("Ok".into(), Some(""), 200).reply();
}