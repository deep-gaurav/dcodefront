use crate::ide_home::Panels;
use yew::prelude::*;
use std::collections::HashMap;
use maplit::hashmap;
use super::file_manager::FsEntry;


pub struct NewFile{
    pub props:Props,
    link:ComponentLink<Self>,
    name_ref:NodeRef,
    select_ref:NodeRef
}

#[derive(Clone,Properties)]
pub struct Props{
    pub oncreate:Callback<FsEntry>,
    pub onclose:Callback<()>
}

pub enum Msg{
    Create,
    Cancel
}

impl Component for NewFile{

    type Message = Msg;
    type Properties = Props;

    fn create(props:Props, link:ComponentLink<Self>) -> Self {

        Self{
            props,
            link,
            name_ref:NodeRef::default(),
            select_ref:NodeRef::default(),
        }
    }
    fn update(&mut self, msg: Msg) -> bool {

        match msg{
            Msg::Create=>{
                let inpel:web_sys::HtmlInputElement = self.name_ref.cast().expect("not inputelement");
                let selectel:web_sys::HtmlSelectElement = self.select_ref.cast().expect("not select element");
                if !inpel.value().is_empty() && !selectel.value().is_empty(){
                    let is_dir;
                    if selectel.value()=="file"{
                        is_dir=false;
                    }else{
                        is_dir=true;
                    }
                    self.props.oncreate.emit(
                        FsEntry{
                            name:inpel.value(),
                            path:String::default(),
                            is_dir
                        }
                    )
                }
            }
            Msg::Cancel=>self.props.onclose.emit(())
        }

        false
    }
    fn view(&self) -> Html {

        html!{
            <div class="modal is-active">
              <div class="modal-background"></div>
              <div class="modal-card">
                <header class="modal-card-head">
                  <p class="modal-card-title">{"New"}</p>
                </header>
                <section class="modal-card-body">
                    <div class="field">
                        <div class="control is-expanded">
                            <input ref=self.name_ref.clone() class="input" type="text" placeholder="File/Directory Name"></input>
                        </div>
                        <div class="select">
                            <select ref=self.select_ref.clone()>
                                <option value="file">{"File"}</option>
                                <option value="folder">{"Directory"}</option>
                            </select>
                        </div>
                    </div>
                </section>
                <footer class="modal-card-foot">
                  <button class="button" onclick=self.link.callback(|_|Msg::Cancel)>{"Cancel"}</button>
                  <button class="button is-success" onclick=self.link.callback(|_|Msg::Create)>{"Create"}</button>
                </footer>
              </div>
            </div>
        }
    }
}
