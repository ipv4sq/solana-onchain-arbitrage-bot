use sea_orm::*;

use crate::arb::repository::RepositoryResult;

pub trait WithConnection {
    fn connection(&self) -> &DatabaseConnection;
}