use super::{
    http,
    model::{InstallRequest, LifecycleEvent, MarketplaceEntry},
};
use az_ui_components::{
    admin::{AsyncResult, EditorDialog, RequestState},
    button::{Button, ButtonSize, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogTitle},
    input::Input,
};
use dioxus::prelude::*;
use dioxus_icons::lucide::RefreshCw;

#[component]
pub(super) fn InstallDialog(
    value: Option<MarketplaceEntry>,
    on_close: Callback<()>,
    on_saved: Callback<()>,
) -> Element {
    let mut git = use_signal(|| value.as_ref().map(|e| e.git.clone()).unwrap_or_default());
    let mut revision = use_signal(|| value.as_ref().map(|e| e.rev.clone()).unwrap_or_default());
    rsx! { EditorDialog { title: "安装插件", description: "安装后按实际提交锁定版本。", submit_label: "安装", pending_label: "正在安装", on_close, on_saved,
        save: move |_| -> AsyncResult<()> {
            let git = git().trim().to_owned(); let rev = revision().trim().to_owned();
            Box::pin(async move { http::install(InstallRequest { git, rev: if rev.is_empty() { None } else { Some(rev) } }).await })
        },
        label { class: "admin-field", span { "Git 仓库" } Input { aria_label: "Git 仓库", r#type: "url", required: true, value: git(), oninput: move |event: FormEvent| git.set(event.value()) } }
        label { class: "admin-field", span { "提交、标签或分支" } Input { aria_label: "插件版本", value: revision(), oninput: move |event: FormEvent| revision.set(event.value()) } }
    } }
}

#[component]
pub(super) fn DetailsDialog(entry: MarketplaceEntry, on_close: Callback<()>) -> Element {
    rsx! {
        Dialog { open: true, on_open_change: move |open: bool| if !open { on_close.call(()) },
            DialogTitle { "{entry.title}" }
            DialogDescription { "{entry.summary}" }
            dl { class: "admin-details",
                dt { "Git 仓库" } dd { code { class: "admin-code", "{entry.git}" } }
                dt { "状态" } dd { "{entry.state_label()}" }
                dt { "活动版本" } dd { code { class: "admin-code", "{entry.active_revision.as_deref().unwrap_or(\"未安装\")}" } }
                dt { "市场版本" } dd { code { class: "admin-code", "{entry.rev}" } }
                dt { "运行目标" } dd { "{entry.runtime.as_deref().unwrap_or(\"未声明\")}" }
                dt { "许可证" } dd { "{entry.license}" }
                dt { "网络" } dd { if entry.capabilities.network.is_empty() { "未申请" } else { "{entry.capabilities.network.join(\"、\")}" } }
                dt { "文件系统" } dd { if entry.capabilities.filesystem.is_empty() { "未申请" } else { "{entry.capabilities.filesystem.join(\"、\")}" } }
                dt { "数据库" } dd { if entry.capabilities.database { "已申请" } else { "未申请" } }
            }
            if let Some(source) = entry.source_id { LifecycleEvents { source } }
            footer { class: "admin-form-footer", Button { variant: ButtonVariant::Outline, onclick: move |_| on_close.call(()), "关闭" } }
        }
    }
}

#[component]
fn LifecycleEvents(source: String) -> Element {
    let mut revision = use_signal(|| 0_u64);
    let events = use_resource(move || {
        let _ = revision();
        let source = source.clone();
        async move {
            http::get::<Vec<LifecycleEvent>>(&format!("/api/runtime/plugins/{source}/events")).await
        }
    });
    rsx! { section { class: "admin-section",
        header { class: "admin-toolbar", h2 { "生命周期记录" } Button { variant: ButtonVariant::Ghost, size: ButtonSize::IconSm, title: "刷新生命周期", aria_label: "刷新生命周期", onclick: move |_| revision += 1, RefreshCw {} } }
        match events.read().as_ref() {
            Some(Ok(events)) => rsx! { ol { class: "admin-delete-targets", for event in events { li { p { "{event.lifecycle} · {event.detail}" } time { class: "admin-meta", "{event.created_at}" } } } } },
            Some(Err(error)) => rsx! { RequestState { error: error.clone(), on_retry: move |_| revision += 1 } },
            None => rsx! { p { role: "status", "正在读取记录" } },
        }
    } }
}
