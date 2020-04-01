use std::rc::Rc;
use std::cell::RefCell;
use web_sys::MessageEvent;
use web_sys::WebSocket;
use yew::agent::HandlerId;
use yew::agent::Context;
use yew::agent::Agent;
use yew::worker::AgentLink;
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use super::file_manager::FsEntry;

use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct TransferData{
    pub command:String,
    pub value:String,
    pub args:Vec<String>
}

#[derive(Serialize,Deserialize)]
pub enum AgentInput{
    ConnectSocket(String),
    SendData(TransferData),
    SaveFile(FsEntry)
}

#[derive(Serialize,Deserialize)]
pub enum AgentOutput{
    Connected(String),
    ErrorConnecting,
    Disconnected,
    SocketMessage(TransferData),
    SaveFile(FsEntry)
}

pub struct SocketAgent{
    link: AgentLink<Self>,
    subscribers: Vec<HandlerId>,
    socket:Option<WebSocket>,
    updatecallback:Callback<(WebSocket,String)>,
    socketmessagecallback:Callback<TransferData>
}

pub enum Msg{
    Connected((WebSocket,String)),
    SocketMessage(TransferData)
}


impl Agent for SocketAgent{

    type Reach = Context;
    type Message = Msg;
    type Input = AgentInput;
    type Output = AgentOutput;

    fn create(link: AgentLink<Self>)->Self{

        SocketAgent{
            updatecallback:link.callback(|sock|Msg::Connected(sock)),
            socketmessagecallback:link.callback(|msg|Msg::SocketMessage(msg)),
            link,
            socket:None,
            subscribers:vec![],
        }
    }

    fn connected(&mut self,id: HandlerId){
        self.subscribers.push(id);
    }

    fn disconnected(&mut self,_id: HandlerId){
        if let Some(idx)=self.subscribers.iter().position(|id|id==&_id){
            self.subscribers.remove(idx);
        }
    }

    fn update(&mut self,msg: Self::Message){
        match msg{
            Msg::Connected(socket)=>{

                let subscribers = self.subscribers.clone();
                let linkclone = self.link.clone();
                let msgcallback = self.socketmessagecallback.clone();
                let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                    // handle message
                    let response = e
                        .data()
                        .as_string()
                        .expect("Can't convert received data to a string");
                    let tdata = serde_json::from_str::<TransferData>(&response);
                    match tdata{
                        Ok(tdata)=>{
                            msgcallback.emit(tdata);
                        }
                        Err(err)=>{
                            log::error!("{}",err);
                        }
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                socket.0.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

                onmessage_callback.forget();
                self.socket=Some(socket.0);
                for subs in self.subscribers.iter(){
                    self.link.respond(subs.clone(), AgentOutput::Connected(socket.1.clone()))
                }

            },
            Msg::SocketMessage(msg)=>{
                for subs in self.subscribers.iter(){
                    self.link.respond(subs.clone(), AgentOutput::SocketMessage(msg.clone()))
                }
            }
        }
    }

    fn handle_input(&mut self,msg: Self::Input, _id: HandlerId){
        match msg{
            Self::Input::ConnectSocket(url)=>{
                let subscribers = self.subscribers.clone();
                let linkclone = self.link.clone();

                let onerror_callback = Closure::wrap(Box::new(move |_| {
                    for subs in subscribers.clone(){
                        linkclone.clone().respond(subs, AgentOutput::ErrorConnecting)
                    }
                }) as Box<dyn FnMut(JsValue)>);


                let ws = WebSocket::new(&format!("ws://{}/ws",url));
                match ws{
                    Ok(ws)=>{
                        let wss= ws.clone();
                        let updatecallback = self.updatecallback.clone();
                        let onopen_callback = Closure::wrap(Box::new(move |_| {
                            updatecallback.emit((wss.clone(),url.clone()));
                        }) as Box<dyn FnMut(JsValue)>);

                        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                        onopen_callback.forget();

                        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                        onerror_callback.forget();
                    }
                    Err(e)=>{
                        log::debug!("Cannot connect {:#?}",e);
                        for subs in &self.subscribers{
                            self.link.respond(subs.clone(), AgentOutput::ErrorConnecting)
                        }
                    }
                }
            }
            Self::Input::SendData(data)=>{
                match &mut self.socket{
                    Some(socket)=>{

                        match serde_json::to_string(&data){
                            Ok(data)=>{
                                if let Err(err)=socket.send_with_str(&data){
                                    log::error!("{:?}",err);
                                }
                            }
                            Err(err)=>{
                                log::error!("{:?}",err);
                            }
                        }
                    }
                    None=>log::debug!("Trying to send data without connection {:#?}",data)
                }
            }
            Self::Input::SaveFile(file)=>{
                for subs in &self.subscribers{
                    self.link.respond(subs.clone(), AgentOutput::SaveFile(file.clone()))
                }
            }
        }
    }
}
