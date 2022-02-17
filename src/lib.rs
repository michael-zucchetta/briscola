use wasm_bindgen::prelude::*;
use yew::prelude::*;

pub mod card;
pub mod hand;
// pub use card::{Card};

pub mod constants;
pub mod deck;
pub mod game;
pub mod painter;
pub mod player;
pub mod console;
pub mod web;

struct Model {
    link: ComponentLink<Self>,
    value: i64,
}

enum Msg {
    AddOne,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => self.value += 1
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}

fn create_canvas(document: &Document) -> HtmlCanvasElement {
    let canvas = HtmlCanvasElement::from(JsValue::from(document.create_element("canvas").unwrap()));
    canvas.set_width(100);
    canvas.set_height(100);
    let ctx =
        CanvasRenderingContext2d::from(JsValue::from(canvas.get_context("2d").unwrap().unwrap()));
    ctx.set_fill_style(&JsValue::from_str("green"));
    ctx.fill_rect(10., 10., 50., 50.);

    canvas
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
    let canvas = create_canvas(&document);
    body.append_child(&canvas).unwrap();
}
