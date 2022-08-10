use crate::infrastructure;
use crate::models::authentication::AuthUser;
use crate::models::portfolio::NewPortfolio;
use crate::models::portfolio::Portfolio;
use crate::models::ticker::NewTicker;
use crate::models::ticker::Ticker;
use crate::models::user::NewUser;
use crate::models::user::User;
use actix_web::{web, HttpResponse, Responder};
use chrono::prelude::*;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use yahoo::{Dividend, Quote, YahooError};
use yahoo_finance_api as yahoo;

pub async fn get_all_users(data: web::Data<infrastructure::state::AppState>) -> impl Responder {
    match crate::models::user::User::get_all(&data.get_connection()) {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

pub async fn register(
    data: web::Data<infrastructure::state::AppState>,
    new_user: web::Json<NewUser>,
) -> impl Responder {
    if verify_email(new_user.email.clone()) == true {
        if verify_password(new_user.password.clone()) == true {
            match crate::models::user::NewUser::create(
                new_user.email.clone(),
                new_user.password.clone(),
                &data.get_connection(),
            ) {
                Ok(user) => HttpResponse::Created().json(user),
                Err(_) => HttpResponse::BadRequest().body("User already exists"),
            }
        } else {
            HttpResponse::BadRequest().body("Password is not strong enough")
        }
    } else {
        HttpResponse::BadRequest().body("Not valid email format")
    }
}

pub async fn login(
    user_auth: web::Json<AuthUser>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    match AuthUser::authenticate(
        &data.get_connection(),
        &user_auth.email,
        &user_auth.password,
    ) {
        Ok((authenticated, token)) => HttpResponse::Ok()
            .append_header(("jwt", token))
            .json(authenticated),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}
#[derive(serde::Deserialize)]
pub struct IdAndValue {
    id: String,
    value: String,
}

pub async fn get_user_by_id(
    id_and_value: web::Json<IdAndValue>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    let id = id_and_value.into_inner().id;

    let user = match User::get_by_id(&data.get_connection(), &id) {
        Ok(result) => result,
        Err(_) => return HttpResponse::BadRequest().body("User ID does not exist."),
    };

    HttpResponse::Ok().json(user)
}

pub async fn update_user_email(
    id_and_value: web::Json<IdAndValue>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    let id_and_email = id_and_value.into_inner();
    let id = id_and_email.id;
    let email = id_and_email.value;
    if verify_email(email.clone()) == true {
        let user = match User::get_by_id(&data.get_connection(), &id) {
            Ok(result) => result,
            Err(_) => return HttpResponse::BadRequest().body("User ID does not exist."),
        };

        match user.update_email(&data.get_connection(), email) {
            Ok(result) => HttpResponse::Ok().json(result),
            Err(_) => HttpResponse::InternalServerError().body("Invalid email address"),
        }
    } else {
        HttpResponse::BadRequest().body("Not valid email format")
    }
}

pub async fn update_user_password(
    id_and_value: web::Json<IdAndValue>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    let id_and_password = id_and_value.into_inner();
    let id = id_and_password.id;
    let password = id_and_password.value;

    if verify_password(password.clone()) == true {
        let user = match User::get_by_id(&data.get_connection(), &id) {
            Ok(result) => result,
            Err(_) => return HttpResponse::BadRequest().body("User ID does not exist."),
        };

        match user.update_password(&data.get_connection(), password) {
            Ok(_) => HttpResponse::Ok().body("Password successfully updated"),
            Err(_) => HttpResponse::InternalServerError().body("Invalid password"),
        }
    } else {
        HttpResponse::BadRequest().body("Password is not strong enough")
    }
}

pub async fn delete_user(
    id: web::Path<String>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    let portfolios = match Portfolio::get_all_from_user(&data.get_connection(), &id) {
        Ok(result) => result,
        Err(err) => return HttpResponse::BadGateway().body(format!("{:?}", err)),
    };

    match User::delete_user(&data.get_connection(), &id) {
        Ok(user) => {
            for portfolio in portfolios {
                match Portfolio::delete_portfolio(&data.get_connection(), &portfolio.id) {
                    Ok(portfolio) => {
                        match Ticker::delete_tickers(&data.get_connection(), &portfolio.id) {
                            Ok(tickers) => (),
                            Err(err) => {
                                return HttpResponse::BadRequest().body(format!("{:?}", err))
                            }
                        }
                    }
                    Err(err) => return HttpResponse::BadRequest().body(format!("{:?}", err)),
                }
            }
            HttpResponse::Ok().body("User successfully deleted")
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}

pub fn verify_email(email: String) -> bool {
    let email_regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
    )
    .unwrap();
    email_regex.is_match(&email) == true
}

pub fn verify_password(passwrod: String) -> bool {
    let password_regex = Regex::new(
        r"^(\P{Ll}*\p{Ll})(\P{Lu}*\p{Lu})(\P{N}*\p{N})([\p{L}\p{N}]*[^\p{L}\p{N}])[\s\S]{8,}$",
    )
    .unwrap();

    //let password_regex = Regex::new(r"^([^\s{2}].)[\s\S]{6,}$").unwrap();
    password_regex.is_match(&passwrod) == true
}

pub async fn create_portfolio(
    data: web::Data<infrastructure::state::AppState>,
    portfolio: web::Json<NewPortfolio>,
) -> impl Responder {
    match NewPortfolio::create(
        portfolio.name.clone(),
        portfolio.user_id.clone(),
        &data.get_connection(),
    ) {
        Ok(created) => HttpResponse::Created().json(created),
        Err(_) => HttpResponse::BadRequest().body("Portfolio with that name already exists"),
    }
}

pub async fn get_portfolios(
    data: web::Data<infrastructure::state::AppState>,
    id_and_value: web::Json<IdAndValue>,
) -> impl Responder {
    match crate::models::portfolio::Portfolio::get_all_from_user(
        &data.get_connection(),
        &id_and_value.into_inner().id,
    ) {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(_) => HttpResponse::BadRequest().body("User ID does not exist."),
    }
}

pub async fn get_portfolio_by_id(
    data: web::Data<infrastructure::state::AppState>,
    id_and_value: web::Json<IdAndValue>,
) -> impl Responder {
    let id = id_and_value.into_inner().id;

    let portfolio = match Portfolio::get_by_id(&data.get_connection(), id) {
        Ok(result) => result,
        Err(_) => return HttpResponse::BadRequest().body("Portfolio with that ID does not exist."),
    };

    HttpResponse::Ok().json(portfolio)
}

pub async fn update_portfolio_name(
    id_and_value: web::Json<IdAndValue>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    let id_and_name = id_and_value.into_inner();
    let id = id_and_name.id;
    let name = id_and_name.value;

    let portfolio = match Portfolio::get_by_id(&data.get_connection(), id) {
        Ok(result) => result,
        Err(_) => return HttpResponse::BadRequest().body("Portfolio with that ID does not exist."),
    };

    match portfolio.update_name(&data.get_connection(), name) {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().body("Invalid email address"),
    }
}

pub async fn delete_portfolio(
    portfolio_id: web::Path<String>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    #[derive(Serialize, Deserialize)]
    struct PortfolioAndTickers {
        portfolio: Portfolio,
        tickers: Vec<Ticker>,
    }

    match Portfolio::delete_portfolio(&data.get_connection(), &portfolio_id) {
        Ok(portfolio) => match Ticker::delete_tickers(&data.get_connection(), &portfolio_id) {
            Ok(tickers) => HttpResponse::Ok().json(PortfolioAndTickers { portfolio, tickers }),
            Err(err) => HttpResponse::BadRequest().body(format!("{:?}", err)),
        },
        Err(err) => HttpResponse::BadRequest().body(format!("{:?}", err)),
    }
}

pub async fn add_ticker(
    data: web::Data<infrastructure::state::AppState>,
    ticker: web::Json<NewTicker>,
) -> impl Responder {
    let provider = yahoo::YahooConnector::new();
    match provider.get_latest_quotes(&ticker.name, "1m").await {
        Ok(_) => (),
        Err(_) => {
            return HttpResponse::BadRequest()
                .body(format!("Ticker '{}' does not exist.", ticker.name))
        }
    };

    match NewTicker::create(
        ticker.name.clone(),
        ticker.portfolio_id.clone(),
        &data.get_connection(),
    ) {
        Ok(created) => HttpResponse::Created().json(created),
        Err(_) => HttpResponse::BadRequest().body("Ticker with that name already exists"),
    }
}

pub async fn delete_ticker(
    data: web::Data<infrastructure::state::AppState>,
    ticker_id: web::Path<String>,
) -> impl Responder {
    match Ticker::delete_ticker(&data.get_connection(), &ticker_id) {
        Ok(ticker) => HttpResponse::Ok().json(ticker),
        Err(err) => HttpResponse::BadRequest().body(format!("Unable to delete ticker: {:?}", err)),
    }
}

fn from_timestamp_to_datetime(timestamp: String) -> DateTime<Utc> {
    let timestamp = timestamp.parse::<i64>().unwrap();
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    return datetime;
}

#[derive(Deserialize, Serialize)]
pub struct SearchedTicker {
    pub symbol: String,
    pub short_name: String,
    pub long_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TickerView {
    id: String,
    name: String,
    symbol: String,
    dividend_value: f64,
    dividend_date: DateTime<Utc>,
    open: f64,
    high: f64,
    low: f64,
    volume: u64,
    close: f64,
    date: DateTime<Utc>,
}
#[derive(Serialize, Deserialize)]
pub struct IdOrSymbol {
    id: String,
    symbol: String,
}

pub async fn get_latest_ticker_info(id_or_symbol: web::Json<IdOrSymbol>) -> impl Responder {
    let mut id = id_or_symbol.id.clone();
    let symbol = id_or_symbol.symbol.clone();
    if id.len() != 36 {
        id = String::from("");
    };

    let provider = yahoo::YahooConnector::new();

    let name = match provider.search_ticker(&symbol).await {
        Ok(items) => items.quotes[0].long_name.clone(),
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Ticker does not exist :{:?}", err))
        }
    };

    let interval = None;
    let range = None;
    let quotes = match provider
        .get_quote_range(&symbol, interval.unwrap_or("1d"), range.unwrap_or("6mo"))
        .await
    {
        Ok(quotes) => quotes,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Response  {}", &err.to_string()))
        }
    };
    let quote = match quotes.last_quote() {
        Ok(result) => result,
        Err(err) => return HttpResponse::BadRequest().body(format!("Quote  {}", &err.to_string())),
    };

    let dividend = match quotes.dividends() {
        Ok(mut dividends) => match dividends.pop() {
            Some(result) => result,
            None => Dividend {
                amount: 0.0,
                date: (0000000000),
            },
        },
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Dividend  {}", &err.to_string()))
        }
    };

    let ticker_view: TickerView = TickerView {
        id: id,
        name: name,
        symbol: symbol.clone(),
        dividend_value: dividend.amount,
        dividend_date: from_timestamp_to_datetime(dividend.date.to_string()),
        open: quote.open,
        high: quote.high,
        low: quote.low,
        volume: quote.volume,
        close: quote.close,
        date: from_timestamp_to_datetime(quote.timestamp.to_string()),
    };

    HttpResponse::Ok().json(ticker_view)
}

pub async fn ticker_search(stock: web::Path<String>) -> impl Responder {
    let provider = yahoo::YahooConnector::new();

    let results = match provider.search_ticker(stock.as_str()).await {
        Ok(result) => result,
        Err(_) => return HttpResponse::BadRequest().body("Ticker does not exist."),
    };

    let item = &results.quotes[0];

    let ticker: SearchedTicker = SearchedTicker {
        symbol: item.symbol.clone(),
        short_name: item.short_name.clone(),
        long_name: item.long_name.clone(),
    };

    HttpResponse::Ok().json(ticker)
}

pub async fn ticker_extensive_search(stock: web::Path<String>) -> impl Responder {
    let provider = yahoo::YahooConnector::new();

    let results = match provider.search_ticker(stock.as_str()).await {
        Ok(result) => result,
        Err(_) => return HttpResponse::BadRequest().body("Ticker does not exist."),
    };

    let items = results.quotes;

    let mut tickers: Vec<SearchedTicker> = Vec::new();

    for item in items {
        let ticker: SearchedTicker = SearchedTicker {
            symbol: item.symbol.clone(),
            short_name: item.short_name.clone(),
            long_name: item.long_name.clone(),
        };

        tickers.push(ticker);
    }

    HttpResponse::Ok().json(tickers)
}

#[derive(Serialize, Deserialize)]
pub struct PortfolioTickerView {
    id: String,
    name: String,
    symbol: String,
    open: f64,
    date: DateTime<Utc>,
}

pub async fn tickers_from_portfolio(
    portfolio_id: web::Path<String>,
    data: web::Data<infrastructure::state::AppState>,
) -> impl Responder {
    let mut tickers_info: Vec<PortfolioTickerView> = Vec::new();

    let tickers = match crate::models::ticker::Ticker::get_all_from_portfolio(
        &data.get_connection(),
        portfolio_id.into_inner(),
    ) {
        Ok(results) => results,
        Err(_) => {
            return HttpResponse::BadRequest()
                .body("Portfolio ID does not exist or there is not any ticker.")
        }
    };

    let names = match get_stocks_name(&tickers).await {
        Ok(result) => result,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Ticker names: {}", &err.to_string()))
        }
    };

    let quotes = match get_stocks_values(&tickers, None, None).await {
        Ok(result) => result,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Ticker values: {}", &err.to_string()))
        }
    };
    /*
    let dividends = match get_stocks_dividends(&tickers, None, None).await {
        Ok(result) => result,
        Err(err) => return HttpResponse::BadRequest().body(format!("Ticker dividend: {}", &err.to_string())),
    };
    */

    for i in 0..tickers.len() {
        let info = PortfolioTickerView {
            id: tickers[i].id.clone(),
            name: names[i].clone(),
            symbol: tickers[i].name.clone(),
            open: quotes[i].open,
            date: from_timestamp_to_datetime(quotes[i].timestamp.to_string()),
        };

        tickers_info.push(info);
    }

    HttpResponse::Ok().json(tickers_info)
}

