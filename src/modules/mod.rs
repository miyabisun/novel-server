pub mod kakuyomu;
pub mod narou;
pub mod nocturne;
pub mod syosetu;

use crate::error::AppError;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    Narou,
    Nocturne,
    Kakuyomu,
}

impl ModuleType {
    pub fn resolve(s: &str) -> Result<Self, AppError> {
        match s {
            "narou" => Ok(Self::Narou),
            "nocturne" => Ok(Self::Nocturne),
            "kakuyomu" => Ok(Self::Kakuyomu),
            _ => Err(AppError::BadRequest("Invalid type".to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Narou => "narou",
            Self::Nocturne => "nocturne",
            Self::Kakuyomu => "kakuyomu",
        }
    }

    pub async fn fetch_ranking_list(
        &self,
        client: &reqwest::Client,
        limit: usize,
        period: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => narou::fetch_ranking_list(client, limit, period).await,
            Self::Nocturne => nocturne::fetch_ranking_list(client, limit, period).await,
            Self::Kakuyomu => kakuyomu::fetch_ranking_list(client, period).await,
        }
    }

    pub async fn fetch_page(
        &self,
        client: &reqwest::Client,
        id: &str,
        page_id: &str,
    ) -> Result<Option<String>, AppError> {
        match self {
            Self::Narou => narou::fetch_page(client, id, page_id).await,
            Self::Nocturne => nocturne::fetch_page(client, id, page_id).await,
            Self::Kakuyomu => kakuyomu::fetch_page(client, id, page_id).await,
        }
    }

    pub async fn fetch_detail(
        &self,
        client: &reqwest::Client,
        id: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => narou::fetch_detail(client, id).await,
            Self::Nocturne => nocturne::fetch_detail(client, id).await,
            Self::Kakuyomu => kakuyomu::fetch_detail(client, id).await,
        }
    }

    pub async fn fetch_search(
        &self,
        client: &reqwest::Client,
        word: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => narou::fetch_search(client, word).await,
            Self::Nocturne => nocturne::fetch_search(client, word).await,
            Self::Kakuyomu => kakuyomu::fetch_search(client, word).await,
        }
    }

    pub async fn fetch_toc(
        &self,
        client: &reqwest::Client,
        id: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => narou::fetch_toc(client, id).await,
            Self::Nocturne => nocturne::fetch_toc(client, id).await,
            Self::Kakuyomu => kakuyomu::fetch_toc(client, id).await,
        }
    }

    pub async fn fetch_data(
        &self,
        client: &reqwest::Client,
        ids: &[String],
    ) -> Result<Vec<Value>, AppError> {
        match self {
            Self::Narou => narou::fetch_data(client, ids).await,
            Self::Nocturne => nocturne::fetch_data(client, ids).await,
            Self::Kakuyomu => kakuyomu::fetch_data(client, ids).await,
        }
    }

    pub async fn fetch_datum(
        &self,
        client: &reqwest::Client,
        id: &str,
    ) -> Result<Value, AppError> {
        match self {
            Self::Narou => narou::fetch_datum(client, id).await,
            Self::Nocturne => nocturne::fetch_datum(client, id).await,
            Self::Kakuyomu => kakuyomu::fetch_datum(client, id).await,
        }
    }
}
