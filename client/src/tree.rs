use super::model::MarketplaceEntry;
use az_ui_components::input::Input;
use dioxus::prelude::*;
use dioxus_icons::lucide::Package;

#[component]
pub(super) fn PluginTree(
    entries: Vec<MarketplaceEntry>,
    selected: Option<String>,
    search: String,
    on_search: Callback<String>,
    on_select: Callback<String>,
) -> Element {
    let mut installed_open = use_signal(|| true);
    let mut available_open = use_signal(|| true);
    let query = search.to_lowercase();
    let entries: Vec<_> = entries
        .into_iter()
        .filter(|entry| {
            format!("{} {} {}", entry.title, entry.summary, entry.tags.join(" "))
                .to_lowercase()
                .contains(&query)
        })
        .collect();
    let keys: Vec<_> = entries
        .iter()
        .filter(|entry| {
            if entry.installed {
                installed_open()
            } else {
                available_open()
            }
        })
        .map(|entry| entry.git.clone())
        .collect();
    let selection = selected.clone();
    rsx! {
        div { class: "extension-browser__search", Input { aria_label: "搜索插件", placeholder: "搜索插件", value: search, oninput: move |event: FormEvent| on_search.call(event.value()) } }
        div { role: "tree", aria_label: "插件", onkeydown: move |event: KeyboardEvent| {
            let current = keys.iter().position(|key| Some(key)==selection.as_ref());
            let next = match event.key() { Key::ArrowDown => Some(current.map_or(0, |current| (current+1).min(keys.len().saturating_sub(1)))), Key::ArrowUp => Some(current.unwrap_or(0).saturating_sub(1)), Key::Home => Some(0), Key::End => Some(keys.len().saturating_sub(1)), _ => None };
            if let Some(next) = next.and_then(|next| keys.get(next)) {
                event.prevent_default(); on_select.call(next.clone());
                spawn(async move { let _=document::eval("await new Promise(requestAnimationFrame);document.querySelector('[role=treeitem][aria-selected=true]')?.focus();return true;").await; });
            }
        },
            for (installed,label) in [(true,"已安装"),(false,"可安装")] {
                details { class: "extension-browser__group", open: if installed { installed_open() } else { available_open() },
                    summary { onclick: move |event| { event.prevent_default(); if installed { installed_open.toggle(); } else { available_open.toggle(); } }, "{label} ({entries.iter().filter(|entry| entry.installed==installed).count()})" }
                    div { role: "group", for entry in entries.iter().filter(|entry| entry.installed==installed) {
                        button { key: "{entry.git}", class: "extension-browser__item", role: "treeitem", "aria-selected": selected.as_deref()==Some(&entry.git), tabindex: if selected.as_deref()==Some(&entry.git) { 0 } else { -1 },
                            onclick: { let git=entry.git.clone(); move |_| on_select.call(git.clone()) },
                            Package {} span { strong { "{entry.title}" } p { "{entry.summary}" } small { "{entry.state_label()}" } }
                        }
                    } }
                }
            }
        }
    }
}
