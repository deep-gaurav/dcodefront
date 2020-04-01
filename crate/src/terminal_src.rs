use std::collections::HashMap;
use std::convert::TryInto;
use web_sys::HtmlTextAreaElement;
use wasm_bindgen::JsCast;
use vt100::Color::*;
use wasm_bindgen::prelude::*;
use vt100;
use js_sys::{Function};
use web_sys::{HtmlElement};
use gloo::{events::EventListener};
use lazy_static::lazy_static;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

lazy_static! {
    static ref KEYCODEMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
      //   keymaptochar={
      //     "ArrowUp":String.fromCharCode(27)+String.fromCharCode(91)+String.fromCharCode(65),
      //     "ArrowDown":String.fromCharCode(27)+String.fromCharCode(91)+String.fromCharCode(66),
      //     "ArrowRight":String.fromCharCode(27)+String.fromCharCode(91)+String.fromCharCode(67),
      //     "ArrowLeft":String.fromCharCode(27)+String.fromCharCode(91)+String.fromCharCode(68),
      //     "Backspace":String.fromCharCode(127),
      //     "Escape":String.fromCharCode(27),
      //     "Enter":String.fromCharCode(13)
      // };
        m.insert("ArrowUp".to_owned(),String::from_utf8(vec![27,91,65]).expect("not utf8"));
        m.insert("ArrowDown".to_owned(),String::from_utf8(vec![27,91,66]).expect("not utf8"));
        m.insert("ArrowRight".to_owned(),String::from_utf8(vec![27,91,67]).expect("not utf8"));
        m.insert("ArrowLeft".to_owned(),String::from_utf8(vec![27,91,68]).expect("not utf8"));
        m.insert("Backspace".to_owned(),String::from_utf8(vec![127]).expect("not utf8"));
        m.insert("Escape".to_owned(),String::from_utf8(vec![27]).expect("not utf8"));
        // m.insert("Enter".to_owned(),String::from_utf8(vec![13]).expect("not utf8"));

        m
    };
}

#[wasm_bindgen]
pub struct VtParser {
    parser:vt100::Parser,
    token: f64,
}

#[wasm_bindgen]
impl VtParser{
    pub fn new(row:u16,cols:u16,token:f64)->VtParser{
        VtParser{
            parser:vt100::Parser::new(row,cols,0),
            token
        }
    }

    pub fn parse(&mut self,vttext:&str){
        self.parser.process(vttext.as_bytes());
    }

    pub fn content(&self)->String{
        self.parser.screen().contents()
    }

    fn termcolor_to_htmlcolor(color:vt100::Color)->String{
        match color{
            Default=>"".to_owned(),
            Idx(index)=>{
                match index {
                    0=>"black".to_string(),
                    1=>"red".to_string(),
                    2=>"green".to_string(),
                    3=>"yellow".to_string(),
                    4=>"blue".to_string(),
                    5=>"violet".to_string(),
                    6=>"aqua".to_string(),
                    7=>"white".to_string(),
                    _ => "".to_string()
                }
            },
            Rgb(r,g,b)=>{
                format!("{},{},{}",r,g,b)
            }
        }
    }

