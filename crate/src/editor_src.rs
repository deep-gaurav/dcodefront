use web_sys::HtmlElement;
use wasm_bindgen::prelude::*;
use serde::Serialize;

pub struct AceEditor{
    editor:AceEditorJS
}


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ace, js_name = edit)]
    fn open_editor(s: JsValue)->AceEditorJS;


    #[wasm_bindgen(js_namespace = ace)]
    fn require(module:&str)->Module;
}
#[wasm_bindgen]
extern "C" {
    fn name() -> String;

    type AceEditorJS;
    type AceSession;
    type UndoManager;
    type CommandManager;
    type Mode;
    type Module;
    type ModeList;

    #[wasm_bindgen(method)]
    pub fn setTheme(this: &AceEditorJS,theme:&str);

    #[wasm_bindgen(method)]
    pub fn resize(this: &AceEditorJS);

    #[wasm_bindgen(method)]
    pub fn setValue(this: &AceEditorJS, value:&str);

    #[wasm_bindgen(method)]
    pub fn getValue(this: &AceEditorJS)->String;

    #[wasm_bindgen(method,getter)]
    pub fn session(this: &AceEditorJS)->AceSession;

    #[wasm_bindgen(method,getter)]
    pub fn commands(this: &AceEditorJS)->CommandManager;

    #[wasm_bindgen(method)]
    pub fn setMode(this: &AceSession, mode: &str);

    #[wasm_bindgen(method)]
    pub fn setOptions(this: &AceEditorJS, option:JsValue);

    #[wasm_bindgen(method)]
    pub fn setUseWrapMode(this: &AceSession, mode: bool);

    #[wasm_bindgen(method)]
    pub fn getUndoManager(this: &AceSession)->UndoManager;

    #[wasm_bindgen(method)]
    pub fn markClean(this:&UndoManager);

    #[wasm_bindgen(method)]
    pub fn isClean(this:&UndoManager)->bool;


    #[wasm_bindgen(method)]
    pub fn getModeForPath(this:&Module,path:&str)->Mode;

    #[wasm_bindgen(method,getter)]
    pub fn mode(this:&Mode)->String;


}

#[derive(Serialize)]
struct AceConfig{
    enableBasicAutocompletion: bool,
    enableSnippets: bool,
    enableLiveAutocompletion: bool
}

impl AceEditor{

    pub fn new(parent:HtmlElement)->AceEditor{
        let parent_jsval:&JsValue = parent.as_ref();
        let editor = open_editor(parent_jsval.clone());
        // editor.setTheme("ace/theme/monokai");
        editor.resize();
        editor.session().setUseWrapMode(false);
        editor.setOptions(JsValue::from_serde(&AceConfig{
            enableBasicAutocompletion:true,
            enableSnippets:true,
            enableLiveAutocompletion:true
        }).expect("Cant convert to JsValue"));
        AceEditor{
            editor
        }
    }

    pub fn set_value(&self,val:&str){
        self.editor.setValue(val);
    }

    pub fn get_value(&self)->String{
        self.editor.getValue()
    }

    pub fn set_mode(&self, mode:&str){
        self.editor.session().setMode(mode);
    }

    pub fn set_mode_from_filename(&self, filename:&str){
        let modelist=require("ace/ext/modelist");
        self.editor.session().setMode(
            &modelist.getModeForPath(filename).mode()
        );
    }

    pub fn set_soft_wrap(&self, mode:bool){
        self.editor.session().setUseWrapMode(mode);
    }

    pub fn markClean(&self){
        self.editor.session().getUndoManager().markClean();
    }

    pub fn isClean(&self)->bool{
        self.editor.session().getUndoManager().isClean()
    }

}
