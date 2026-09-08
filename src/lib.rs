use az_dioxus_admin_shell::{ApplicationPage, ApplicationPlugin, ApplicationScene};
use az_ui_components::button::{Button, ButtonVariant};
use dill::CatalogBuilder;
use dioxus::prelude::*;

#[derive(Debug)]
pub struct MarketplacePlugin;

impl ApplicationPlugin for MarketplacePlugin {
    fn pages(&self) -> Vec<ApplicationPage> {
        vec![ApplicationPage {
            id: "marketplace",
            label: "插件市场",
            icon: Some("store"),
            scene: ApplicationScene { id: "system", label: "系统" },
            render: MarketplacePage,
        }]
    }
}

pub fn register(builder: &mut CatalogBuilder) {
    builder.add_value(MarketplacePlugin).bind::<dyn ApplicationPlugin, MarketplacePlugin>();
}

#[allow(non_snake_case)]
fn MarketplacePage() -> Element {
    let mut status = use_signal(|| "市场索引已连接".to_owned());
    rsx! {
        section {
            h2 { "插件市场" }
            p { "从 Git 仓库发现、安装和回滚租户插件。" }
            div { class: "grid gap-3 md:grid-cols-2",
                article { class: "border p-4",
                    h3 { "Hello Counter" }
                    p { "最小全栈示例，提供页面和健康接口。" }
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Outline,
                        onclick: move |_| status.set("安装请求已提交，正在等待运行时激活".to_owned()),
                        "安装"
                    }
                }
            }
            p { role: "status", "{status}" }
        }
    }
}

#[cfg(feature = "server")]
pub fn router(_catalog: &dill::Catalog) -> anyhow::Result<axum::Router> {
    use axum::{Router, routing::get};
    Ok(Router::new().route("/api/plugins/marketplace", get(|| async { "ok" })))
}
