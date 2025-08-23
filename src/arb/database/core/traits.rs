use sea_orm::*;

pub trait WithConnection {
    fn connection(&self) -> &DatabaseConnection;
}
