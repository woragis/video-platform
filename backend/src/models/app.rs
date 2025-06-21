use redis::Client as RedisClient;
use tokio_postgres::Client as PostgresClient;


#[derive(Debug)]
pub struct AppState {
    pub db: PostgresClient,
    pub redis: RedisClient
}