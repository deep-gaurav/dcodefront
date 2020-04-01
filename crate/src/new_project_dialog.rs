use crate::ide_home::Panels;
use yew::prelude::*;
use std::collections::HashMap;
use maplit::hashmap;


#[derive(Debug,Clone,Default,PartialEq)]
pub struct ProjectConfig{
    pub name:String,
    pub config:Template,
    pub panels:Panels
}

#[derive(Clone,Debug,Default,PartialEq)]
pub struct Template{
    pub name:String,
    pub init:String
}

pub struct NewProjectDialog{
    pub props:Props,
    link:ComponentLink<Self>,
    name_ref:NodeRef,
    select_ref:NodeRef,
    templates:HashMap<String,Template>
}

#[derive(Clone,Properties)]
pub struct Props{
    pub oncreate:Callback<ProjectConfig>
}

pub enum Msg{
    Create
}

impl Component for NewProjectDialog{

    type Message = Msg;
    type Properties = Props;

    fn create(props:Props, link:ComponentLink<Self>) -> Self {

        let templates = hashmap!{
            "blank".to_string() => Template{
                name:"blank".to_string(),
                init:"mkdir -p /projects/{} && cd /projects/{}".to_string()
            },
            "react".to_string() => Template{
                name:"react".to_string(),
                init:"mkdir -p /projects/{} && cd /projects/{} && yarn create react-app .".to_string()
            }
        };

        NewProjectDialog{
            props,
            link,
            name_ref:NodeRef::default(),
            select_ref:NodeRef::default(),
            templates
        }
    }
    fn update(&mut self, msg: Msg) -> bool {

        match msg{
            Msg::Create=>{
                let inpel:web_sys::HtmlInputElement = self.name_ref.cast().expect("not inputelement");
                let selectel:web_sys::HtmlSelectElement = self.select_ref.cast().expect("not select element");
                if !inpel.value().is_empty() && !selectel.value().is_empty(){
                    self.props.oncreate.emit(
                        ProjectConfig{
                            name:inpel.value(),
                            config:self.templates[&selectel.value()].clone(),
                            panels:Panels{
                                file_manager:false,
                                terminal:true
                            }
                        }
                    )
                }
            }
        }

        false
    }
    fn view(&self) -> Html {


        let mut lists = html!();
        for temp in &self.templates{
            lists = html!(
                <>
                    {lists}
                    <option value={&temp.1.name}>{&temp.1.name}</option>
                </>
            )
        }
        html!{
            <div class="modal is-active">
              <div class="modal-background"></div>
              <div class="modal-card">
                <header class="modal-card-head">
                  <p class="modal-card-title">{"Create Project"}</p>
                </header>
                <section class="modal-card-body">
                    <div class="field">
                        <div class="control is-expanded">
                            <input ref=self.name_ref.clone() class="input" type="text" placeholder="Project Name"></input>
                        </div>
                        <div class="select">
                            <select ref=self.select_ref.clone()>
                                {
                                    lists
                                }
                            </select>
                        </div>
                    </div>
                </section>
                <footer class="modal-card-foot">
                  <button class="button is-success" onclick=self.link.callback(|_|Msg::Create)>{"Create"}</button>
                </footer>
              </div>
            </div>
        }
    }
}
