use crate::models::authentication::Claims;
use crate::schema::users;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result;
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Queryable, PartialEq, Insertable, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub is_deleted: bool,
}

impl User {
    pub fn new(email: String, password: String) -> User {
        User {
            id: Uuid::new_v4().to_string(),
            email,
            password,
            is_deleted: false,
        }
    }

    pub fn get_all(connection: &PgConnection) -> Result<Vec<User>, result::Error> {
        users::table.load::<User>(connection)
    }

    pub fn get_by_id(connection: &PgConnection, id: &String) -> Result<User, result::Error> {
        match users::table
            .filter(users::id.eq(id))
            .get_result::<User>(connection)
        {
            Ok(result) => Ok(result),
            Err(err) => Err(err),
        }
    }

    pub fn get_by_email(connection: &PgConnection, email: &String) -> Result<User, result::Error> {
        match users::table
            .filter(users::email.eq(email))
            .load::<User>(connection)
        {
            Ok(mut results) => match results.pop() {
                Some(user) => Ok(user),
                None => Err(result::Error::NotFound),
            },
            Err(err) => Err(err),
        }
    }

    pub fn update_email(
        self,
        connection: &PgConnection,
        email: String,
    ) -> Result<User, result::Error> {
        match diesel::update(users::table.find(self.id))
            .set(users::email.eq(email))
            .get_result::<User>(connection)
        {
            Ok(user) => Ok(user),
            Err(err) => Err(err),
        }
    }

    pub fn update_password(
        self,
        connection: &PgConnection,
        password: String,
    ) -> Result<(), result::Error> {
        let hash_password = match bcrypt::hash(password) {
            Ok(hashed) => Ok(hashed),
            Err(err) => Err(err),
        };

        match diesel::update(users::table.find(self.id))
            .set(users::password.eq(hash_password.unwrap()))
            .get_result::<User>(connection)
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn delete_user(connection: &PgConnection, user_id: &String) -> Result<User, result::Error> {
        match diesel::update(users::table.find(user_id))
            .filter(users::is_deleted.eq(false))
            .set(users::is_deleted.eq(true))
            .get_result::<User>(connection)
        {
            Ok(user) => Ok(user),
            Err(err) => Err(err),
        }
    }

    pub fn generate_jwt(&self) -> String {
        crate::models::authentication::generate(&self)
    }

    pub fn from_jwt(claims: &Claims) -> Self {
        User {
            id: String::from(&claims.sub),
            email: String::from(&claims.email),
            password: String::new(),
            is_deleted: false,
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct NewUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

impl NewUser {
    pub fn create(
        email: String,
        password: String,
        connection: &PgConnection,
    ) -> Result<User, result::Error> {
        let hash_password = match bcrypt::hash(password) {
            Ok(hashed) => hashed,
            Err(err) => return Err(result::Error::__Nonexhaustive),
        };
        let user: User = User::new(email.clone(), hash_password);

        match User::get_by_email(&connection, &email) {
            Ok(_) => Err(result::Error::__Nonexhaustive),
            Err(_) => diesel::insert_into(users::table)
                .values(&user)
                .get_result::<User>(connection),
        }
    }
}
