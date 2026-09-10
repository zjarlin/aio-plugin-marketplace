use axum::{Router, routing::get};
use dill::CatalogBuilder;

#[derive(Debug)]
pub struct MarketplaceService;

pub fn register(builder: &mut CatalogBuilder) -> anyhow::Result<()> {
    builder.add_value(MarketplaceService);
    Ok(())
}

pub fn router(_catalog: &dill::Catalog) -> anyhow::Result<Router> {
    Ok(Router::new().route("/api/plugins/marketplace", get(|| async { "ok" })))
}
