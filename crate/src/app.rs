use yew::prelude::*;

use super::connect_form::{ConnectForm,WebSocketMeta};
use super::ide_home::AppHome;

pub struct App{
    component_link:ComponentLink<Self>,
    state:AppState
}

pub enum AppState{
    Disconnected,
    Connected(WebSocketMeta),

}

pub enum Msg{
    Connect(WebSocketMeta)
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {component_link:link,state:AppState::Disconnected}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg{
            Msg::Connect(meta)=>{
                self.state=AppState::Connected(meta)
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <section class="">
                <div>
                {
                    match &self.state{
                        AppState::Disconnected => {
                            html! {
                                <ConnectForm onconnect=self.component_link.callback(|meta|{
                                    log::info!("Connect");
                                    Msg::Connect(meta)
                                    })
                                />
                            }
                        }
                        AppState::Connected(meta)=>{
                            html!{
                                <AppHome meta=meta/>
                            }
                        }
                    }
                }
                </div>
            </section>
        }
    }
 }