    pub fn html_content(&self)->String{
        let mut outcontent = String::new();

        let cursor_pos = self.parser.screen().cursor_position();
        for row in 0..self.parser.screen().size().0{
            for col in 0..self.parser.screen().size().1{
                let cell = self.parser.screen().cell(row,col);
                if let Some(cell) = cell{
                    if cursor_pos!=(row,col){
                        outcontent=outcontent+&format!(r#"<font color="{}">{}</font>"#,VtParser::termcolor_to_htmlcolor(cell.fgcolor()),cell.contents());
                    }else{
                        let mut cellcon=cell.contents();
                        if cellcon == "".to_string(){
                            cellcon=" ".to_owned();
                        }
                        outcontent=outcontent+&format!(r#"<font color="{}" id="termcursor" style="background-color:gray">{}</font>"#,VtParser::termcolor_to_htmlcolor(cell.fgcolor()),cellcon);

                    }
                }
            }
            outcontent=outcontent+"<br>"
        }

        outcontent
    }
}

pub struct Terminal{
    onData:yew::Callback<String>,
    parser:VtParser,
    parent:HtmlElement,
    termdiv:web_sys::Element,
    inputListener:EventListener,
    keydownListener:EventListener,
    clickListender:EventListener,
    textarea:HtmlElement
}
impl Terminal {

    pub fn new(parent:HtmlElement,cb:yew::Callback<String>,cols:u16)->Result<Terminal,JsValue>{
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let el = document.create_element("textarea")?;
        let tdiv = document.create_element("div")?;
        tdiv.set_attribute("style","font-family:Courier New;white-space:pre;")?;
        el.set_attribute("autocomplete","off")?;
        el.set_attribute("autocapitalize","none")?;
        el.set_attribute("style",r#"
                position: fixed;
                opacity:0;
                height:0;
                width:0;
                z-index:-200;
        "#)?;
        let el2 = el.clone();
        let el3 = el.clone();
        let elc = el.clone();
        let cb2 = cb.clone();
        let cb3 = cb.clone();
        parent.append_child(&el);
        parent.append_child(&tdiv);
        let ev = EventListener::new(&el, "input", move |event|{
            let el3 = el2.clone().dyn_into::<HtmlTextAreaElement>().expect("Not inputElement");
            let val = el3.value();
            let cb3 = cb2.clone();
            let this = JsValue::NULL;
            // cb3.call1(&this,&JsValue::from_str(&val));
            cb3.emit(val);
            el3.set_value("");

        });
        let click_listenner = EventListener::new(&tdiv,"click",move |_ev|{
            elc.clone().dyn_into::<HtmlElement>().expect("NotHtmlElement").focus();
        });
        let key_down_ev = EventListener::new(&el, "keydown", move |event|{
            log::info!("Received keydown");
            let key_event = event.clone().dyn_into::<web_sys::KeyboardEvent>().expect("Not key event");
            let keycode = key_event.key_code();
            let inpel = el3.clone().dyn_into::<HtmlTextAreaElement>().expect("Not inputElement");
            let this = JsValue::NULL;

            if inpel.value().is_empty(){

                if key_event.ctrl_key(){

                    let ucode = keycode-64;
                    match ucode.try_into(){
                        Ok(code) => {

                            let stri = String::from_utf8(vec![code]);
                            if let Ok(val) = stri{

                                cb3.emit(val);
                            }
                        }
                        Err(err)=>{
                            log::error!("Cannot convert to string {}",err);
                        }
                    }
                }else{
                    let key = key_event.key();
                    if let Some(code)=KEYCODEMAP.get(&key){
                        cb3.emit(code.to_string())

                    }
                }
            }


        });
        let term = Terminal{
            onData:cb,
            parser:VtParser::new(24,cols,0.0),
            parent,
            inputListener:ev,
            keydownListener:key_down_ev,
            termdiv:tdiv,
            clickListender:click_listenner,
            textarea:el.clone().dyn_into().unwrap()
        };
        // let f = Closure::wrap(Box::new(move ||{term.parser.parse("");}));
        // el.set_oninput(Some(f.as_ref().unchecked_ref()));
        //
        // f.forget();
        Ok(term)
    }

    pub fn write(&mut self,inp:&str){
        self.parser.parse(inp);
        let htt = self.parser.html_content();
        self.termdiv.set_inner_html(&htt);

        let cursor = self.parent.clone().dyn_into::<web_sys::Element>().unwrap().query_selector("#termcursor");
        if let Ok(cursor)=cursor{
            if let Some(cursor)=cursor{
                let pos = cursor.get_bounding_client_rect();
                self.textarea.set_attribute("style",&format!(r#"
                    position:fixed;
                    left:{}px;
                    top:{}px;
                    opacity:0;
                    height:0;
                    width:0;
                    z-index:-200;
                "#,pos.right(),pos.bottom()));
            }
        }
    }

    pub fn get_title(&self)->String{
        self.parser.parser.screen().title().to_string()
    }

}
