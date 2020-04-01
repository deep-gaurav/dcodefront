use crate::socket_agent::TransferData;
use web_sys::HtmlElement;
use yew::prelude::*;
use super::terminal_src::Terminal;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use super::socket_agent::{AgentInput,AgentOutput,SocketAgent};

pub struct TerminalComp{
    link:ComponentLink<Self>,
    divref:NodeRef,
    props:Props,
    terminal:Option<Terminal>,
    socket_agent:Box<dyn yew::Bridge<SocketAgent>>
}

#[derive(Properties,Clone,PartialEq)]
pub struct Props{
    pub termid:String,
    pub thost:super::ide_home::TerminalHost,
    pub title_change:Callback<String>
}

pub enum Msg{
    Input(String),
    Write(String),
    Ignore
}

fn get_max_text(el:&web_sys::Element)->Result<u16,JsValue>{
    let window = yew::utils::window();
    let style=window.get_computed_style(el)?;
    if let Some(style)=style{
        // let reg = regex::Regex::new(r"\d+").expect("Not regex");
        let font_size = style.get_property_value("font-size")?;
        let font_size = &font_size[..font_size.len()-2];
        let width = style.get_property_value("width")?;
        let width = &width[..width.len()-2];
        let font_size:u16 = {
            // let capt = reg.captures(&font_size).expect("Cant capture");
            (font_size.parse::<f64>().expect("Not u32") * 0.625).floor() as u16
        };
        let width:u16 = {
            // let capt = reg.captures(&width).expect("Cant capture");
            width.parse().expect("Not u32")
        };

        return Ok(width/font_size)

    }
    Err(JsValue::from_str("cant get computed style"))
}

impl Component for TerminalComp{

    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>)->Self{

        log::debug!("new term {} created",props.termid);

        let term_id = props.termid.clone();
        let socket_agent = SocketAgent::bridge(link.callback(move |data:AgentOutput|{
            if let AgentOutput::SocketMessage(msg)=data{
                if msg.command=="exec" && msg.value==term_id.clone(){
                    if let Some(out)=msg.args.get(1){
                        return Msg::Write(out.to_string());
                    }
                }
            }
            Msg::Ignore
        }));
        TerminalComp{
            link,
            divref:NodeRef::default(),
            props,
            terminal:None,
            socket_agent
        }
    }

    fn update(&mut self,msg: Self::Message)->bool{
        match msg{
            Self::Message::Input(data)=>{
                log::info!("input {}",data);
                self.socket_agent.send(AgentInput::SendData(
                    TransferData{
                        command:"exec".to_string(),
                        value:self.props.termid.clone(),
                        args:vec![data]
                    }
                ))
            }
            Self::Message::Write(data)=>{
                if let Some(term)=&mut self.terminal{
                    term.write(&data);
                    let title = term.get_title();
                    if !title.is_empty() && self.props.thost.title == title{
                        self.props.title_change.emit(title);
                    }
                }
            }
            Self::Message::Ignore=>{
                return false;
            }
        }

        true
    }

    fn change(&mut self,_props: Self::Properties)->bool{
        // log::debug!("term {} changed {}",self.props.termid,_props.termid);
        if self.props != _props{
            self.props = _props;
            true
        }else{
            false
        }

    }

    fn destroy(&mut self){
        log::debug!("term {} destroyed",self.props.termid);

    }

    fn mounted(&mut self)->bool{
        // log::debug!("new term {} mounted",self.props.termid);

        let divel = self.divref.cast::<HtmlElement>().expect("not htmlelement");
        let el = self.divref.cast::<web_sys::Element>().expect("Not Element");
        let max_line = get_max_text(&el).unwrap_or(24) as u16 -2;
        log::info!("detected term cols {}",max_line);
        self.socket_agent.send(AgentInput::SendData(
            TransferData{
                command:"process".to_string(),
                value:"resize".to_string(),
                args:vec![
                    self.props.termid.clone(),
                    "80".to_string(),
                    format!("{}",max_line)
                ]
            }
        ));

        match Terminal::new(divel,self.link.callback(|data|Msg::Input(data)),max_line){
            Ok(term)=>{
                let data;
                if self.props.thost.init_cmd.is_empty(){
                    data=format!("cd {} \n",self.props.thost.init_dir);
                }else{
                    data=format!("cd / && {} && cd {} \n",self.props.thost.init_cmd,self.props.thost.init_dir)
                }

                self.socket_agent.send(AgentInput::SendData(
                    TransferData{
                        command:"exec".to_string(),
                        value:self.props.termid.clone(),
                        args:vec![data]
                    }
                ));
                self.terminal=Some(term);

            }
            Err(err)=>{
                log::error!("Cannot create term {:#?}",err);
            }
        }

        false
    }

    fn view(&self)->Html{

        html!{
            <div style="overflow:auto;" ref=self.divref.clone()></div>
        }
    }
}