pub async fn get_stocks_name(tickers: &Vec<Ticker>) -> Result<Vec<String>, YahooError> {
    let mut tickers_name: Vec<String> = Vec::new();

    let provider = yahoo::YahooConnector::new();

    for ticker in tickers {
        let name = match provider.search_ticker(&ticker.name).await {
            Ok(result) => result.quotes[0].long_name.clone(),
            Err(err) => return Err(err),
        };

        tickers_name.push(name);
    }

    return Ok(tickers_name);
}

pub async fn get_stocks_values(
    tickers: &Vec<Ticker>,
    interval: Option<&str>,
    range: Option<&str>,
) -> Result<Vec<Quote>, YahooError> {
    let mut tickers_values: Vec<Quote> = Vec::new();

    let provider = yahoo::YahooConnector::new();

    for ticker in tickers {
        let quote = match provider
            .get_quote_range(
                &ticker.name,
                interval.unwrap_or("1d"),
                range.unwrap_or("6mo"),
            )
            .await
        {
            Ok(quotes) => match quotes.last_quote() {
                Ok(result) => result,
                Err(err) => return Err(err),
            },
            Err(err) => return Err(err),
        };

        tickers_values.push(quote);
    }

    return Ok(tickers_values);
}

pub async fn get_stocks_dividends(
    tickers: &Vec<Ticker>,
    interval: Option<&str>,
    range: Option<&str>,
) -> Result<Vec<Dividend>, YahooError> {
    let mut tickers_dividend: Vec<Dividend> = Vec::new();

    let provider = yahoo::YahooConnector::new();

    for ticker in tickers {
        let quotes = match provider
            .get_quote_range(
                &ticker.name,
                interval.unwrap_or("1d"),
                range.unwrap_or("6mo"),
            )
            .await
        {
            Ok(result) => result,
            Err(err) => return Err(err),
        };

        let dividend = match quotes.dividends() {
            Ok(mut dividends) => match dividends.pop() {
                Some(result) => result,
                None => Dividend {
                    amount: 0.0,
                    date: (0000000000),
                },
            },
            Err(err) => return Err(err),
        };

        tickers_dividend.push(dividend);
    }

    return Ok(tickers_dividend);
}

