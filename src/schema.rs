table! {
    portfolios (id) {
        id -> Varchar,
        name -> Varchar,
        created_at -> Date,
        is_deleted -> Bool,
        user_id -> Varchar,
    }
}

table! {
    tickers (id) {
        id -> Varchar,
        name -> Varchar,
        portfolio_id -> Varchar,
        is_deleted -> Bool,
    }
}

table! {
    users (id) {
        id -> Varchar,
        email -> Varchar,
        password -> Varchar,
        is_deleted -> Bool,
    }
}

joinable!(portfolios -> users (user_id));
joinable!(tickers -> portfolios (portfolio_id));

allow_tables_to_appear_in_same_query!(portfolios, tickers, users,);
