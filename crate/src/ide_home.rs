use serde::{Deserialize, Serialize};
use yew::prelude::*;

use super::new_project_dialog::{NewProjectDialog, ProjectConfig};
use maplit::hashmap;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, MessageEvent, WebSocket};

use super::connect_form::WebSocketMeta;
use super::socket_agent::TransferData;
use super::terminal::TerminalComp;
use super::file_manager::{FileManager,FsEntry};
use super::editor::Editor;
use super::socket_agent::{AgentInput, AgentOutput, SocketAgent};

pub struct AppHome {
    meta: WebSocketMeta,
    project_config: Option<ProjectConfig>,
    link: ComponentLink<Self>,
    state: ConnectedStateData,
    socket_agent: Box<dyn yew::Bridge<SocketAgent>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub meta: WebSocketMeta,
}

#[derive(Debug,Default,Clone,PartialEq)]
pub struct Panels{
    pub file_manager:bool,
    pub terminal:bool
}

#[derive(Clone,PartialEq)]
pub struct TerminalHost {
    pub title: String,
    pub is_active: bool,
    pub init_dir: String,
    pub init_cmd: String,
    // terminal: Html,
}

#[derive(Clone,PartialEq)]
pub struct EditorHost {
    pub file: FsEntry,
    pub is_active: bool,
    pub is_clean:bool
}

#[derive(Clone, Default)]
pub struct ConnectedStateData {
    ping: f64,
    terminals: HashMap<String, TerminalHost>,
    openfiles: Vec<EditorHost>
}

pub enum Msg {
    PingUpdate(f64),
    CreateProject(ProjectConfig),
    SocketMessage(TransferData),
    SwitchTerminal(String),
    RenameTerminal(String,String),
    Update,
    Ignore,
    CreateTerm(String),
    RemoveTerm(String),
    SetPanels(Panels),
    OpenFile(FsEntry),
    CleanStatus((FsEntry,bool)),
    CloseFile(EditorHost),
    SwitchFile(EditorHost),
    Save
}

impl AppHome {
    fn send_ping(&mut self) {
        let date = js_sys::Date::new_0();
        let msec = date.get_time();

        let pingdata = TransferData {
            command: "ping".to_string(),
            value: "0".to_string(),
            args: vec![format!("{}", msec)],
        };

        self.socket_agent.send(AgentInput::SendData(pingdata));
    }

    fn send_data(&mut self, data: TransferData) {
        self.socket_agent.send(AgentInput::SendData(data));
    }

    fn handle_data(&mut self, data: TransferData) {
        match data.command.as_str() {
            "ping" => {
                let p: f64 = data.args[1].parse().expect("Not f64");
                let date = js_sys::Date::new_0();
                let msec = date.get_time();

                // log::info!("ping {}",msec-p);
                self.send_ping();
                self.link.send_message(Msg::PingUpdate(msec - p))
            }
            "process" => match data.value.as_str() {
                "list" => {
                    let mut to_remove = vec![];
                    for k in self.state.terminals.keys() {
                        if data.args.iter().position(|p| p == k).is_none() {
                            to_remove.push(k.clone());
                        }
                    }
                    if !to_remove.is_empty() {
                        log::debug!(
                            "Removing terms {:?} received list {:?}",
                            to_remove,
                            data.args
                        );
                        for e in to_remove {
                            self.state.terminals.remove(&e);
                        }
                        self.link.send_message(Msg::Update);
                    }
                }
                _ => log::info!("Unknown process value {:?}", data),
            },
            "exec"=>{
                // match self.state.terminals.g
            },
            "fs" => {

            },
            _cmd => {
                log::info!("Unknown command {:#?}", data);
            }
        }
    }
}

