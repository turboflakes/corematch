use crate::block::BlockNumber;
use gloo::timers::callback::Interval;
use yew::{classes, html, Component, Context, Html, Properties};

const SIX_SECS_TARGET: u32 = 6;

pub enum Msg {
    Update,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub block_number: Option<BlockNumber>,
    pub visible: bool,
}

pub struct BlockTimer {
    seconds: u32,
    milliseconds: u32,
    interval: Option<Interval>,
}

impl BlockTimer {
    fn reset(&mut self) {
        self.seconds = SIX_SECS_TARGET;
        self.milliseconds = 0;
    }
}

impl Component for BlockTimer {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let interval_handle = {
            let link = ctx.link().clone();
            Interval::new(100, move || link.send_message(Msg::Update))
        };

        Self {
            seconds: SIX_SECS_TARGET,
            milliseconds: 0,
            interval: Some(interval_handle),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update => {
                if self.seconds == 0 && self.milliseconds == 0 {
                    return false;
                }

                if self.milliseconds == 0 {
                    self.milliseconds = 9;
                    self.seconds -= 1;
                } else {
                    self.milliseconds -= 1;
                }
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.reset();
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let visible_class = if ctx.props().visible {
            Some("visible")
        } else {
            Some("hidden")
        };
        html! {
            <span class={classes!("countdown", visible_class)}>{"timer: "}<b>{format!(" {}.{}s", self.seconds, self.milliseconds)}</b></span>
        }
    }
}
