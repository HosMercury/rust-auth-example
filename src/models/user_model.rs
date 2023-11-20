use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: Secret<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertUser {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub updated_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_login: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: Secret<String>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub updated_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::iso8601::option")]
    pub last_login: Option<OffsetDateTime>,
}

impl User {
    pub async fn get(pool: Pool<Postgres>, id: Uuid) -> GetUser {
        sqlx::query_as!(
            GetUser,
            "SELECT id, username, email, created_at, updated_at, last_login FROM users WHERE id = $1",
            id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
        })
        .expect("Failed to execute query")
    }

    pub async fn all(pool: Pool<Postgres>) -> Vec<GetUser> {
        sqlx::query_as!(
            GetUser,
            "SELECT id, username, email, created_at, updated_at, last_login FROM users"
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
        })
        .expect("Failed to execute query")
    }

    pub async fn create(pool: Pool<Postgres>, payload: UpsertUser) {
        let UpsertUser {
            email,
            username,
            password,
        } = payload;

        // salting and hashing
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("salting error")
            .to_string();

        sqlx::query!(
            "INSERT INTO users (id, email, username, password) VALUES ( $1, $2, $3 , $4) returning id",
            Uuid::new_v4(),
            email,
            username,
            password_hash        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
        })
        .expect("Failed to execute query");
    }

    pub async fn update(pool: Pool<Postgres>, payload: UpsertUser, id: Uuid) {
        let UpsertUser {
            email,
            username,
            password,
        } = payload;

        sqlx::query!(
            "UPDATE users SET email = $1, username = $2, password = $3 WHERE id = $4 RETURNING id",
            email,
            username,
            password,
            id
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
        })
        .expect("Failed to execute query");
    }

    pub async fn login(pool: Pool<Postgres>, payload: LoginUser) {
        let result = sqlx::query_as!(
            LoginUser,
            "SELECT username, password FROM users WHERE username = $1",
            payload.username
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
        })
        .expect("Failed to execute query")
        .unwrap();

        let parsed_hash: PasswordHash<'_> =
            PasswordHash::new(&result.password.expose_secret()).unwrap();

        if Argon2::default()
            .verify_password(payload.password.expose_secret().as_bytes(), &parsed_hash)
            .is_ok()
        {
            println!("sucessss");
        } else {
            println!("failed");
        }
    }
}