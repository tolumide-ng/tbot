use hyper::{Method, Body, Response};

use crate::base_repository::db::DB;
use crate::startup::server::AppState;
use crate::helpers::response::ApiBody;
use crate::helpers::{
    response::{TResult},
    gen_pkce::Pkce,
    scope::Scope,
    keyval::KeyVal,
};
use crate::configurations::{variables::SettingsVars};
use crate::middlewares::request_builder::RequestBuilder;


pub async fn authorize_bot(app_state: AppState) -> TResult<ApiBody> {
    let SettingsVars {client_id, callback_url, state, ..} = app_state.env_vars;    
    
    let pkce: String = Pkce::new().to_string();
    let scopes = vec![Scope::ReadTweet, Scope::ReadUsers, Scope::ReadFollows, Scope::WriteFollows, 
    Scope::OfflineAccess, Scope::WriteTweet, Scope::WriteLike, Scope::ReadLike];
    
    DB::update_pkce(&app_state.db_pool, &pkce, app_state.user.unwrap().v2_user.user_id).await?;

    // use uuid::Uuid;

    let query_params = KeyVal::new()
        .add_list_keyval(vec![
            ("response_type".to_string(), "code".to_string()),
            ("client_id".to_string(), client_id),
            ("redirect_uri".to_string(), callback_url),
            ("scope".to_string(), Scope::with_scopes(scopes)),
            ("state".to_string(), state),
            ("code_challenge".to_string(), pkce),
            ("code_challenge_method".to_string(), "plain".to_string()),
        ]);

    let request = RequestBuilder::new(Method::GET, "https://twitter.com/i/oauth2/authorize".into())
        .add_query_params(query_params)
        .build_request();

    println!("THE REQUEST {:#?}", request);

    let response_body= Response::builder().status(302)
        .header("Location", request.uri().to_string())
        .body(Body::from(request.uri().to_string())).unwrap();

    Ok(response_body)
}