use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use r2d2::Pool;
use std::env;
use std::sync::Arc;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub struct StaticData {
    pub db: DbPool,
}

#[derive(Clone)]
pub struct AppState {
    pub static_data: Arc<StaticData>,
}

impl AppState {
    pub fn get_connection(&self) -> DbConnection {
        self.static_data
            .db
            .get()
            .expect("Failed to retrieve DB connection from pool")
    }
}

pub fn initialize() -> AppState {
    let db_pool = get_connection_pool();

    AppState {
        static_data: Arc::new(StaticData { db: db_pool }),
    }
}

pub fn get_connection_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool.")
}
