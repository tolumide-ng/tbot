use anyhow::Context;
use sqlx::{Error::RowNotFound, Pool, Postgres, Transaction};
use uuid::Uuid;
use crate::helpers::db_helper::{AllTweetIds, TweetIds, TweetType};
use crate::helpers::{response::TResult};
use crate::errors::response::TError;

#[derive(Debug)]
pub struct DB;

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub v1_active: bool,
    pub v2_active: bool,
}

#[derive(Debug)]
pub struct V2User {
    pub id: i32,
    pub user_id: Uuid,
    pub twitter_user_id: Option<String>,
    pub pkce: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug)]
pub struct V1User {
    pub id: i32,
    pub user_id: Uuid,
    pub twitter_user_id: Option<String>,
    pub oauth_token: String,
    pub oauth_secret: String,
    pub oauth_verifier: Option<String>,
}


impl DB {
    pub async fn add_v2_user(pool: &Pool<Postgres>, user_id: Uuid) {
        let user = sqlx::query!(r#"INSERT INTO auth_two (user_id) VALUES ($1) RETURNING user_id"#, user_id).fetch_one(pool).await;

        if let Err(e) = user {
            println!("THE EXPERIENCED ERROR {:#?}", e);
        }
        // 
    }

    pub async fn add_v1_user(pool: &Pool<Postgres>, user_id: Uuid) {
        let user = sqlx::query!(
            r#"INSERT INTO auth_one (user_id) VALUES ($1) RETURNING user_id"#, user_id)
            .fetch_one(pool).await;

        if let Err(_e) = user {
            // 
        }
        // 
    }

    pub async fn update_pkce(pool: &Pool<Postgres>, pkce: &str, user_id: Uuid) -> TResult<()> {
        sqlx::query(r#"UPDATE auth_two SET pkce=$1 WHERE user_id=$2 RETURNING *"#)
            .bind(pkce)
            .bind(user_id)
            .execute(&*pool).await?;

        Ok(())
    }

    pub async fn insert_tweet_ids<'a>(pool: &Pool<Postgres>, user_id: Uuid, all_tweets: AllTweetIds<'a>) -> TResult<()> {
        let mut transaction = pool.begin().await.context("Unable to acquire db pool connection")?;

        let original_tweets = all_tweets.get_tweets();
        DB::save_ids(&mut transaction, original_tweets, user_id, TweetType::Tweets).await?;
        let likes = all_tweets.get_likes();
        DB::save_ids(&mut transaction, likes, user_id, TweetType::Likes).await?;
        let rts = all_tweets.get_rts();
        DB::save_ids(&mut transaction, rts, user_id, TweetType::Rts).await?;

        transaction.commit().await.context("Failed to commit SQL transaction to save all tweet ids")?;


        Ok(())
    }

    async fn save_ids<'a>(transaction: &mut Transaction<'_, Postgres>, ids: &TweetIds<'a>, user_id: Uuid, tweet_type: TweetType) -> TResult<()>{
        for id_vec in ids {
            let the_ids: Vec<&str> = id_vec.iter().map( |x| {x.as_str()}).collect();
            sqlx::query(r#"INSERT INTO play_tweets (user_id, tweet_type, tweet_ids) VALUES ($1, $2, $3)"#)
                .bind(user_id)
                .bind(tweet_type.to_string())
                .bind(the_ids)
                .execute(&mut *transaction).await?;
        }

        Ok(())
    }

    pub async fn user_exists(pool: &Pool<Postgres>, user_id: Uuid) -> TResult<Option<AuthUser>> {
        let user = sqlx::query_as!(
            AuthUser, 
            r#"SELECT * FROM user_preference WHERE (user_id = $1)"#, user_id
        )
            .fetch_one(pool)
            .await;

        if let Err(e) = user {
            return match e {
                RowNotFound => Ok(None),
                _ => {Err(TError::DatabaseError(e))}
            }
        }

        Ok(Some(user.unwrap()))
    }

    pub async fn v2_user(pool: &Pool<Postgres>, user_id: Uuid) -> TResult<Option<V2User>> {
        let user = sqlx::query_as!(
            V2User, 
            r#"SELECT * FROM auth_two WHERE (user_id = $1)"#, user_id
        )
            .fetch_one(pool)
            .await;

        if let Err(e) = user {
            return match e {
                RowNotFound => Ok(None),
                _ => {Err(TError::DatabaseError(e))}
            }
        }

        Ok(Some(user.unwrap()))
    }

     pub async fn v1_user(pool: &Pool<Postgres>, user_id: Uuid) -> TResult<Option<V1User>> {
         let user = sqlx::query_as!(
             V1User, 
             r#"SELECT * FROM auth_one WHERE (user_id = $1)"#, user_id
            ).fetch_one(pool).await;

        if let Err(e) = user {
            println!("THERE WAS AN ERROR::::::: {:#?}", e);
            return match e {
                RowNotFound => Ok(None),
                _ => {Err(TError::DatabaseError(e))}
            }
        }

        println!("THE V1 USER TO OBTAIN:::::::::::::::::::::::::::::::::::___________________::::::::::::::::::::::::::::::::::: {:#?}", user);

        Ok(Some(user.unwrap()))
    }

    pub async fn update_secets(pool: &Pool<Postgres>, access_token: String, refresh_token: String, user_id: Uuid) -> TResult<()> {
        sqlx::query(r#"UPDATE auth_two SET access_token=$1, refresh_token=$2 WHERE user_id=$3 RETURNING *"#)
            .bind(access_token)
            .bind(refresh_token)
            .bind(user_id)
            .execute(&*pool).await?;

        Ok(())
    }

    pub async fn update_v1_secets(pool: &Pool<Postgres>, oauth_token: String, oauth_secret: String, user_id: Uuid) -> TResult<()> {
        sqlx::query(r#"UPDATE auth_one SET oauth_token=$1, oauth_secret=$2 WHERE user_id=$3 RETURNING *"#)
            .bind(oauth_token)
            .bind(oauth_secret)
            .bind(user_id)
            .execute(&*pool).await?;

        Ok(())
    }


    pub async fn add_oauth_verifier(pool: &Pool<Postgres>, oauth_verifier: &str, user_id: Uuid) -> TResult<()>{
        sqlx::query(r#"UPDATE auth_one SET oauth_verifier=$1 WHERE user_id=$2 RETURNING *"#)
            .bind(oauth_verifier)
            .bind(user_id)
            .execute(&*pool).await?;

        Ok(())
    }

     pub async fn create_v1_secets(pool: &Pool<Postgres>, user_id: Uuid, oauth_token: String, oauth_secret: String) -> TResult<()> {
         if DB::v1_user(pool, user_id).await?.is_some() {
             sqlx::query(r#"DELETE FROM auth_one WHERE user_id=$1"#).bind(user_id).execute(&*pool).await?;
         }

        sqlx::query(r#"INSERT INTO auth_one (user_id, oauth_token, oauth_secret) VALUES ($1, $2, $3) RETURNING user_id"#)
            .bind(user_id)
            .bind(oauth_token)
            .bind(oauth_secret)
            .execute(&*pool).await?;

        Ok(())
    }

    // let user = sqlx::query!(r#"INSERT INTO auth_two (user_id) VALUES ($1) RETURNING user_id"#, user_id).fetch_one(pool).await;

    pub async fn update_twitter_id(pool: &Pool<Postgres>, twitter_user_id: &str, user_id: Uuid) -> TResult<()> {
        sqlx::query(r#"UPDATE auth_two SET twitter_user_id=$1 WHERE user_id=$2"#)
            .bind(twitter_user_id)
            .bind(user_id)
            .execute(&*pool).await?;
            
        Ok(())
    }
}