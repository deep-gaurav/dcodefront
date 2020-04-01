use yew::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use super::editor_src::AceEditor;
use super::file_manager::{FsEntry};
use super::socket_agent::{AgentInput,AgentOutput,SocketAgent,TransferData};
use lazy_static::lazy_static;
use std::collections::HashMap;
use maplit::hashmap;
use std::iter::DoubleEndedIterator;
use super::ide_home::EditorHost;
use yew::services::interval::{IntervalTask,IntervalService};

lazy_static! {
    static ref MODES: HashMap<String,String> = {
        let mut m = hashmap!{
            "py".to_string() => "ace/mode/python".to_string(),
            "js".to_string() => "ace/mode/javascript".to_string(),
            "css".to_string() => "ace/mode/css".to_string(),
            "html".to_string() => "ace/mode/html".to_string(),
            "jsx".to_string() => "ace/mode/jsx".to_string(),
            "ts".to_string() => "ace/mode/typescript".to_string(),
            "rs".to_string() => "ace/mode/rust".to_string(),
            "yml".to_string() => "ace/mode/yaml".to_string(),
            "toml".to_string() => "ace/mode/toml".to_string(),
        };
        m
    };
}

pub struct Editor{
    editor_ref:NodeRef,
    props:Props,
    editor:Option<AceEditor>,
    link:ComponentLink<Self>,
    bridge:Box<dyn yew::Bridge<SocketAgent>>,
    clean_interval_check:IntervalTask
}
pub enum Msg{
    SocketMessage(TransferData),
    CheckClean,
    Save,
    Ignore
}

#[derive(Clone,Properties,PartialEq)]
pub struct Props{
    pub file:FsEntry,
    pub host:EditorHost,
    pub clean_callback:Callback<(FsEntry,bool)>
}

impl Component for Editor{

    type Properties = Props;
    type Message = Msg;

    fn create(props: Self::Properties, link: ComponentLink<Self>)->Self{
        let propsc = props.file.clone();
        let mut bridge = SocketAgent::bridge(link.callback(move|out|{
            if let AgentOutput::SocketMessage(data)=out{
                Msg::SocketMessage(data)
            } else if let AgentOutput::SaveFile(file)=out{
                if file==propsc.clone(){
                    Msg::Save
                }else{
                    Msg::Ignore
                }
            }else{
                Msg::Ignore
            }
        }));
        bridge.send(AgentInput::SendData(
            TransferData{
                command:"fs".to_string(),
                value:"open".to_string(),
                args:vec![
                    props.file.path.to_string()
                ]
            }
        ));
        let task = IntervalService::new().spawn(std::time::Duration::from_secs(1),link.callback(|_|Msg::CheckClean));
        Self{
            editor_ref:NodeRef::default(),
            editor:None,
            props,
            bridge,
            link,
            clean_interval_check:task
        }
    }

    fn mounted(&mut self)->bool{
        let par:HtmlElement = self.editor_ref.cast().expect("Not HtmlElement");
        let editor=AceEditor::new(par);
        editor.set_mode_from_filename(&self.props.file.path);
        self.editor=Some(
            editor
        );
        false
    }

    fn update(&mut self,msg: Self::Message)->ShouldRender{
        match msg{
            Msg::Ignore=>false,
            Msg::SocketMessage(msg)=>{
                if msg.command=="fs"{
                    if msg.value=="open"{
                        if let Some(fname) = msg.args.get(0){
                            if self.props.file.path==fname.as_str(){
                                if let Some(editor)=&mut self.editor{
                                    if let Some(val)=msg.args.get(1){
                                        editor.set_value(&val);
                                        editor.markClean();
                                    }
                                }
                            }
                        }
                    }
                    if msg.value=="save"{
                        if let Some(fname)=msg.args.get(0){
                            if let Ok(ff)=serde_json::from_str::<FsEntry>(&fname){
                                if self.props.file==ff{
                                    if let Some(editor)=&self.editor{
                                        editor.markClean();
                                        self.props.clean_callback.emit((self.props.file.clone(),editor.isClean()))
                                    }

                                }
                            }
                        }
                    }
                }
                false
            },
            Msg::CheckClean=>{
                if let Some(editor)=&self.editor{
                    self.props.clean_callback.emit((self.props.file.clone(),editor.isClean()))
                }
                false
            }
            Msg::Save=>{
                if let Some(editor)=&self.editor{
                    if let Ok(ffs)=serde_json::to_string(&self.props.file){
                        let tdata = TransferData{
                            command:"fs".to_string(),
                            value:"save".to_string(),
                            args:vec![
                                ffs,editor.get_value()
                            ]
                        };
                        self.bridge.send(AgentInput::SendData(tdata));
                    }
                }
                false
            }
        }
    }

    fn view(&self)->Html{

        html!{
            <div>
                <div ref=self.editor_ref.clone() style="height:70vh">
                </div>
            </div>
        }
    }
}
