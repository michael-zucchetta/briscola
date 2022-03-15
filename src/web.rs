use crate::card;
use crate::hand;
use crate::deck;
use crate::game;
use crate::painter;
use ansi_term::Colour::{Yellow, Red, Green, Blue};
use ansi_term::Colour;

use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlImageElement, Element, SvgElement, SvgImageElement};
use std::fmt;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

const imagesPath: &str = "/assets/briscola/bresciane";
fn from_card_to_url(card: card::Card) -> String {
    [imagesPath, "/",card.suit.to_string(), "/", card.value.to_string(), ".svg"].concat()
}

fn generate_svg(document: &Document, path: &String) -> SvgElement {
    // image original size
    const width: usize = 170usize;
    const height: usize = 340usize;
    let svg = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap()
         .dyn_into::<web_sys::SvgElement>()
         .map_err(|_| ())
         .unwrap();
    svg.set_attribute("width", &width.to_string()).unwrap();
    svg.set_attribute("height", &height.to_string()).unwrap();
    svg.set_attribute("viewBox", &format!("0 0 {width} {height}")).unwrap();
    let svgImage = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "image").unwrap()
         .dyn_into::<web_sys::SvgImageElement>()
         .map_err(|_| ())
         .unwrap();
    svgImage.set_attribute_ns(Some("http://www.w3.org/1999/xlink"), "xlink:href", &path);
    svg.append_child(&svgImage).unwrap();
    return svg;
}

pub struct WebPainter {
  player1CardsAreaDiv: Element
  player2CardsAreaDiv: Element
  playedCardsArea: Element
  deckArea: Element 
}

impl WebPainter {
    fn get_card(card: card::Card) -> (u8, &'static str, Colour) {
        let (color, suit_as_string) = match card.suit {
            card::CardSuit::Cups => (Blue, "Cups"),
            card::CardSuit::Batons => (Green, "Batons"),
            card::CardSuit::Coins => (Yellow, "Coins"),
            card::CardSuit::Swords => (Red, "Swords"),
        };
        let value = card.value.eval();
        (value, suit_as_string, color)
    }
    pub fn new() -> WebPainter {
       WebPainter {} 
    }
}

impl painter::Painter for WebPainter {
    fn print_card(card: card::Card) {
        let (value, suit, color) = WebPainter::get_card(card);
        log!("{} {}", value, color.bold().paint(suit));
    }

    fn draw_beginning(deck: &deck::Deck) {
      log!("Beginning game");
      log!("Briscola is {}", deck.get_briscola());
    }

    fn update_game() {
    }
    fn print_cards(hand: hand::Hand) {
       log!("User hand is {:?}", hand.get_hand_ref());
    }

    fn player_played_card(card: card::Card, player: usize) {
       log!("Card played by player {} is {}", player, card);
    }

    fn player_won(player: usize, cards: &Vec<card::Card>) {
      log!("Player {} won turn, and won these cards {:?}", player, cards);
    }

    fn player_scores(score1: u8, score2: u8) {
        log!("Player 1 score is {}", score1);
        log!("Player 2 score is {}", score2);
    }
}

#[derive(Clone)]
pub struct Web {

}

impl Web {

  pub fn new() -> Web {
    Web {}
  }
}

impl game::UserInput for Web {
    fn user_input(&self, hand: &hand::Hand) -> usize {
      0usize
    }
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
    canvas.set_width(600);
    canvas.set_height(1200);
    canvas.style().set_property("width", "300px");
    canvas.style().set_property("height", "600px");
    let ctx =
        CanvasRenderingContext2d::from(JsValue::from(canvas.get_context("2d").unwrap().unwrap()));
    ctx.set_fill_style(&JsValue::from_str("red"));
    ctx.fill_rect(10., 10., 200., 200.);
    ctx.scale(2f64, 2f64);

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
    log!("Ciccio");
    let canvas = create_canvas(&document);
    let div = document.create_element("div").unwrap();
    div.append_child(&canvas).unwrap();
    body.append_child(&div).unwrap();
    let image = ImageFuture::new("/assets/briscola/bresciane/batons/02.svg");
    /*let svg = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap()
         .dyn_into::<web_sys::SvgElement>()
         .map_err(|_| ())
         .unwrap();
    svg.set_attribute("width","500").unwrap();
    svg.set_attribute("height","500").unwrap();
    svg.set_attribute("viewBox", "0 0 500 500").unwrap();
    let svgImage = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "image").unwrap()
         .dyn_into::<web_sys::SvgImageElement>()
         .map_err(|_| ())
         .unwrap();*/
    let card = card::Card::new( card::CardNumber::Two, card::CardSuit::Cups);
    let svg = generate_svg(&document, &from_card_to_url(card));
    // svgImage.set_attribute_ns(Some("http://www.w3.org/1999/xlink"), "xlink:href", &from_card_to_url(card));
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    context.set_image_smoothing_enabled(true);
    // svg.append_child(&svgImage).unwrap();
    // should use this context.draw_image_with_svg_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    /*
Try calling the drawImage function inside the image's load function to ensure that the image is actually loaded before trying to draw it.
internal_image.addEventListener("load", function() {
  context.drawImage(internal_image, 10, 10);
}, false);*/
    /*context.draw_image_with_svg_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    // context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
    // &image.image.unwrap(),
    &svgImage,
    0f64,
    0f64,
    2f64 * 170f64,
    2f64 * 340f64,
    0f64,
    0f64,
    2f64 * 170f64,
    2f64 * 340f64 
    ).unwrap();*/
    let div2 = document.create_element("div").unwrap();
    div2.append_child(&svg).unwrap();
    body.append_child(&div2).unwrap();
    // yew::start_app::<App>();
}

