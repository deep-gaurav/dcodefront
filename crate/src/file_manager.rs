use yew::prelude::*;
use super::new_project_dialog::ProjectConfig;

use super::socket_agent::{AgentInput,AgentOutput,SocketAgent,TransferData};
use serde::{Serialize,Deserialize};
use yew::services::{IntervalService};
use yew::services::interval::IntervalTask;
use super::new_file::NewFile;

pub struct FileManager {
    link:ComponentLink<Self>,
    props:Props,
    agent_bridge:Box<dyn yew::Bridge<SocketAgent>>,
    current_dir:String,
    entries:Vec<FsEntry>,
    refreshInterval:IntervalTask,
    open_new_dialog:bool
}

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
pub struct FsEntry {
    pub is_dir:bool,
    pub name:String,
    pub path:String
}


pub enum Msg {
    SocketMessage(TransferData),
    Refresh,
    OpenFolder(String),
    OpenFile(FsEntry),
    OpenNewDialog,
    CreateFile(FsEntry),
    CloseNewDialog,
    Ignore
}

#[derive(Properties,Clone,PartialEq)]
pub struct Props {
    pub project_config:ProjectConfig,
    pub open_file:Callback<FsEntry>
}

impl Component for FileManager{

    type Properties = Props;
    type Message = Msg;

    fn create(props: Self::Properties, link: ComponentLink<Self>)->Self{

        let agent_bridge=SocketAgent::bridge(link.callback(|data|match data{
            AgentOutput::SocketMessage(data)=>Msg::SocketMessage(data),
            _ => Msg::Ignore
        }));
        let task = IntervalService::new().spawn(std::time::Duration::from_secs(1), link.callback(|_|Msg::Refresh));

        Self{

            current_dir:format!("/projects/{}",props.project_config.name),
            link,
            props,
            agent_bridge,
            entries:vec![],
            refreshInterval:task,
            open_new_dialog:false
        }
    }

    fn change(&mut self,_props: Self::Properties)->bool{
        if self.props!=_props{
            self.props=_props;
            true
        }
        else{
            false
        }
    }

    fn update(&mut self,msg: Self::Message)->ShouldRender{


        match msg{
            Msg::Ignore=>false,
            Msg::SocketMessage(data)=>{
                if data.command=="fs" && data.value=="list"{
                    let entries:Vec<FsEntry> = data.args.iter().filter_map(|entry|serde_json::from_str(entry).ok()).collect();
                    self.entries=entries;
                    true
                }else{
                    false
                }
            }
            Msg::OpenFolder(path)=>{
                self.current_dir=path.clone();
                self.agent_bridge.send(AgentInput::SendData(
                    TransferData{
                        command:"fs".to_string(),
                        value:"list".to_string(),
                        args:vec![path]
                    }
                ));
                false
            }
            Msg::OpenFile(file)=>{
                self.props.open_file.emit(file);
                false
            }
            Msg::Refresh=>{
                self.agent_bridge.send(AgentInput::SendData(
                    TransferData{
                        command:"fs".to_string(),
                        value:"list".to_string(),
                        args:vec![self.current_dir.to_string()]
                    }
                ));
                false
            }
            Msg::OpenNewDialog=>{
                self.open_new_dialog=true;
                true
            }
            Msg::CloseNewDialog=>{
                self.open_new_dialog=false;
                true
            }
            Msg::CreateFile(mut file)=>{
                if file.path.ends_with("/"){
                    file.path=format!("{}{}",self.current_dir,file.name);
                }else{
                    file.path=format!("{}/{}",self.current_dir,file.name);
                }
                let tdata = {
                    if file.is_dir{
                        TransferData{
                            command:"fs".to_string(),
                            value:"new_dir".to_string(),
                            args:vec![file.path]
                        }
                    }else{
                        let ffs = serde_json::to_string(&file).expect("Cant convert to string");
                        TransferData{
                            command:"fs".to_string(),
                            value:"new".to_string(),
                            args:vec![
                                ffs
                            ]
                        }
                    }
                };
                self.agent_bridge.send(AgentInput::SendData(tdata));
                self.open_new_dialog=false;
                false
            }
        }
    }

    fn mounted(&mut self)->ShouldRender{
        self.agent_bridge.send(AgentInput::SendData(
            TransferData{
                command:"fs".to_string(),
                value:"list".to_string(),
                args:vec![self.current_dir.clone()]
            }
        ));
        false
    }

    fn view(&self) -> Html{

        if !self.props.project_config.panels.file_manager{
            html!{
                <div>
                </div>
            }
        }else{
            html!{
                <nav class="panel">
                    {
                        if self.open_new_dialog{
                            html!{
                                <NewFile oncreate=self.link.callback(|file|Msg::CreateFile(file)) onclose=self.link.callback(|_|Msg::CloseNewDialog) />
                            }
                        }else{
                            html!{

                            }
                        }
                    }
                    <p class="panel-heading">{"File Manager"}</p>
                    <div class="panel-block">
                      <p class="control level is-mobile">
                          <a class="level-left" onclick=self.link.callback(|_|Msg::OpenNewDialog)>
                            <span class="icon is-small level-item">
                              <i class="fas fa-plus"></i>
                            </span>
                            <span class="level-item">
                                {"New"}
                            </span>
                          </a>
                      </p>
                    </div>
                    { for self.entries.iter().map(|entry| {
                        let entry = entry.clone();
                        let entry_callback = entry.clone();
                          html! {
                              <a class="panel-block" key=entry.path onclick={
                                  if entry_callback.clone().is_dir{
                                      self.link.callback(move |_|Msg::OpenFolder(entry_callback.path.clone()))
                                  }else{
                                      self.link.callback(move |_|Msg::OpenFile(entry_callback.clone()))
                                  }
                              }>
                              <span class="panel-icon">
                                <i class={
                                    if entry.is_dir{
                                        "fas fa-folder"
                                    }else{
                                        "fas fa-file"
                                    }
                                } aria-hidden="true"></i>
                              </span>
                              {
                                  entry.name
                              }
                            </a>
                          }
                        })
                    }
                </nav>
            }
        }

    }

}
