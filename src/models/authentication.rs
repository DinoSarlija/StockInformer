use super::user::User;
use crate::diesel::result;
use chrono::Utc;
use jsonwebtoken;
use pwhash::bcrypt;

#[derive(Queryable, PartialEq, Debug, serde::Deserialize)]
pub struct AuthUser {
    pub email: String,
    pub password: String,
}

impl AuthUser {
    pub fn authenticate(
        connection: &crate::diesel::PgConnection,
        email: &str,
        password: &str,
    ) -> Result<(User, String), result::Error> {
        let user = match User::get_by_email(connection, &email.to_string()) {
            Ok(user) => user,
            Err(err) => return Err(err),
        };

        match AuthUser::verify(password.to_string(), &user) {
            Ok(_) => (),
            Err(err) => return Err(err),
        };

        let token = user.generate_jwt();

        Ok((user, token))
    }

    fn verify(password: String, user: &User) -> Result<(), result::Error> {
        if bcrypt::verify(password, &user.password) == true {
            Ok(())
        } else {
            Err(result::Error::NotFound)
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate(user: &crate::models::user::User) -> String {
    let secret = match dotenv::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => "".to_string(),
    };

    let duration = match dotenv::var("JWT_LIFETIME_IN_SECONDS") {
        Ok(d) => d,
        Err(_) => "300".to_string(),
    };

    let duration: i64 = duration.parse().unwrap();
    let exp = Utc::now() + chrono::Duration::seconds(duration);

    let claims = Claims {
        sub: String::from(&user.id),
        email: String::from(&user.email),
        exp: exp.timestamp(),
        iat: Utc::now().timestamp(),
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(&secret.as_bytes()),
    )
    .unwrap_or_default()
}

pub fn verify(token: String) -> Result<crate::models::user::User, jsonwebtoken::errors::Error> {
    let secret = match dotenv::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => "".to_string(),
    };

    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
    )?;

    Ok(crate::models::user::User::from_jwt(&token_data.claims))
}
