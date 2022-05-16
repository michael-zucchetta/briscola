use crate::card;
use crate::hand;
use crate::deck;
use crate::game;
use crate::painter;
use crate::player;
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

struct SvgHandCard {
  card: Option<card::Card>,
}

const imagesPath: &str = "/assets/briscola/bresciane";
const retro_image_path: &str = "./assets/briscola/bresciane/retro.svg";
fn from_card_to_url(card: &card::Card) -> String {
    [imagesPath, "/",card.suit.to_string(), "/", card.value.to_string(), ".svg"].concat()
}

fn generate_svg(document: &Document, path: &str, id: Option<&str>) -> SvgElement {
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
  document: Document,
  player1CardsAreaDiv: Element,
  player2CardsAreaDiv: Element,
  playedCardsAreaDiv: Element,
  deckAreaBriscola: Element,
  deckAreaCards: Element
}

fn create_div(id_name: &str, document: &Document) -> Element {
  let div = document.create_element("div").unwrap();
  div.set_id(id_name);
  div
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

    fn generate_svg(&self, path: &str, id: Option<&str>) -> SvgElement {
	// image original size
	const width: usize = 170usize;
	const height: usize = 340usize;
	let svg = self.document.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg").unwrap()
	     .dyn_into::<web_sys::SvgElement>()
	     .map_err(|_| ())
	     .unwrap();
	svg.set_attribute("width", &width.to_string()).unwrap();
	svg.set_attribute("height", &height.to_string()).unwrap();
	svg.set_attribute("viewBox", &format!("0 0 {width} {height}")).unwrap();
	let svgImage = self.document.create_element_ns(Some("http://www.w3.org/2000/svg"), "image").unwrap()
	     .dyn_into::<web_sys::SvgImageElement>()
	     .map_err(|_| ())
	     .unwrap();
	svgImage.set_attribute_ns(Some("http://www.w3.org/1999/xlink"), "xlink:href", &path);
	svg.append_child(&svgImage).unwrap();
	return svg;
    }

    pub fn new() -> WebPainter {
        let document = web_sys::window().unwrap().document().unwrap();
        let player1CardsAreaDiv = create_div("player-1-area", &document);
        let player2CardsAreaDiv = create_div("player-2-area", &document);
        let playingAreaContainerDiv = create_div("playing-area", &document);
        let playedCardsAreaDiv = create_div("played-cards-area", &document);
        let deckAreaDiv = create_div("deck-area", &document);
        let deckAreaBriscola = create_div("deck-briscola-area", &document);
        let deckAreaCards = create_div("deck-cards-area", &document);
        deckAreaDiv.append_child(&deckAreaBriscola).unwrap();
        deckAreaDiv.append_child(&deckAreaCards).unwrap();
        playingAreaContainerDiv.append_child(&playedCardsAreaDiv).unwrap();
        playingAreaContainerDiv.append_child(&deckAreaDiv).unwrap();

        let body = document.body().expect("document should have a body");
        body.append_child(&player1CardsAreaDiv).unwrap();
        body.append_child(&playingAreaContainerDiv).unwrap();
        body.append_child(&player2CardsAreaDiv).unwrap();
        WebPainter {
            document: document,
            player1CardsAreaDiv: player1CardsAreaDiv,
            player2CardsAreaDiv: player2CardsAreaDiv,
            playedCardsAreaDiv: playedCardsAreaDiv,
            deckAreaCards: deckAreaCards,
            deckAreaBriscola: deckAreaBriscola
        }
    }
}

impl<Y> painter::Painter<Y> for WebPainter where Y: game::UserInput {
    fn print_card(card: card::Card) {
        let (value, suit, color) = WebPainter::get_card(card);
        log!("{} {}", value, color.bold().paint(suit));
    }

    fn draw_beginning(&self, deck: &deck::Deck) {
      log!("Beginning game");
      let briscola = deck.get_briscola();
      log!("Briscola is {}", briscola);
      let briscola_svg = self.generate_svg(&from_card_to_url(&briscola), Some("id"));
      self.deckAreaBriscola.append_child(&briscola_svg).unwrap();
      log!("Deck Size is {}, {}", deck.size(), deck.size());
      for i in 0..deck.size() {
          // this has to stay here
          let retro_svg = self.generate_svg(retro_image_path, None);
          log!("GOgo");
          let div = create_div("", &self.document);
          self.deckAreaCards.append_child(&div).unwrap();
          div.append_child(&retro_svg);
      }
      log!("GOgo22");
    }

    fn update_game() {
    }
    fn print_cards(&self, player: &player::Player<Y>) { // hand: &hand::Hand, player: usize) {
       let hand = player.get_hand();
       let div_container = if player.playing_order == 0 {
           &self.player1CardsAreaDiv
       } else {
           &self.player2CardsAreaDiv
       };
       for (card_idx, card) in hand.get_hand().iter().enumerate() {
          let card_svg = self.generate_svg(&from_card_to_url(card), Some("id1"));
          let f = Closure::wrap(Box::new(move || {
              log!("hello {}", card_idx);
          }) as Box<dyn FnMut()>);
          card_svg.set_onclick(Some(f.as_ref().unchecked_ref()));
          f.forget();
          div_container.append_child(&card_svg).unwrap();
       }
       log!("User 2 hand is {:?}", hand.get_hand_ref());
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
      Web { }
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
    // log!("Ciccio");
    // let app = App::<SvgHandCard>::new();
    // app.mount_to_body();
    let canvas = create_canvas(&document);
    let div = document.create_element("div").unwrap();
    div.append_child(&canvas).unwrap();
    body.append_child(&div).unwrap();
    // let image = ImageFuture::new("/assets/briscola/bresciane/retro.svg");
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
    let svg = generate_svg(&document, &retro_image_path, None);//&from_card_to_url(card));
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
    //
    let webPainter = WebPainter::new();
    let web = Web::new();
    let game = game::Game::new(
       game::PlayersSize::Two,
       game::GameMode::PlayerVsAI,
       webPainter,
       web,
    );
    let player_won = game.game();
    println!("Player {} won", player_won);
}
