use crate::schema::portfolios;

use chrono::{DateTime, NaiveDate, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, PartialEq, Insertable, Serialize, Deserialize, Debug)]
#[table_name = "portfolios"]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub created_at: NaiveDate,
    pub is_deleted: bool,
    pub user_id: String,
}

impl Portfolio {
    pub fn new(name: String, user_id: String) -> Portfolio {
        let utc: DateTime<Utc> = Utc::now();
        let naive_date = utc.naive_utc().date();
        Portfolio {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: naive_date,
            is_deleted: false,
            user_id,
        }
    }

    pub fn get_all_from_user(
        connection: &PgConnection,
        user_id: &String,
    ) -> Result<Vec<Portfolio>, result::Error> {
        portfolios::table
            .filter(portfolios::user_id.eq(user_id))
            .filter(portfolios::is_deleted.eq(false))
            .load::<Portfolio>(connection)
    }

    pub fn get_by_id(connection: &PgConnection, id: String) -> Result<Portfolio, result::Error> {
        match portfolios::table
            .filter(portfolios::id.eq(id))
            .filter(portfolios::is_deleted.eq(false))
            .get_result::<Portfolio>(connection)
        {
            Ok(result) => Ok(result),
            Err(err) => Err(err),
        }
    }

    pub fn get_by_name(
        connection: &PgConnection,
        name: &String,
        user_id: &String,
    ) -> Result<Option<Portfolio>, result::Error> {
        match portfolios::table
            .filter(portfolios::user_id.eq(user_id))
            .filter(portfolios::name.eq(name))
            .filter(portfolios::is_deleted.eq(false))
            .load::<Portfolio>(connection)
        {
            Ok(mut results) => Ok(results.pop()),
            Err(err) => Err(err),
        }
    }

    pub fn update_name(
        self,
        connection: &PgConnection,
        name: String,
    ) -> Result<Portfolio, result::Error> {
        match diesel::update(portfolios::table.find(self.id))
            .filter(portfolios::is_deleted.eq(false))
            .set(portfolios::name.eq(name))
            .get_result::<Portfolio>(connection)
        {
            Ok(portfolio) => Ok(portfolio),
            Err(err) => Err(err),
        }
    }

    pub fn delete_portfolio(
        connection: &PgConnection,
        portfolio_id: &String,
    ) -> Result<Portfolio, result::Error> {
        match diesel::update(portfolios::table.find(portfolio_id))
            .filter(portfolios::is_deleted.eq(false))
            .set(portfolios::is_deleted.eq(true))
            .get_result::<Portfolio>(connection)
        {
            Ok(portfolio) => Ok(portfolio),
            Err(err) => Err(err),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewPortfolio {
    pub name: String,
    pub user_id: String,
}

impl NewPortfolio {
    pub fn create(
        name: String,
        user_id: String,
        connection: &PgConnection,
    ) -> Result<Portfolio, result::Error> {
        let portfolio: Portfolio = Portfolio::new(name.clone(), user_id.clone());

        match Portfolio::get_by_name(connection, &name, &user_id).unwrap() {
            Some(_) => Err(result::Error::__Nonexhaustive),
            None => diesel::insert_into(portfolios::table)
                .values(&portfolio)
                .get_result::<Portfolio>(connection),
        }
    }
}
