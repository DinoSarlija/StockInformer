use crate::infrastructure::setup;
use actix_web::web::{self};

pub fn setup_routes(cfg: &mut web::ServiceConfig) {
    // Register
    cfg.service(web::resource("/register").route(web::post().to(setup::register)));
    // Login
    cfg.service(web::resource("/login").route(web::get().to(setup::login)));

    // User
    //GET
    cfg.service(web::resource("/users").route(web::get().to(setup::get_all_users)));
    cfg.service(web::resource("/user").route(web::get().to(setup::get_user_by_id)));
    // PUT
    cfg.service(web::resource("/user/email").route(web::put().to(setup::update_user_email)));
    cfg.service(web::resource("/user/password").route(web::put().to(setup::update_user_password)));
    // DELETE
    cfg.service(web::resource("/user/{id}").route(web::put().to(setup::delete_user)));

    //Portfolio
    //GET
    cfg.service(
        web::resource("/portfolios")
            .route(web::get().to(setup::get_portfolios))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );

    //ovo nije potrebno
    cfg.service(
        web::resource("/portfolio")
            .route(web::get().to(setup::get_portfolio_by_id))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
    //POST
    cfg.service(
        web::resource("portfolio/new")
            .route(web::post().to(setup::create_portfolio))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
    //PUT
    cfg.service(
        web::resource("portfolio/name")
            .route(web::put().to(setup::update_portfolio_name))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
    //DELETE
    cfg.service(
        web::resource("portfolio/{id}")
            .route(web::put().to(setup::delete_portfolio))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );

    //Ticker
    //GET
    cfg.service(
        web::resource("/ticker/info")
            .route(web::get().to(setup::get_latest_ticker_info))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
    //POST
    cfg.service(
        web::resource("/ticker/new")
            .route(web::post().to(setup::add_ticker))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
    //PUT
    //DELETE
    cfg.service(
        web::resource("/ticker/{ticker}")
            .route(web::put().to(setup::delete_ticker))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );

    //cfg.service(web::resource("/tickers/{portfolio_id}").route(web::get().to(setup::tickers_from_portfolio)).wrap(crate::infrastructure::middleware::LoggedGuard));

    cfg.service(
        web::resource("/tickers/{portfolio_id}")
            .route(web::get().to(setup::tickers_from_portfolio))
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
    cfg.service(
        web::resource("/ticker/search/{name}")
            .route(web::get().to(setup::ticker_search))
            .wrap(crate::infrastructure::middleware::LoggedGuard)
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
    cfg.service(
        web::resource("/ticker/search/{name}/extended")
            .route(web::get().to(setup::ticker_extensive_search))
            .wrap(crate::infrastructure::middleware::LoggedGuard)
            .wrap(crate::infrastructure::middleware::LoggedGuard),
    );
}
