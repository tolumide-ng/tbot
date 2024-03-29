use std::{collections::HashMap, borrow::Cow};
use derive_more;
use hyper::{Uri};
use url::{Url};

use crate::{helpers::response::TResult};

#[cfg(test)]
#[path = "./keyval.test.rs"]
mod keyval_test;

#[derive(Debug, derive_more::Deref, derive_more::DerefMut, derive_more::From, Clone, Default)]
pub struct KeyVal(HashMap<Cow<'static, str>, Cow<'static, str>>);



impl KeyVal {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_keyval(mut self, key: String, val: String) -> Self {
        self.insert(key.into(), val.into());
        self
    }

    pub fn new_with_keyval(key: String, val: String) -> Self {
        let mut dict = Self::new();
        dict.insert(key.into(), val.into());

        dict
    }

    pub fn add_list_keyval(mut self, list: Vec<(String, String)>) -> Self {
        for (k, v) in list {
            self.insert(k.into(), v.into());
        }
        self
    }

    pub fn query_params_to_keyval(uri: &Uri) -> TResult<Self> {
        let mut uri_string = uri.to_string();

        if !uri_string.starts_with("https:/") {
            uri_string = format!("https:/{}", uri_string);
        }

        let parsed_uri = Url::parse(&uri_string)?;

        let mut dic = Self::new();

        if let Some(all_qs) = parsed_uri.query() {
            let params: Vec<&str> = all_qs.split("&").collect();

            for param in params {
                let vec_param = param.split("=").collect::<Vec<_>>();
                dic = dic.add_keyval(vec_param[0].into(), vec_param[1].into());
            }
        }

        Ok(dic)
    }

    pub fn string_to_keyval(s: String) -> Option<Self> {
        // let valid = s.split("");
        let ampersand: Vec<&str> = s.matches("&").collect();
        let equals_sign: Vec<&str> = s.matches("=").collect();

        if ampersand.len() + 1 != equals_sign.len() {
            return None
        }

        let mut dic = Self::new();

        let params: Vec<&str> = s.split("&").collect();

        for param in params {
            // let pair_string = pair.to_string();
            let k_v = param.split("=").collect::<Vec<_>>();
            dic = dic.add_keyval(k_v[0].into(), k_v[1].into());
        }

        Some(dic)

    }

    pub fn to_urlencode(&self) -> String {
        self.iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>().join("&")
    }
    

    pub fn validate(&self, name: String, value: String) -> bool {
        if let Some(obtained_value) = &self.get(name.as_str()) {
            if obtained_value.to_string() == value {
                // return Ok("akld".into())
                return true;
            }

            return false;
        }

        return false;
    }

    pub fn every(&self, names: Vec<String>) -> Option<&Self> {
        let keys = self.keys().cloned().map(|k| k.to_string()).collect::<Vec<String>>();

        for index in 0..names.len() {
            if !keys.contains(&names[index]) {
                return None;
            }
        }
        
        Some(&self)
    }

    // pub fn get_from(self, name: String) -> String {
    //     return self.get(name.as_str()).unwrap()
    // }

    // pub fn validate_multiple(&self, values: Vec<String>) {
    //     let mut errors: Vec<String> = vec![];

    //     for value in values {
    //         // if let Some(current_value)
    //     }
    // }
}
