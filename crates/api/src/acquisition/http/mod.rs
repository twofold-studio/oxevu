use super::*;
use crate::contract::{ErrorBody, acquisition_catalog_model::*, acquisition_transfer_model::*};
use axum::extract::Query;
use utoipa_axum::{router::OpenApiRouter, routes};
mod catalog;
mod configuration;
mod migration;
mod model;
mod transfers;
use model::*;

pub(crate) fn router() -> OpenApiRouter<App> {
    catalog::router()
        .merge(transfers::router())
        .merge(configuration::router())
        .merge(migration::router())
}
