use crate::schema::tickers;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, PartialEq, Insertable, Serialize, Deserialize, Debug, Clone)]
#[table_name = "tickers"]
pub struct Ticker {
    pub id: String,
    pub name: String,
    pub portfolio_id: String,
    pub is_deleted: bool,
}

impl Ticker {
    pub fn new(name: String, portfolio_id: String) -> Ticker {
        Ticker {
            id: Uuid::new_v4().to_string(),
            name,
            portfolio_id,
            is_deleted: false,
        }
    }

    pub fn get_all_from_portfolio(
        connection: &PgConnection,
        portfolio_id: String,
    ) -> Result<Vec<Ticker>, result::Error> {
        tickers::table
            .filter(tickers::portfolio_id.eq(portfolio_id))
            .filter(tickers::is_deleted.eq(false))
            .load::<Ticker>(connection)
    }

    pub fn get_by_id(
        connection: &PgConnection,
        ticker_id: &String,
    ) -> Result<Option<Ticker>, result::Error> {
        match tickers::table
            .filter(tickers::portfolio_id.eq(ticker_id))
            .filter(tickers::is_deleted.eq(false))
            .load::<Ticker>(connection)
        {
            Ok(mut results) => Ok(results.pop()),
            Err(err) => Err(err),
        }
    }

    pub fn get_by_name(
        connection: &PgConnection,
        name: &String,
        portfolio_id: &String,
    ) -> Result<Option<Ticker>, result::Error> {
        match tickers::table
            .filter(tickers::portfolio_id.eq(portfolio_id))
            .filter(tickers::name.eq(name))
            .filter(tickers::is_deleted.eq(false))
            .load::<Ticker>(connection)
        {
            Ok(mut results) => Ok(results.pop()),
            Err(err) => Err(err),
        }
    }

    pub fn delete_ticker(
        connection: &PgConnection,
        ticker_id: &String,
    ) -> Result<Ticker, result::Error> {
        match diesel::update(tickers::table.find(ticker_id))
            .filter(tickers::is_deleted.eq(false))
            .set(tickers::is_deleted.eq(true))
            .get_result::<Ticker>(connection)
        {
            Ok(ticker) => Ok(ticker),
            Err(err) => Err(err),
        }
    }

    pub fn delete_tickers(
        connection: &PgConnection,
        portfolio_id: &String,
    ) -> Result<Vec<Ticker>, result::Error> {
        match diesel::update(tickers::table.filter(tickers::portfolio_id.eq(portfolio_id)))
            .filter(tickers::is_deleted.eq(false))
            .set(tickers::is_deleted.eq(true))
            .get_results::<Ticker>(connection)
        {
            Ok(ticker) => Ok(ticker),
            Err(err) => Err(err),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewTicker {
    pub name: String,
    pub portfolio_id: String,
}

impl NewTicker {
    pub fn create(
        name: String,
        portfolio_id: String,
        connection: &PgConnection,
    ) -> Result<Ticker, result::Error> {
        let ticker: Ticker = Ticker::new(name.clone(), portfolio_id.clone());

        match Ticker::get_by_name(connection, &name, &portfolio_id).unwrap() {
            Some(_) => Err(result::Error::__Nonexhaustive),
            None => diesel::insert_into(tickers::table)
                .values(&ticker)
                .get_result::<Ticker>(connection),
        }
    }
}
