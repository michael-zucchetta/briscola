use crate::card;
use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlImageElement, SvgImageElement};
use std::fmt;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};


fn from_card_to_url(card: card::Card) {

}

// change display with draw and use fmt in console
// https://doc.rust-lang.org/reference/conditional-compilation.html
/*impl fmt::Display for card::Card {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f, "{}", 1) 
   }
}*/

fn create_canvas(document: &Document) -> HtmlCanvasElement {
    let canvas = HtmlCanvasElement::from(JsValue::from(document.create_element("canvas").unwrap()));
    canvas.set_width(500);
    canvas.set_height(800);
    let ctx =
        CanvasRenderingContext2d::from(JsValue::from(canvas.get_context("2d").unwrap().unwrap()));
    ctx.set_fill_style(&JsValue::from_str("red"));
    ctx.fill_rect(10., 10., 200., 200.);

    canvas
}

pub struct ImageFuture {
    image: Option<HtmlImageElement>,
    load_failed: Rc<Cell<bool>>,
}

impl ImageFuture {
    pub fn new(path: &str) -> Self {
        let image = HtmlImageElement::new().unwrap();
        image.set_src(path);
        ImageFuture {
            image: Some(image),
            load_failed: Rc::new(Cell::new(false)),
        }
    }
}


impl Future for ImageFuture {
    type Output = Result<HtmlImageElement, ()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &self.image {
            Some(image) if image.complete() => {
                let failed = self.load_failed.get();
                if failed {
                    Poll::Ready(Err(()))
                } else {
                    let image = self.image.take().unwrap();
                    Poll::Ready(Ok(image))
                }
            }
            Some(image) => {
                let waker = cx.waker().clone();
                let on_load_closure = Closure::wrap(Box::new(move || {
                    waker.wake_by_ref();
                }) as Box<dyn FnMut()>);
                image.set_onload(Some(on_load_closure.as_ref().unchecked_ref()));
                on_load_closure.forget();

                let waker = cx.waker().clone();
                let failed_flag = self.load_failed.clone();
                let on_error_closure = Closure::wrap(Box::new(move || {
                    failed_flag.set(true);
                    waker.wake_by_ref();
                }) as Box<dyn FnMut()>);
                image.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));
                on_error_closure.forget();

                Poll::Pending
            }
            _ => Poll::Ready(Err(())),
        }
    }
}

pub fn main() {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().expect("document should have a body");

    let canvas = create_canvas(&document);
    let div = document.create_element("div").unwrap();
    div.append_child(&canvas).unwrap();
    body.append_child(&div).unwrap();
    let image = ImageFuture::new("/assets/briscola/bresciane/bastoni/02.svg");
    let svg = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    // should use this context.draw_image_with_svg_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    /*
Try calling the drawImage function inside the image's load function to ensure that the image is actually loaded before trying to draw it.
internal_image.addEventListener("load", function() {
  context.drawImage(internal_image, 10, 10);
}, false);*/
    context.draw_image_with_svg_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    // context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    //	&image.image.unwrap(),
    &svg,
    0f64,
    0f64,
    100f64,
    200f64,
    0f64,
    0f64,
    100f64,
    200f64 
    ).unwrap();
    let div2 = document.create_element("div").unwrap();
    // div2.append_child(&image.image.unwrap()).unwrap();
    body.append_child(&div2).unwrap();
    // yew::start_app::<App>();
}

