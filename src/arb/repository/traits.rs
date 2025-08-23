use async_trait::async_trait;
use sea_orm::*;
use crate::arb::repository::error::RepositoryResult;

#[async_trait]
pub trait Repository<T: EntityTrait> {
    async fn find_by_id(&self, id: i32) -> RepositoryResult<Option<T::Model>>;
    async fn find_all(&self) -> RepositoryResult<Vec<T::Model>>;
    async fn create(&self, model: T::ActiveModel) -> RepositoryResult<T::Model>;
    async fn update(&self, model: T::ActiveModel) -> RepositoryResult<T::Model>;
    async fn delete(&self, id: i32) -> RepositoryResult<bool>;
    async fn count(&self) -> RepositoryResult<u64>;
}

#[async_trait]
pub trait Paginate<T: EntityTrait> {
    async fn paginate(
        &self,
        page: u64,
        per_page: u64,
    ) -> RepositoryResult<(Vec<T::Model>, u64)>;
}

#[async_trait]
pub trait Search<T: EntityTrait> {
    async fn search(&self, query: &str) -> RepositoryResult<Vec<T::Model>>;
}

#[async_trait]
pub trait BatchOperations<T: EntityTrait> {
    async fn batch_create(&self, models: Vec<T::ActiveModel>) -> RepositoryResult<()>;
    async fn batch_update(&self, models: Vec<T::ActiveModel>) -> RepositoryResult<()>;
    async fn batch_delete(&self, ids: Vec<i32>) -> RepositoryResult<u64>;
}

pub trait WithConnection {
    fn connection(&self) -> &DatabaseConnection;
}