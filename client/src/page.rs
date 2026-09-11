use super::{
    dialogs::{DetailsDialog, InstallDialog},
    http,
    model::{MarketplaceEntry, PluginState, short_revision},
};
use az_ui_components::{
    admin::{
        AsyncResult, CollectionTable, DeleteRecordsDialog, PageHeader, PageSurface, RequestState,
        SortValue, StatusMessage,
    },
    button::{Button, ButtonSize, ButtonVariant},
    data_table::{DataTableCellContext, DataTableColumn},
    select::{Select, SelectItem},
};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Download, Info, Pause, Play, RefreshCw, RotateCcw, Trash2};

#[allow(non_snake_case)]
pub(super) fn MarketplacePage() -> Element {
    let mut revision = use_signal(|| 0_u64);
    let resource = use_resource(move || {
        let _ = revision();
        http::get::<Vec<MarketplaceEntry>>("/api/runtime/marketplace")
    });
    let mut installing = use_signal(|| None::<Option<MarketplaceEntry>>);
    let mut removing = use_signal(|| None::<MarketplaceEntry>);
    let mut details = use_signal(|| None::<MarketplaceEntry>);
    let mut filter = use_signal(|| "all".to_owned());
    let mut busy = use_signal(|| false);
    let mut status = use_signal(|| None::<(bool, String)>);
    let action = use_callback(move |(entry, action): (MarketplaceEntry, String)| {
        if busy() {
            return;
        }
        busy.set(true);
        status.set(None);
        spawn(async move {
            let result =
                http::action(entry.source_id.as_deref().unwrap_or_default(), &action).await;
            busy.set(false);
            match result {
                Ok(()) => {
                    status.set(Some((false, format!("{}：操作已完成", entry.title))));
                    revision += 1;
                }
                Err(error) => status.set(Some((true, error))),
            }
        });
    });
    let entries = match resource.read().as_ref().cloned() {
        Some(Ok(value)) => value,
        Some(Err(error)) => {
            return rsx! { PageSurface { RequestState { error, on_retry: move |_| revision += 1 } } };
        }
        None => return rsx! { PageSurface { RequestState {} } },
    };
    rsx! {
        PageSurface {
            PageHeader { title: "插件市场", detail: format!("{} 个插件 · {} 个已安装", entries.len(), entries.iter().filter(|e| e.installed).count()),
                Button { size: ButtonSize::Icon, variant: ButtonVariant::Outline, title: "刷新市场", aria_label: "刷新市场", onclick: move |_| revision += 1, RefreshCw {} }
                Button { disabled: busy(), onclick: move |_| installing.set(Some(None)), Download {} "从 Git 安装" }
            }
            if busy() { p { role: "status", "正在执行插件操作" } }
            if let Some((error, message)) = status() { StatusMessage { error, message } }
            CollectionTable { label: "插件", rows: entries.into_iter().filter(|e| filter() == "all" || e.installed == (filter() == "installed")).collect::<Vec<_>>(),
                columns: vec![DataTableColumn::leaf("title", "插件").width(280), DataTableColumn::leaf("state", "状态").width(120), DataTableColumn::leaf("revision", "活动版本").width(180), DataTableColumn::leaf("actions", "操作").width(240)],
                row_key: |e: MarketplaceEntry| e.git, search_text: |e: MarketplaceEntry| format!("{} {} {} {}", e.title, e.git, e.summary, e.tags.join(" ")), sort_value: |(e, _): (MarketplaceEntry, String)| SortValue::Text(e.title), sortable: vec!["title".into()],
                tools: rsx! { label { class: "admin-filter", "安装状态" Select { aria_label: "安装状态", value: filter(), options: [("all", "全部插件"), ("installed", "已安装"), ("available", "未安装")].into_iter().map(|(id,label)| SelectItem::new(id,label)).collect(), on_value_change: move |value| filter.set(value) } } },
                render_cell: move |context: DataTableCellContext<MarketplaceEntry>| { let entry = context.row; match context.column.key.as_str() {
                    "title" => rsx! { div { class: "admin-section", button { r#type: "button", class: "admin-row-title", onclick: { let e = entry.clone(); move |_| details.set(Some(e.clone())) }, "{entry.title}" } p { class: "admin-meta", "{entry.summary}" } } },
                    "state" => rsx! { span { class: "admin-status", "data-enabled": (entry.state == Some(PluginState::Active)).to_string(), if entry.state == Some(PluginState::Active) { Check {} } "{entry.state_label()}" } },
                    "revision" => rsx! { code { class: "admin-code", "{entry.active_revision.as_deref().map(short_revision).unwrap_or(\"未安装\")}" } },
                    "actions" => rsx! { div { class: "admin-actions",
                        Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, title: "详情 {entry.title}", aria_label: "详情 {entry.title}", onclick: { let e = entry.clone(); move |_| details.set(Some(e.clone())) }, Info {} }
                        if !entry.installed { Button { variant: ButtonVariant::Outline, disabled: busy(), aria_label: "安装 {entry.title}", onclick: { let e = entry.clone(); move |_| installing.set(Some(Some(e.clone()))) }, Download {} "安装" } }
                        else {
                            Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, disabled: busy(), title: "安装更新 {entry.title}", aria_label: "安装更新 {entry.title}", onclick: { let e = entry.clone(); move |_| installing.set(Some(Some(e.clone()))) }, Download {} }
                            if entry.state == Some(PluginState::Active) { Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, disabled: busy(), title: "停用 {entry.title}", aria_label: "停用 {entry.title}", onclick: { let e = entry.clone(); move |_| action.call((e.clone(), "disable".into())) }, Pause {} } }
                            else { Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, disabled: busy(), title: "启用 {entry.title}", aria_label: "启用 {entry.title}", onclick: { let e = entry.clone(); move |_| action.call((e.clone(), "enable".into())) }, Play {} } }
                            Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, disabled: busy(), title: "回滚 {entry.title}", aria_label: "回滚 {entry.title}", onclick: { let e = entry.clone(); move |_| action.call((e.clone(), "rollback".into())) }, RotateCcw {} }
                            Button { size: ButtonSize::IconSm, variant: ButtonVariant::Ghost, disabled: busy(), title: "卸载 {entry.title}", aria_label: "卸载 {entry.title}", onclick: move |_| removing.set(Some(entry.clone())), Trash2 {} }
                        }
                    } }, _ => rsx! {},
                } },
            }
        }
        if let Some(value) = installing() { InstallDialog { value, on_close: move |_| installing.set(None), on_saved: move |_| { installing.set(None); status.set(Some((false, "插件已安装并更新当前租户目录".into()))); revision += 1; } } }
        if let Some(entry) = details() { DetailsDialog { entry, on_close: move |_| details.set(None) } }
        if let Some(entry) = removing() { DeleteRecordsDialog { title: "卸载插件", confirm_label: "确认卸载", warning: "从当前租户移除页面、路由和运行实例，保留业务数据与版本历史。", items: vec![entry], item_label: |e: MarketplaceEntry| e.title,
            delete: |e: MarketplaceEntry| -> AsyncResult<()> { Box::pin(async move { http::action(e.source_id.as_deref().unwrap_or_default(), "uninstall").await }) },
            on_close: move |_| removing.set(None), on_deleted: move |_| { status.set(Some((false, "插件已卸载".into()))); revision += 1; },
        } }
    }
}
