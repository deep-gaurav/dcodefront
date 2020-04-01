
use yew::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use std::convert::TryInto;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ MessageEvent, WebSocket, HtmlInputElement};
use super::socket_agent::{SocketAgent,AgentInput,AgentOutput};

#[derive(Clone,PartialEq)]
pub struct WebSocketMeta{
    // pub ws:WebSocket,
    pub url:String
}

pub struct ConnectForm {
    link: ComponentLink<Self>,
    onconnect: Callback<WebSocketMeta>,
    url_ref: NodeRef,
    is_connecting:bool,
    socket_agent:Box<yew::Bridge<SocketAgent>>
}

pub enum Msg {
    Clicked,
    Connected(WebSocketMeta),
    Error,
    Ignore
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub onconnect: Callback<WebSocketMeta>,
}

impl Component for ConnectForm {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let agent = SocketAgent::bridge(link.callback(|data|{
            match data{
                AgentOutput::Connected(url)=>Msg::Connected(WebSocketMeta{url}),
                AgentOutput::Disconnected | AgentOutput::ErrorConnecting => Msg::Error,
                _=>Msg::Ignore
            }
        }));
        ConnectForm {
            link,
            onconnect: props.onconnect,
            url_ref: NodeRef::default(),
            is_connecting:false,
            socket_agent:agent
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked => {
                // self.onconnect.emit(());

                let el:HtmlInputElement=self.url_ref.cast().expect("Not input Element");
                // let ws = WebSocket::new(&format!("ws://{}/ws",el.value()));
                // self.url_ref.cast::<Self>();
                self.socket_agent.send(AgentInput::ConnectSocket(el.value()));
                self.is_connecting=true;
            }
            Msg::Connected(meta)=>{
                self.onconnect.emit(meta);
            }
            Msg::Error=>{
                self.is_connecting=false;
            }
            Msg::Ignore=>{
                return false;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.onconnect = props.onconnect;
        true
    }

    fn view(&self) -> Html {
        let mut classs= "button is-info".to_owned();
        if self.is_connecting{
            classs+=" is-loading";
        }
        html! {


            <div class="field has-addons">
                <div class="control is-expanded">
                    <input ref=self.url_ref.clone() class="input" type="text" placeholder="Backend URL"></input>
                </div>
                <div class="control">
                    <a class=classs onclick=self.link.callback(|_|Msg::Clicked)>
                        {"Connect"}
                    </a>
                </div>
            </div>

        }
    }
}
