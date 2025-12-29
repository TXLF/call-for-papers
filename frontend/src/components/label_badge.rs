use yew::prelude::*;
use crate::types::Label;

#[derive(Properties, PartialEq)]
pub struct LabelBadgeProps {
    pub label: Label,
    #[prop_or_default]
    pub removable: bool,
    #[prop_or_default]
    pub on_remove: Option<Callback<String>>,
}

#[function_component(LabelBadge)]
pub fn label_badge(props: &LabelBadgeProps) -> Html {
    let label = &props.label;
    let background_color = label.color.clone().unwrap_or_else(|| "#6B7280".to_string());

    let on_remove_click = {
        let label_id = label.id.clone();
        let on_remove = props.on_remove.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(callback) = &on_remove {
                callback.emit(label_id.clone());
            }
        })
    };

    html! {
        <span class="label-badge" style={format!("background-color: {}", background_color)}>
            <span class="label-name">{ &label.name }</span>
            if props.removable {
                <button
                    class="label-remove-btn"
                    onclick={on_remove_click}
                    aria-label={format!("Remove {} label", label.name)}
                >
                    { "×" }
                </button>
            }
        </span>
    }
}
