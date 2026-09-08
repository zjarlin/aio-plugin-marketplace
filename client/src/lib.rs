use az_dioxus_admin_shell::{ApplicationPage, ApplicationPlugin, ApplicationScene};
use az_ui_components::{
    badge::{Badge, BadgeVariant},
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
};
use dill::CatalogBuilder;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MarketplacePlugin;

impl ApplicationPlugin for MarketplacePlugin {
    fn pages(&self) -> Vec<ApplicationPage> {
        vec![ApplicationPage {
            id: "marketplace",
            label: "插件市场",
            icon: Some("store"),
            scene: ApplicationScene {
                id: "system",
                label: "系统",
            },
            required_permission: Some("plugin:manage"),
            render: MarketplacePage,
        }]
    }
}

pub fn register(builder: &mut CatalogBuilder) {
    builder
        .add_value(MarketplacePlugin)
        .bind::<dyn ApplicationPlugin, MarketplacePlugin>();
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct RuntimeResponse<T> {
    data: T,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct MarketplaceEntry {
    git: String,
    rev: String,
    title: String,
    summary: String,
    license: String,
    tags: Vec<String>,
    installed: bool,
    source_id: Option<String>,
    state: Option<PluginState>,
    active_revision: Option<String>,
    runtime: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PluginState {
    Active,
    Disabled,
    Failed,
}

#[derive(Serialize)]
struct InstallRequest<'a> {
    git: &'a str,
    rev: Option<&'a str>,
}

#[allow(non_snake_case)]
fn MarketplacePage() -> Element {
    let entries = use_resource(load_marketplace);
    let status = use_signal(|| None::<String>);
    let mut uninstalling = use_signal(|| None::<MarketplaceEntry>);
    let Some(result) = entries.read().as_ref().cloned() else {
        return rsx! { p { "正在读取插件市场" } };
    };
    let entries = match result {
        Ok(entries) => entries,
        Err(error) => return rsx! { p { role: "alert", "加载插件市场失败: {error}" } },
    };
    rsx! {
        section {
            h2 { "插件市场" }
            p { "从 Git 仓库发现、安装和回滚当前租户的插件。" }
            if let Some(message) = status() {
                p { role: "status", "{message}" }
            }
            div { class: "grid gap-3 md:grid-cols-2",
                for entry in entries {
                    MarketplaceCard { entry, status, uninstalling }
                }
            }
        }
        if let Some(entry) = uninstalling() {
            Dialog {
                open: true,
                on_open_change: move |open: bool| if !open { uninstalling.set(None) },
                DialogTitle { "卸载 {entry.title}" }
                DialogDescription { "页面和路由将立即从当前租户移除，历史版本仍可审计。" }
                div { class: "flex justify-end gap-2",
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| uninstalling.set(None),
                        "取消"
                    }
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| {
                            let source_id = entry.source_id.clone().unwrap_or_default();
                            spawn(async move {
                                perform(&format!("/api/runtime/plugins/{source_id}/uninstall"), status).await;
                            });
                        },
                        "确认卸载"
                    }
                }
            }
        }
    }
}

#[component]
fn MarketplaceCard(
    entry: MarketplaceEntry,
    status: Signal<Option<String>>,
    uninstalling: Signal<Option<MarketplaceEntry>>,
) -> Element {
    let install_entry = entry.clone();
    let uninstall_entry = entry.clone();
    let source_id = entry.source_id.clone().unwrap_or_default();
    let disable_source = source_id.clone();
    let enable_source = source_id.clone();
    let rollback_source = source_id;
    rsx! {
        article { class: "border p-4",
            div { class: "flex items-center justify-between gap-2",
                h3 { "{entry.title}" }
                Badge { variant: BadgeVariant::Outline, "{entry.license}" }
            }
            p { "{entry.summary}" }
            p { class: "text-sm text-muted-foreground", "{entry.git}" }
            if let Some(revision) = entry.active_revision.as_ref() {
                p { class: "text-sm text-muted-foreground", "活动版本：{short_revision(revision)}" }
            }
            if let Some(runtime) = entry.runtime.as_ref() {
                p { class: "text-sm text-muted-foreground", "运行目标：{runtime}" }
            }
            div { class: "flex flex-wrap gap-2",
                if !entry.installed {
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let request = install_entry.clone();
                            spawn(async move { install(&request, status).await });
                        },
                        "安装"
                    }
                } else {
                    if entry.state == Some(PluginState::Active) {
                        Button {
                            r#type: "button",
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                let source = disable_source.clone();
                                spawn(async move { perform(&format!("/api/runtime/plugins/{source}/disable"), status).await });
                            },
                            "停用"
                        }
                    } else {
                        Button {
                            r#type: "button",
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                let source = enable_source.clone();
                                spawn(async move { perform(&format!("/api/runtime/plugins/{source}/enable"), status).await });
                            },
                            "启用"
                        }
                    }
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| {
                            let source = rollback_source.clone();
                            spawn(async move { perform(&format!("/api/runtime/plugins/{source}/rollback"), status).await });
                        },
                        "回滚"
                    }
                    Button {
                        r#type: "button",
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| uninstalling.set(Some(uninstall_entry.clone())),
                        "卸载"
                    }
                }
            }
        }
    }
}

fn short_revision(revision: &str) -> &str {
    revision.get(..12).unwrap_or(revision)
}

async fn load_marketplace() -> Result<Vec<MarketplaceEntry>, String> {
    get::<Vec<MarketplaceEntry>>("/api/runtime/marketplace").await
}

async fn install(entry: &MarketplaceEntry, mut status: Signal<Option<String>>) {
    status.set(Some(format!("正在安装 {}", entry.title)));
    let request = InstallRequest {
        git: &entry.git,
        rev: Some(&entry.rev),
    };
    let result = async {
        let builder = gloo_net::http::Request::post("/api/runtime/plugins/install")
            .json(&request)
            .map_err(|error| error.to_string())?;
        let response = builder.send().await.map_err(|error| error.to_string())?;
        if response.ok() {
            Ok(())
        } else {
            Err(response.text().await.unwrap_or_default())
        }
    }
    .await;
    finish(result, status);
}

async fn perform(path: &str, mut status: Signal<Option<String>>) {
    status.set(Some("正在执行插件生命周期操作".to_owned()));
    let result = match gloo_net::http::Request::post(path).send().await {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => Err(response.text().await.unwrap_or_default()),
        Err(error) => Err(error.to_string()),
    };
    finish(result, status);
}

fn finish(result: Result<(), String>, mut status: Signal<Option<String>>) {
    match result {
        Ok(()) => {
            status.set(Some("操作成功，正在刷新插件目录".to_owned()));
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        }
        Err(error) => status.set(Some(format!("操作失败: {error}"))),
    }
}

async fn get<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let response = gloo_net::http::Request::get(path)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.ok() {
        return Err(response.text().await.unwrap_or_default());
    }
    response
        .json::<RuntimeResponse<T>>()
        .await
        .map(|response| response.data)
        .map_err(|error| error.to_string())
}
