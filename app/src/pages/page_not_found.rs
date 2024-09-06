use crate::router::Routes;
use corematch_common::components::buttons::ActionButton;
use yew::{html, Callback, Component, Context, Html};
use yew_router::{prelude::use_navigator, scope_ext::RouterScopeExt};

pub struct PageNotFound;

impl Component for PageNotFound {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();
        let onclick = Callback::from(move |_| {
            navigator.push(&Routes::Index);
        });

        html! {
            <div class="page__not_found">
                <img class="corematch__icon" src="/images/corematch_icon_animated_page_not_found.svg" alt="page not found" />
                <div class="action">
                    <ActionButton label={"play corematch"} disable={false} onclick={onclick}>
                        <img class="icon" src="/images/start_icon_white_clear.svg" alt="start_icon" />
                    </ActionButton>
                </div>
            </div>
        }
    }
}
