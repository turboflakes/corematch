use crate::app::App;
use crate::pages::page_not_found::PageNotFound;
use crate::runtimes::support::SupportedRelayRuntime;
use serde::{Deserialize, Serialize};
use yew::{html, Component, Context, Html};
use yew_router::{BrowserRouter, Routable, Switch};

#[derive(Routable, PartialEq, Eq, Clone, Debug)]
pub enum Routes {
    #[at("/")]
    Index,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Serialize, Deserialize)]
pub struct Query {
    pub chain: SupportedRelayRuntime,
}

pub struct Router;

impl Component for Router {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <Switch<Routes> render={switch} />
            </BrowserRouter>
        }
    }
}

fn switch(routes: Routes) -> Html {
    match routes {
        Routes::Index => {
            html! { <App /> }
        }
        Routes::NotFound => {
            html! { <PageNotFound /> }
        }
    }
}
