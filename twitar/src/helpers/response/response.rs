use std::collections::HashMap;

use http::{Request, HeaderMap, HeaderValue, StatusCode};
use hyper::{Response, Body};
use serde::{Serialize, Deserialize};
use serde_json::Value;


use crate::errors::response::{TError, TwitterErrors};
use crate::helpers::request::HyperClient;

// pub type ApiResponse = http::Result<Response<Body>>;

// pub type ApiResponse<T> = Result<T, anyhow::Error>;
pub type TResult<T> = std::result::Result<T, TError>;
pub type THeaders = HeaderMap<HeaderValue>;
pub type ApiBody = Response<Body>;


#[cfg(test)]
#[path = "./response.test.rs"]
mod response_test;


const X_RATE_LIMIT_RESET: &str = "X-Rate-Limit-Reset";

pub const CONTENT_TYPE: &'static str = "application/x-www-form-urlencoded";


pub async fn make_request(request: Request<Body>, client: HyperClient) -> TResult<(THeaders, Vec<u8>)> {
    let res: Response<Body> = client.request(request).await.unwrap();
    
    let (parts, body) = res.into_parts();
    let body = hyper::body::to_bytes(body).await?.to_vec();

    // println!("WHAT THE ERROR IS LIKE \n\n\n {:#?} \n\n\n", String::from_utf8_lossy(&body));
    // println!("THE PARTS {:#?}", parts);
    
    if let Ok(errors) = serde_json::from_slice::<TwitterErrors>(&body) {
        // println!("THE LOOPED ERROR SETS");
        if errors.errors.iter().any(|e| e.code == 88)
        && parts.headers.contains_key(X_RATE_LIMIT_RESET) {
            return Err(TError::RateLimit())
        } else {
            return Err(TError::TwitterError(parts.headers, errors))
        }
    }

    if !parts.status.is_success() {
        // println!("THERE WAS AN ISSUE!!!");
        return Err(TError::BadStatus(parts.status))
    }

    // println!("THIS WAS A SUCCESS {:#?}", parts);
  
    Ok((parts.headers, body))
}



#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ResponseBuilder<T: Serialize> {
    message: String,
    body: Option<T>,
    code: u16,
}

impl<T> ResponseBuilder<T> where T: Serialize {
    pub fn new(message: String, body: Option<T>, code: u16) -> Self {
        Self {message, body, code}
    }

    fn make_body(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    // pub fn reply_err(errs: &[(TError, &'static str, Option<StatusCode>)]) {}

    pub fn reply(self) -> TResult<ApiBody> {
        let body = Body::from(self.make_body());
        let code = StatusCode::from_u16(self.code)?;
        let response = Response::builder().status(code).body(body).unwrap();

        Ok(response)
    }
}




#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterResponseData {
    data: Vec<HashMap<String, String>>,
    meta: HashMap<String, Value>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterResponseHashData {
    data: HashMap<String, String>,
}


impl TwitterResponseHashData {
        // Only use for responses whose vector (also called array) cotnains only one hashmap (also called objects)
    pub fn into_one_dict(self) -> HashMap<String, String> {
        let mut dict: HashMap<String, String> = HashMap::new();

        for key in self.data.keys() {
            // dict.extend(self.data.iter());
            dict.insert(key.into(), self.data.get(key).unwrap().into());
        };
        return dict
    }
}



impl TwitterResponseData {
    pub fn separate_tweets_from_rts(&self, exclude_head: bool) -> HashMap<String, Vec<String>>{
        let mut dict = HashMap::new();
        let mut tweets = vec![];
        let mut rts = vec![];

        let mut start = 0;

        if exclude_head {
            start = 11;
        }

        for num in start..self.data.len() {
            let tweet = &self.data[num];
            let id = tweet.get("id").unwrap().clone();
            if tweet.get("text").unwrap().starts_with("RT") {
                rts.push(id);
            } else {
                tweets.push(id)
            }
        }

        dict.insert("tweets".into(), tweets);
        dict.insert("rts".into(), rts);

        dict
    }

    pub fn parse_metadata(&self) -> HashMap<String, String> {
        let keys = self.meta.keys().cloned().collect::<Vec<String>>();
        let map = keys.iter()
            .map(|k,| (k.to_string(), self.meta.get(k).unwrap().to_string())).collect::<HashMap<_, _>>();
        
        return map;
    }

    // Gets ids from a vector of dictionaries containing "id" and "text"
    pub fn get_ids(&self) -> Vec<String> {
        self.data.iter().map(|map| map.get("id").unwrap().clone()).collect::<Vec<_>>()
    }
}