#[cfg(test)]
mod tests {

    use super::{
        from_timestamp_to_datetime, get_latest_ticker_info, verify_email, verify_password,
        IdOrSymbol, TickerView,
    };
    use actix_web::{web, HttpResponse, Responder};
    use chrono::{DateTime, NaiveDate, Utc};

    #[test]
    fn test_verify_email_valid() {
        let email: String = String::from("test@mail.com");
        assert_eq!(verify_email(email), true);
    }

    #[test]
    fn test_verify_email_invalid1() {
        let email: String = String::from("@mail.com");
        assert_eq!(verify_email(email), false);
    }

    #[test]
    fn test_verify_email_invalid2() {
        let email: String = String::from("testmail.com");
        assert_eq!(verify_email(email), false);
    }

    #[test]
    fn test_verify_email_invalid3() {
        let email: String = String::from("test@.com");
        assert_eq!(verify_email(email), false);
    }

    #[test]
    fn test_verify_email_invalid4() {
        let email: String = String::from("test@mail.");
        assert_eq!(verify_email(email), false);
    }

    #[test]
    fn test_verify_email_invalid5() {
        let email: String = String::from("testmailcom");
        assert_eq!(verify_email(email), false);
    }
    #[test]
    fn test_verify_email_invalid6() {
        let email: String = String::from("");
        assert_eq!(verify_email(email), false);
    }
    #[test]
    fn test_verify_email_invalid7() {
        let email: String = String::from("       ");
        assert_eq!(verify_email(email), false);
    }

    #[test]
    fn test_verify_password_valid() {
        let password: String = String::from("pA5!ssword12");
        assert_eq!(verify_password(password), true);
    }

    #[test]
    fn test_verify_password_invalid() {
        let password: String = String::from("         ");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid2() {
        let password: String = String::from("12345678");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid3() {
        let password: String = String::from("password");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid4() {
        let password: String = String::from("password1");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid5() {
        let password: String = String::from("password1!");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid6() {
        let password: String = String::from("PASSWORD");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid7() {
        let password: String = String::from("PASSWORD1");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid8() {
        let password: String = String::from("PASSWORD1!");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_verify_password_invalid9() {
        let password: String = String::from("Test1?");
        assert_eq!(verify_password(password), false);
    }

    #[test]
    fn test_from_timestamp_to_datetime() {
        //01.01.2022. 00:00:00
        let timestamp = 1640995200;
        let timestamp_string = timestamp.to_string();

        let datetime: DateTime<Utc> =
            DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2022, 1, 1).and_hms(0, 0, 0), Utc);

        assert_eq!(from_timestamp_to_datetime(timestamp_string), datetime);
    }
}