impl Component for AppHome {
    type Properties = Props;
    type Message = Msg;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let propsclone = props.clone();
        let socket_agent = SocketAgent::bridge(link.callback(|data| match data {
            AgentOutput::SocketMessage(message) => Msg::SocketMessage(message),
            _ => Msg::Ignore,
        }));
        let adt = AppHome {
            meta: props.meta,
            link,
            state: ConnectedStateData::default(),
            project_config: None,
            socket_agent,
        };
        adt
    }

    fn mounted(&mut self) -> bool {
        self.send_ping();
        false
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::PingUpdate(ping) => {
                self.state.ping = ping;
                true
            }
            Msg::CreateProject(project) => {
                log::info!("name {:#?}", project);
                self.link.send_message(Msg::CreateTerm(format!("{}",project.config.init.replace("{}",&project.name))));
                self.project_config = Some(project);
                true
            }
            Msg::CreateTerm(init_cmd) => {
                let terms = self.state.terminals.len();
                for term in &mut self.state.terminals {
                    term.1.is_active = false;
                }
                let termid = format!("term{}",terms+1);
                // log::debug!("creating term {}",termid);
                self.state.terminals.insert(
                     termid.clone(),
                    TerminalHost {
                        is_active: true,
                        title: termid.clone(),
                        init_dir: format!("/projects/{}",self.project_config.clone().unwrap_or_default().name),
                        init_cmd
                        // terminal: html!(<TerminalComp termid={termid.clone()} />),
                    },
                );
                // log::debug!("created term {}",termid.clone());


                self.send_data(TransferData {
                    command: "process".to_string(),
                    value: "new".to_string(),
                    args: vec![format!("term{}", terms + 1)],
                });
                true
            }
            Msg::SwitchTerminal(termid)=>{
                let terms = self.state.terminals.len();
                for term in &mut self.state.terminals {
                    term.1.is_active = false;
                }
                if let Some(term)=self.state.terminals.get_mut(&termid){
                    term.is_active=true;
                }

                true
            }
            Msg::Update => true,
            Msg::Ignore => false,
            Msg::SocketMessage(msg) => {
                self.handle_data(msg);
                false
            }
            Msg::RemoveTerm(termid) => {
                self.state.terminals.remove(&termid);
                let mut has_active = false;
                for term in &self.state.terminals{
                    if term.1.is_active{
                        has_active=true;
                        break;
                    }
                }
                if !has_active{
                    if let Some(term)=self.state.terminals.iter_mut().next(){
                        term.1.is_active=true;
                    }
                }
                true
            }
            Msg::RenameTerminal(termid,title) => {
                if let Some(term) = self.state.terminals.get_mut(&termid){
                    term.title=title;
                }
                true
            }
            Msg::SetPanels(panels)=>{
                if let Some(config) = &mut self.project_config{
                    config.panels=panels;
                    true
                }else{
                    false
                }
            }
            Msg::OpenFile(file)=>{
                // log::debug!("Open file {:#?}",file);
                if let None = self.state.openfiles.iter().position(|f|&f.file==&file){
                    for f in &mut self.state.openfiles{
                        f.is_active=false;
                    }
                    // log::debug!("Create file {:#?}",file);

                    self.state.openfiles.push(EditorHost{
                        is_active:true,
                        file:file,
                        is_clean:true
                    });
                }
                true
            }
            Msg::CloseFile(file)=>{
                if let Some(index) = self.state.openfiles.iter().position(|f|f==&file){
                    self.state.openfiles.remove(index);
                }
                let mut has_active=false;
                for f in &mut self.state.openfiles{
                    if f.is_active {
                        has_active=true;
                        break;
                    }
                }
                if !has_active{
                    if let Some(editor) = self.state.openfiles.iter_mut().next(){
                        editor.is_active=true;
                    }
                }
                true
            }
            Msg::SwitchFile(file)=>{
                for f in &mut self.state.openfiles{
                    f.is_active=false;
                }
                if let Some(file)= self.state.openfiles.iter().position(|f|f==&file){
                    if let Some(file)=self.state.openfiles.get_mut(file){
                        file.is_active=true;
                    }
                }
                true
            }
            Msg::CleanStatus(file)=>{
                if let Some(index) = self.state.openfiles.iter().position(|f|&f.file==&file.0){
                    if self.state.openfiles[index].is_clean!=file.1{
                        self.state.openfiles[index].is_clean=file.1;
                        true
                    }else{
                        false
                    }
                }else{
                    false
                }
            }
            Msg::Save=>{
                for ed in &self.state.openfiles{
                    if ed.is_active{
                        self.socket_agent.send(AgentInput::SaveFile(ed.file.clone()));
                    }
                }
                false
            }
        }

        // false
    }

    fn view(&self) -> Html {
        let terminal_tabs = html!{
            <>
                { for self.state.terminals.iter().map(|tab| {
                      let title = tab.0.clone();
                      let title1 = tab.0.clone();
                      html! {
                          <li key=tab.0.clone() class={
                              if tab.1.is_active{
                                  "is-active"
                              }else{
                                  ""
                              }
                          }>
                            <a onclick=self.link.callback(move |_|Msg::SwitchTerminal(title.clone()))>{tab.1.title.clone()} <span onclick=self.link.callback(move |_|Msg::RemoveTerm(title1.clone())) class="delete is-small"></span></a>
                          </li>

                      }
                    })
                }
            </>
        };
        let terminals = html!{
            <div>
            { for self.state.terminals.iter().map(|tab| {
                let id = tab.0.clone();
                  html! {
                      <div key=tab.0.clone() class={
                          if tab.1.is_active{
                              ""
                          }else{
                              "is-hidden"
                          }
                      }>
                          <TerminalComp  thost=tab.1.clone() title_change=self.link.callback(move |data:String|Msg::RenameTerminal(id.clone(),data.clone()))  termid=tab.1.title.clone()/>
                      </div>
                  }
                })
            }
            </div>
        };

        let mut active_editor=false;

        for ed in &self.state.openfiles{
            if ed.is_active{
                active_editor=true;
                break;
            }
        }


        html! {
            <div>

                <div class="level is-mobile">
                    <div class="level-left">
                        <div class="tags has-addons level-item">
                            <span class="tag">{"Ping"}</span>
                            <span class="tag is-primary">{self.state.ping}{"ms"}</span>
                        </div>
                    </div>
                    <div class="level-right">
                        <div class="level-item">
                            {match &self.project_config{
                                Some(config)=>{
                                    let panels = config.panels.clone();
                                    let panels_terminal = config.panels.clone();
                                    html!{
                                        <div>
                                        <button class="button" disabled={!active_editor} onclick=self.link.callback(
                                            move|_|Msg::Save
                                        )>
                                          <span class="icon is-small">
                                            <i class="fas fa-save"></i>
                                          </span>
                                        </button>

                                        <button class="button" onclick=self.link.callback(
                                            move|_|Msg::SetPanels(
                                                Panels{
                                                    file_manager:!panels.file_manager,
                                                    ..panels.clone()
                                                }
                                            )
                                        )>
                                          <span class="icon is-small">
                                            <i class="fas fa-folder"></i>
                                          </span>
                                        </button>

                                        <button class="button" onclick=self.link.callback(
                                            move|_|Msg::SetPanels(
                                                Panels{
                                                    terminal:!panels_terminal.terminal,
                                                    ..panels_terminal.clone()
                                                }
                                            )
                                        )>
                                          <span class="icon is-small">
                                            <i class="fas fa-terminal"></i>
                                          </span>
                                        </button>
                                        </div>
                                    }
                                }
                                None=>html!()
                            }}
                        </div>
                    </div>
                </div>
                {
                    match &self.project_config{
                        Some(config)=>{
                            html!(
                                <div class="media" style="overflow:auto;">
                                    <div class="media-left">
                                        <div class="">
                                            <FileManager project_config=config.clone() open_file=self.link.callback(|file|Msg::OpenFile(file)) />
                                        </div>
                                    </div>
                                    <div class="media-content">
                                        <div class="">
                                            <div class="container">
                                                <div class="tabs level-item is-boxed">
                                                    <ul>
                                                    { for self.state.openfiles.iter().map(|file| {
                                                          let file = file.clone();
                                                          let file_switch = file.clone();
                                                          let file_close = file.clone();
                                                          html! {
                                                              <li key=file.file.path.clone() class={
                                                                  if file.is_active{
                                                                      "is-active"
                                                                  }else{
                                                                      ""
                                                                  }
                                                              }>
                                                                <a onclick=self.link.callback(move |_|Msg::SwitchFile(file_switch.clone()))>{file.file.name.clone()}{
                                                                    if file.is_clean{
                                                                        html!{

                                                                        }
                                                                    }else{
                                                                        html!{
                                                                            <sup>{"*"}</sup>
                                                                        }
                                                                    }
                                                                } <span onclick=self.link.callback(move |_|Msg::CloseFile(file_close.clone())) class="delete is-small"></span></a>
                                                              </li>

                                                          }
                                                        })
                                                    }
                                                    </ul>
                                                </div>
                                                <div class="">
                                                    { for self.state.openfiles.iter().map(|file| {
                                                          let file = file.clone();
                                                          html! {
                                                              <div key=file.file.path.clone() class={
                                                                  if file.is_active{
                                                                      ""
                                                                  }else{
                                                                      "is-hidden"
                                                                  }
                                                              }>
                                                                <Editor host=file.clone() file=file.file.clone() clean_callback=self.link.callback(|file|Msg::CleanStatus(file))/>
                                                              </div>

                                                          }
                                                        })
                                                    }
                                                </div>

                                                // <Editor />
                                                <div class={
                                                    if config.panels.terminal{
                                                        ""
                                                    }else{
                                                        "is-hidden"
                                                    }
                                                }>
                                                <div class="level is-mobile" style="overflow:auto;">
                                                    <div class="level-left" >
                                                        <div class="tabs level-item is-boxed">
                                                            <ul>
                                                                {terminal_tabs}
                                                            </ul>
                                                        </div>
                                                    </div>
                                                    <div class="level-right">
                                                        <div class="level-item field has-addons">
                                                            <button onclick=self.link.callback(|_|Msg::CreateTerm("".to_string())) class="button">
                                                                {"+"}
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                                {terminals}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            )
                        }
                        None => {
                            html!(
                                <NewProjectDialog oncreate=self.link.callback(|config|Msg::CreateProject(config))/>
                            )
                        }
                    }
                }
            </div>
        }
    }
}
