use std::cell::RefCell;
use std::rc::Rc;

use rand::Rng;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use web_sys::{window, Document, Element};

pub mod card;
pub mod console;
pub mod constants;
pub mod deck;
pub mod game;
pub mod hand;
pub mod painter;
pub mod player;

const FULL_DECK_SIZE: usize = 40;

#[derive(Clone, Copy)]
enum Phase {
    Dealing,
    Lead,
    Follow,
    Resolve,
    Collect,
    DrawPlayer1,
    DrawPlayer2,
    Finished,
}

struct BrowserGame {
    deck: deck::Deck,
    briscola: card::Card,
    player1_hand: Vec<card::Card>,
    player2_hand: Vec<card::Card>,
    player1_won: Vec<card::Card>,
    player2_won: Vec<card::Card>,
    trick_cards: Vec<(usize, card::Card)>,
    player_turn: game::PlayerTurn,
    pending_trick_winner: Option<usize>,
    phase: Phase,
    status: String,
    trick_number: usize,
    deck_size: usize,
    draw_first_player: usize,
    draw_second_player: usize,
}

impl BrowserGame {
    fn new() -> Self {
        let deck = deck::Deck::new();
        let briscola = deck.get_briscola();
        let player_turn = match rand::thread_rng().gen_range(0..2) {
            0 => game::PlayerTurn::Player1,
            _ => game::PlayerTurn::Player2,
        };

        Self {
            deck,
            briscola,
            player1_hand: Vec::with_capacity(constants::HAND_SIZE),
            player2_hand: Vec::with_capacity(constants::HAND_SIZE),
            player1_won: Vec::new(),
            player2_won: Vec::new(),
            trick_cards: Vec::with_capacity(2),
            player_turn,
            pending_trick_winner: None,
            phase: Phase::Dealing,
            status: "Shuffling deck for the animated table.".to_string(),
            trick_number: 1,
            deck_size: FULL_DECK_SIZE,
            draw_first_player: 1,
            draw_second_player: 2,
        }
    }

    fn leader_index(&self) -> usize {
        match self.player_turn {
            game::PlayerTurn::Player1 => 1,
            game::PlayerTurn::Player2 => 2,
        }
    }

    fn other_player(player_index: usize) -> usize {
        if player_index == 1 {
            2
        } else {
            1
        }
    }

    fn player_label(player_index: usize) -> &'static str {
        if player_index == 1 {
            "you"
        } else {
            "Player 2"
        }
    }

    fn player_sentence_label(player_index: usize) -> &'static str {
        if player_index == 1 {
            "You"
        } else {
            "Player 2"
        }
    }

    fn hand_mut(&mut self, player_index: usize) -> &mut Vec<card::Card> {
        if player_index == 1 {
            &mut self.player1_hand
        } else {
            &mut self.player2_hand
        }
    }

    fn won_cards_mut(&mut self, player_index: usize) -> &mut Vec<card::Card> {
        if player_index == 1 {
            &mut self.player1_won
        } else {
            &mut self.player2_won
        }
    }

    fn deal_card_to(&mut self, player_index: usize) {
        if let Some(card) = self.deck.get_card() {
            self.hand_mut(player_index).push(card);
            self.deck_size = self.deck_size.saturating_sub(1);
        }
    }

    fn play_random_card(&mut self, player_index: usize) {
        let hand = self.hand_mut(player_index);
        let selected = rand::thread_rng().gen_range(0..hand.len());
        let card = hand.remove(selected);
        self.trick_cards.push((player_index, card));
    }

    fn waiting_for_player1(&self) -> bool {
        match self.phase {
            Phase::Lead => self.leader_index() == 1,
            Phase::Follow => Self::other_player(self.leader_index()) == 1,
            _ => false,
        }
    }

    fn play_player1_card(&mut self, selected: usize) -> Option<i32> {
        if !self.waiting_for_player1() || selected >= self.player1_hand.len() {
            return None;
        }

        let card = self.player1_hand.remove(selected);
        self.trick_cards.push((1, card));

        match self.phase {
            Phase::Lead => {
                self.phase = Phase::Follow;
                self.status = format!("Trick {}. You lead.", self.trick_number);
                Some(1050)
            }
            Phase::Follow => {
                self.phase = Phase::Resolve;
                self.status = format!("Trick {}. You answer.", self.trick_number);
                Some(1150)
            }
            _ => None,
        }
    }

    fn wins_first(&self, card1: card::Card, card2: card::Card) -> bool {
        if card1.suit == self.briscola.suit {
            if card2.suit != self.briscola.suit {
                true
            } else {
                card1.value.eval() > card2.value.eval()
            }
        } else if card2.suit == self.briscola.suit {
            false
        } else {
            card1.value.eval() > card2.value.eval()
        }
    }

    fn preview_trick_winner(&self) -> usize {
        let (lead_player, lead_card) = self.trick_cards[0];
        let (follow_player, follow_card) = self.trick_cards[1];

        if self.wins_first(lead_card, follow_card) {
            lead_player
        } else {
            follow_player
        }
    }

    fn collect_trick(&mut self) -> usize {
        let winner = self
            .pending_trick_winner
            .take()
            .unwrap_or_else(|| self.preview_trick_winner());
        let won_cards = self
            .trick_cards
            .drain(..)
            .map(|(_, card)| card)
            .collect::<Vec<_>>();
        self.won_cards_mut(winner).extend(won_cards);
        self.player_turn = if winner == 1 {
            game::PlayerTurn::Player1
        } else {
            game::PlayerTurn::Player2
        };
        self.trick_number += 1;
        winner
    }

    fn calculate_score(cards: &[card::Card]) -> u8 {
        cards
            .iter()
            .map(|card| match card.value.eval() {
                1 => 11,
                3 => 10,
                8 => 2,
                9 => 3,
                10 => 4,
                _ => 0,
            })
            .sum()
    }

    fn scores(&self) -> (u8, u8) {
        (
            Self::calculate_score(&self.player1_won),
            Self::calculate_score(&self.player2_won),
        )
    }

    fn game_winner(&self) -> usize {
        let (score1, score2) = self.scores();
        if score1 > score2 {
            1
        } else {
            2
        }
    }

    fn advance(&mut self) -> Option<i32> {
        match self.phase {
            Phase::Dealing => {
                let dealt_cards = self.player1_hand.len() + self.player2_hand.len();
                let next_player = if dealt_cards % 2 == 0 { 1 } else { 2 };
                self.deal_card_to(next_player);

                let dealt_total = dealt_cards + 1;
                if dealt_total == constants::HAND_SIZE * 2 {
                    self.phase = Phase::Lead;
                    self.status = format!(
                        "Deal complete. Player {} leads the first trick.",
                        self.leader_index()
                    );
                    Some(1200)
                } else {
                    let hand_size = if next_player == 1 {
                        self.player1_hand.len()
                    } else {
                        self.player2_hand.len()
                    };
                    self.status = format!(
                        "Dealt a card to {}. Hand now has {} cards. ({}/{})",
                        Self::player_label(next_player),
                        hand_size,
                        dealt_total,
                        constants::HAND_SIZE * 2,
                    );
                    Some(650)
                }
            }
            Phase::Lead => {
                let player = self.leader_index();
                if player == 1 {
                    self.status = format!("Trick {}. Select a card to lead.", self.trick_number);
                    return None;
                }
                self.play_random_card(player);
                self.phase = Phase::Follow;
                self.status = format!("Trick {}. Player {} leads.", self.trick_number, player);
                Some(1050)
            }
            Phase::Follow => {
                let player = Self::other_player(self.leader_index());
                if player == 1 {
                    self.status = format!("Trick {}. Select a card to answer.", self.trick_number);
                    return None;
                }
                self.play_random_card(player);
                self.phase = Phase::Resolve;
                self.status = format!("Trick {}. Player {} answers.", self.trick_number, player);
                Some(1150)
            }
            Phase::Resolve => {
                let winner = self.preview_trick_winner();
                self.pending_trick_winner = Some(winner);
                self.phase = Phase::Collect;
                self.status = format!("Player {} wins trick {}.", winner, self.trick_number);
                Some(1200)
            }
            Phase::Collect => {
                let winner = self.collect_trick();

                if self.player1_hand.is_empty()
                    && self.player2_hand.is_empty()
                    && self.deck_size == 0
                {
                    self.phase = Phase::Finished;
                    let (score1, score2) = self.scores();
                    self.status = format!(
                        "Player {} wins the game, {} to {}.",
                        self.game_winner(),
                        score1,
                        score2
                    );
                    None
                } else if self.deck_size > 0 {
                    self.draw_first_player = winner;
                    self.draw_second_player = Self::other_player(winner);
                    self.phase = Phase::DrawPlayer1;
                    self.status =
                        format!("Player {} collects the trick. Winner draws first.", winner);
                    Some(950)
                } else {
                    self.phase = Phase::Lead;
                    self.status = format!("Deck empty. Player {} leads the final stretch.", winner);
                    Some(1100)
                }
            }
            Phase::DrawPlayer1 => {
                let player = self.draw_first_player;
                self.deal_card_to(player);
                self.phase = Phase::DrawPlayer2;
                self.status = format!(
                    "{} drew the first replacement card.",
                    Self::player_sentence_label(player)
                );
                Some(950)
            }
            Phase::DrawPlayer2 => {
                if self.deck_size > 0 {
                    let player = self.draw_second_player;
                    self.deal_card_to(player);
                    self.status = format!(
                        "{} drew the second replacement card.",
                        Self::player_sentence_label(player)
                    );
                } else {
                    self.status = "No second replacement card remains in the deck.".to_string();
                }
                self.phase = Phase::Lead;
                Some(1100)
            }
            Phase::Finished => None,
        }
    }
}

fn document() -> Document {
    window()
        .expect("window should exist")
        .document()
        .expect("document should exist")
}

fn set_styles(document: &Document) {
    if document.get_element_by_id("briscola-styles").is_some() {
        return;
    }

    let style = document
        .create_element("style")
        .expect("style element should be created");
    style
        .set_attribute("id", "briscola-styles")
        .expect("style id should be set");
    style.set_inner_html(
        r#"
        :root {
            --felt-dark: #0d3a27;
            --felt-mid: #176545;
            --felt-light: #46a471;
            --ink: #102117;
            --panel: rgba(244, 235, 215, 0.92);
            --line: rgba(255, 255, 255, 0.18);
            --shadow: rgba(7, 24, 15, 0.28);
        }

        * {
            box-sizing: border-box;
        }

        body {
            margin: 0;
            min-height: 100vh;
            font-family: Georgia, "Times New Roman", serif;
            color: #f7f0df;
            background:
                radial-gradient(circle at top, rgba(255,255,255,0.14), transparent 28%),
                linear-gradient(180deg, var(--felt-light) 0%, var(--felt-mid) 34%, var(--felt-dark) 100%);
        }

        .dashboard {
            min-height: 100vh;
            padding: 24px;
            display: grid;
            grid-template-rows: auto 1fr auto;
            gap: 24px;
        }

        .table-header {
            display: grid;
            grid-template-columns: minmax(0, 1fr) auto;
            gap: 16px;
            padding: 18px 22px;
            border: 1px solid var(--line);
            border-radius: 20px;
            background: rgba(10, 41, 24, 0.35);
            backdrop-filter: blur(8px);
            box-shadow: 0 18px 40px var(--shadow);
        }

        .table-title {
            margin: 0;
            font-size: clamp(28px, 4vw, 42px);
            letter-spacing: 0.04em;
        }

        .table-subtitle {
            margin: 6px 0 0;
            font-size: 14px;
            letter-spacing: 0.08em;
            text-transform: uppercase;
            opacity: 0.78;
        }

        .header-rail {
            display: flex;
            flex-wrap: wrap;
            justify-content: flex-end;
            gap: 10px;
            align-items: flex-start;
        }

        .header-pill {
            padding: 10px 14px;
            border-radius: 999px;
            background: rgba(244, 235, 215, 0.14);
            border: 1px solid rgba(244, 235, 215, 0.26);
            font-size: 13px;
            text-transform: uppercase;
            letter-spacing: 0.08em;
        }

        .table-board {
            position: relative;
            display: grid;
            grid-template-rows: auto 1fr auto;
            gap: 28px;
            padding: 28px;
            border-radius: 28px;
            border: 1px solid var(--line);
            background:
                radial-gradient(circle at center, rgba(255,255,255,0.08), transparent 55%),
                rgba(8, 40, 24, 0.32);
            box-shadow: inset 0 1px 0 rgba(255,255,255,0.08), 0 28px 60px var(--shadow);
            overflow: hidden;
        }

        .table-board::before {
            content: "";
            position: absolute;
            inset: 12px;
            border: 1px dashed rgba(255,255,255,0.18);
            border-radius: 22px;
            pointer-events: none;
        }

        .player-zone {
            position: relative;
            z-index: 1;
            display: grid;
            gap: 14px;
            justify-items: center;
        }

        .player-label {
            margin: 0;
            font-size: 14px;
            text-transform: uppercase;
            letter-spacing: 0.14em;
            opacity: 0.82;
        }

        .player-hand {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 14px;
            flex-wrap: wrap;
            min-height: 168px;
        }

        .table-middle {
            position: relative;
            z-index: 1;
            display: grid;
            grid-template-columns: repeat(3, minmax(160px, 220px));
            justify-content: center;
            gap: 24px;
            align-items: stretch;
        }

        .stack-panel {
            padding: 18px;
            border-radius: 22px;
            background: rgba(244, 235, 215, 0.08);
            border: 1px solid rgba(255,255,255,0.14);
            box-shadow: 0 18px 35px rgba(0,0,0,0.14);
        }

        .stack-label {
            margin: 0 0 12px;
            text-transform: uppercase;
            letter-spacing: 0.12em;
            font-size: 12px;
            opacity: 0.8;
        }

        .deck-stack,
        .trick-stack {
            position: relative;
            width: 112px;
            height: 168px;
            margin: 0 auto;
        }

        .deck-stack .card-shell {
            position: absolute;
            inset: 0;
            min-width: 0;
            width: 100%;
        }

        .deck-stack .card-back:nth-child(1) {
            transform: translate(12px, 10px) rotate(8deg);
        }

        .deck-stack .card-back:nth-child(2) {
            transform: translate(6px, 5px) rotate(4deg);
        }

        .deck-stack .card-back:nth-child(3) {
            transform: translate(0, 0);
        }

        .trick-slots {
            display: grid;
            gap: 12px;
            justify-items: center;
        }

        .trick-slot {
            display: grid;
            gap: 8px;
            justify-items: center;
        }

        .trick-slot-label {
            margin: 0;
            font-size: 11px;
            letter-spacing: 0.12em;
            text-transform: uppercase;
            opacity: 0.74;
        }

        .stack-count {
            margin: 14px 0 0;
            text-align: center;
            font-size: 28px;
            font-weight: 700;
        }

        .stack-caption {
            margin: 4px 0 0;
            text-align: center;
            font-size: 13px;
            opacity: 0.76;
        }

        .card-shell,
        .card-button,
        .card-placeholder {
            width: min(18vw, 112px);
            min-width: 86px;
            aspect-ratio: 2 / 3;
            border-radius: 14px;
            overflow: hidden;
            box-shadow: 0 12px 28px rgba(0,0,0,0.24);
            background: #f9f6eb;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .card-button {
            padding: 0;
            border: 0;
            cursor: pointer;
            transition: transform 160ms ease, box-shadow 160ms ease;
        }

        .card-button:hover,
        .card-button:focus-visible {
            outline: 3px solid rgba(255, 240, 178, 0.88);
            outline-offset: 4px;
            transform: translateY(-8px);
            box-shadow: 0 20px 34px rgba(0,0,0,0.3);
        }

        .card-button:active {
            transform: translateY(-4px);
        }

        .card-shell svg,
        .card-button svg,
        .card-shell img {
            width: 100%;
            height: 100%;
            display: block;
        }

        .card-back {
            border-radius: 14px;
            border: 2px solid rgba(244, 235, 215, 0.32);
            background:
                linear-gradient(135deg, rgba(255,255,255,0.12), transparent),
                repeating-linear-gradient(
                    45deg,
                    rgba(245, 229, 184, 0.22) 0,
                    rgba(245, 229, 184, 0.22) 8px,
                    rgba(109, 49, 28, 0.4) 8px,
                    rgba(109, 49, 28, 0.4) 16px
                ),
                linear-gradient(180deg, #8b4520 0%, #6f2b16 100%);
        }

        .card-back::after,
        .card-placeholder::after {
            content: "";
            width: calc(100% - 18px);
            height: calc(100% - 18px);
            border-radius: 10px;
            border: 1px solid rgba(244, 235, 215, 0.45);
            background: rgba(255,255,255,0.06);
        }

        .card-placeholder {
            border: 2px dashed rgba(244, 235, 215, 0.32);
            background: rgba(255, 255, 255, 0.04);
            box-shadow: inset 0 0 0 1px rgba(255,255,255,0.05);
        }

        .footer-bar {
            display: flex;
            justify-content: center;
            gap: 12px;
            flex-wrap: wrap;
        }

        .footer-pill {
            padding: 10px 16px;
            border-radius: 999px;
            background: rgba(10, 41, 24, 0.35);
            border: 1px solid rgba(255,255,255,0.18);
            font-size: 13px;
            letter-spacing: 0.08em;
            text-transform: uppercase;
        }

        .footer-pill.action {
            background: rgba(255, 240, 178, 0.2);
            border-color: rgba(255, 240, 178, 0.48);
            color: #fff4bb;
        }

        @media (max-width: 900px) {
            .table-middle {
                grid-template-columns: 1fr;
            }
        }

        @media (max-width: 720px) {
            .dashboard {
                padding: 16px;
                gap: 16px;
            }

            .table-board {
                padding: 18px;
                gap: 20px;
            }

            .table-header {
                grid-template-columns: 1fr;
            }

            .header-rail {
                justify-content: flex-start;
            }
        }
    "#,
    );

    document
        .head()
        .expect("document head should exist")
        .append_child(&style)
        .expect("style should be appended");
}

fn create_element(document: &Document, tag: &str, class_name: &str) -> Element {
    let element = document
        .create_element(tag)
        .expect("element should be created");
    if !class_name.is_empty() {
        element.set_class_name(class_name);
    }
    element
}

fn append_text(document: &Document, parent: &Element, tag: &str, class_name: &str, text: &str) {
    let element = create_element(document, tag, class_name);
    element.set_text_content(Some(text));
    parent
        .append_child(&element)
        .expect("text element should be appended");
}

fn render_card_svg(document: &Document, card: card::Card) -> Element {
    let card_shell = create_element(document, "div", "card-shell");
    card_shell.set_inner_html(card_svg(card));
    card_shell
}

fn render_card_button(
    document: &Document,
    state: Rc<RefCell<BrowserGame>>,
    card: card::Card,
    selected: usize,
) -> Element {
    let button = create_element(document, "button", "card-button");
    button
        .set_attribute("type", "button")
        .expect("button type should be set");
    button
        .set_attribute("aria-label", &format!("Play card {}", selected + 1))
        .expect("button label should be set");
    button.set_inner_html(card_svg(card));

    let callback_state = Rc::clone(&state);
    let callback = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        select_player1_card(Rc::clone(&callback_state), selected);
    }) as Box<dyn FnMut()>);

    button
        .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
        .expect("card click handler should be attached");
    callback.forget();
    button
}

fn render_card_back(document: &Document) -> Element {
    create_element(document, "div", "card-shell card-back")
}

fn render_placeholder_card(document: &Document) -> Element {
    create_element(document, "div", "card-placeholder")
}

fn build_player_zone(
    document: &Document,
    label: &str,
    cards: &[card::Card],
    face_up: bool,
    selectable: bool,
    state: Rc<RefCell<BrowserGame>>,
) -> Element {
    let zone = create_element(document, "section", "player-zone");
    append_text(document, &zone, "p", "player-label", label);

    let hand = create_element(document, "div", "player-hand");
    for (index, &card) in cards.iter().enumerate() {
        let card_element = if selectable {
            render_card_button(document, Rc::clone(&state), card, index)
        } else if face_up {
            render_card_svg(document, card)
        } else {
            render_card_back(document)
        };
        hand.append_child(&card_element)
            .expect("player card should be appended");
    }

    zone.append_child(&hand)
        .expect("player hand should be appended");
    zone
}

fn build_deck_panel(document: &Document, deck_size: usize) -> Element {
    let panel = create_element(document, "section", "stack-panel");
    append_text(document, &panel, "p", "stack-label", "Deck");

    let stack = create_element(document, "div", "deck-stack");
    if deck_size == 0 {
        stack
            .append_child(&render_placeholder_card(document))
            .expect("deck placeholder should be appended");
    } else {
        for _ in 0..deck_size.min(3) {
            stack
                .append_child(&render_card_back(document))
                .expect("deck card should be appended");
        }
    }
    panel
        .append_child(&stack)
        .expect("deck stack should be appended");

    append_text(document, &panel, "p", "stack-count", &deck_size.to_string());
    append_text(
        document,
        &panel,
        "p",
        "stack-caption",
        "cards remaining in the draw pile",
    );
    panel
}

fn build_trick_panel(document: &Document, trick_cards: &[(usize, card::Card)]) -> Element {
    let panel = create_element(document, "section", "stack-panel");
    append_text(document, &panel, "p", "stack-label", "Current Trick");

    let slots = create_element(document, "div", "trick-slots");
    for player_index in [2usize, 1usize] {
        let slot = create_element(document, "div", "trick-slot");
        append_text(
            document,
            &slot,
            "p",
            "trick-slot-label",
            &format!("Player {}", player_index),
        );

        let card_node = trick_cards
            .iter()
            .find(|(owner, _)| *owner == player_index)
            .map(|(_, card)| render_card_svg(document, *card))
            .unwrap_or_else(|| render_placeholder_card(document));

        slot.append_child(&card_node)
            .expect("trick card should be appended");
        slots
            .append_child(&slot)
            .expect("trick slot should be appended");
    }

    panel
        .append_child(&slots)
        .expect("trick slots should be appended");
    append_text(
        document,
        &panel,
        "p",
        "stack-caption",
        "Cards stay here until the trick is collected",
    );
    panel
}

fn build_trump_panel(document: &Document, briscola: card::Card, deck_size: usize) -> Element {
    let panel = create_element(document, "section", "stack-panel");
    append_text(document, &panel, "p", "stack-label", "Briscola");
    if deck_size == 0 {
        panel
            .append_child(&render_placeholder_card(document))
            .expect("briscola placeholder should be appended");
        append_text(
            document,
            &panel,
            "p",
            "stack-caption",
            "Trump suit; the visible card has been drawn",
        );
    } else {
        panel
            .append_child(&render_card_svg(document, briscola))
            .expect("briscola card should be appended");
        append_text(
            document,
            &panel,
            "p",
            "stack-caption",
            "Visible trump card, drawn last",
        );
    }
    panel
}

fn render_dashboard(document: &Document, state: Rc<RefCell<BrowserGame>>) {
    set_styles(document);
    let body = document.body().expect("document body should exist");
    body.set_inner_html("");

    let state_ref = state.borrow();
    let (score1, score2) = state_ref.scores();

    let dashboard = create_element(document, "main", "dashboard");

    let header = create_element(document, "header", "table-header");
    let title_wrap = create_element(document, "div", "");
    append_text(document, &title_wrap, "h1", "table-title", "Briscola");
    append_text(
        document,
        &title_wrap,
        "p",
        "table-subtitle",
        "Player 1 is controlled from this browser",
    );
    header
        .append_child(&title_wrap)
        .expect("header title should be appended");

    let header_rail = create_element(document, "div", "header-rail");
    append_text(
        document,
        &header_rail,
        "div",
        "header-pill",
        &state_ref.status,
    );
    append_text(
        document,
        &header_rail,
        "div",
        "header-pill",
        &format!("Player 1: {} pts", score1),
    );
    append_text(
        document,
        &header_rail,
        "div",
        "header-pill",
        &format!("Player 2: {} pts", score2),
    );
    header
        .append_child(&header_rail)
        .expect("header rail should be appended");

    let board = create_element(document, "section", "table-board");
    board
        .append_child(&build_player_zone(
            document,
            &format!("Player 2 • {} cards", state_ref.player2_hand.len()),
            &state_ref.player2_hand,
            false,
            false,
            Rc::clone(&state),
        ))
        .expect("player 2 zone should be appended");

    let middle = create_element(document, "section", "table-middle");
    middle
        .append_child(&build_deck_panel(document, state_ref.deck_size))
        .expect("deck panel should be appended");
    middle
        .append_child(&build_trick_panel(document, &state_ref.trick_cards))
        .expect("trick panel should be appended");
    middle
        .append_child(&build_trump_panel(
            document,
            state_ref.briscola,
            state_ref.deck_size,
        ))
        .expect("trump panel should be appended");
    board
        .append_child(&middle)
        .expect("middle section should be appended");

    board
        .append_child(&build_player_zone(
            document,
            &format!("Player 1 • {} cards", state_ref.player1_hand.len()),
            &state_ref.player1_hand,
            true,
            state_ref.waiting_for_player1(),
            Rc::clone(&state),
        ))
        .expect("player 1 zone should be appended");

    let footer = create_element(document, "footer", "footer-bar");
    append_text(
        document,
        &footer,
        "div",
        "footer-pill",
        &format!("Leader: Player {}", state_ref.leader_index()),
    );
    append_text(
        document,
        &footer,
        "div",
        "footer-pill",
        &format!(
            "Won cards: {} / {}",
            state_ref.player1_won.len(),
            state_ref.player2_won.len()
        ),
    );
    if state_ref.waiting_for_player1() {
        append_text(document, &footer, "div", "footer-pill action", "Your move");
    }

    dashboard
        .append_child(&header)
        .expect("header should be appended");
    dashboard
        .append_child(&board)
        .expect("board should be appended");
    dashboard
        .append_child(&footer)
        .expect("footer should be appended");

    body.append_child(&dashboard)
        .expect("dashboard should be appended");
}

fn schedule_next_tick(state: Rc<RefCell<BrowserGame>>, delay_ms: i32) {
    let callback_state = Rc::clone(&state);
    let callback = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        tick(Rc::clone(&callback_state));
    }) as Box<dyn FnMut()>);

    window()
        .expect("window should exist")
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            delay_ms,
        )
        .expect("timeout should be scheduled");

    callback.forget();
}

fn select_player1_card(state: Rc<RefCell<BrowserGame>>, selected: usize) {
    let next_delay = {
        let mut game = state.borrow_mut();
        game.play_player1_card(selected)
    };

    render_dashboard(&document(), Rc::clone(&state));

    if let Some(delay_ms) = next_delay {
        schedule_next_tick(state, delay_ms);
    }
}

fn tick(state: Rc<RefCell<BrowserGame>>) {
    let next_delay = {
        let mut game = state.borrow_mut();
        let delay = game.advance();
        delay
    };

    render_dashboard(&document(), Rc::clone(&state));

    if let Some(delay_ms) = next_delay {
        schedule_next_tick(state, delay_ms);
    }
}

fn card_svg(card: card::Card) -> &'static str {
    match (card.suit, card.value) {
        (card::CardSuit::Batons, card::CardNumber::Ace) => {
            include_str!("../assets/briscola/bresciane/bastoni/asso.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Two) => {
            include_str!("../assets/briscola/bresciane/bastoni/02.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Three) => {
            include_str!("../assets/briscola/bresciane/bastoni/03.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Four) => {
            include_str!("../assets/briscola/bresciane/bastoni/04.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Five) => {
            include_str!("../assets/briscola/bresciane/bastoni/05.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Six) => {
            include_str!("../assets/briscola/bresciane/bastoni/06.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Seven) => {
            include_str!("../assets/briscola/bresciane/bastoni/07.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Knave) => {
            include_str!("../assets/briscola/bresciane/bastoni/fante.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::Knight) => {
            include_str!("../assets/briscola/bresciane/bastoni/cavallo.svg")
        }
        (card::CardSuit::Batons, card::CardNumber::King) => {
            include_str!("../assets/briscola/bresciane/bastoni/re.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Ace) => {
            include_str!("../assets/briscola/bresciane/coppe/asso.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Two) => {
            include_str!("../assets/briscola/bresciane/coppe/02.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Three) => {
            include_str!("../assets/briscola/bresciane/coppe/03.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Four) => {
            include_str!("../assets/briscola/bresciane/coppe/04.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Five) => {
            include_str!("../assets/briscola/bresciane/coppe/05.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Six) => {
            include_str!("../assets/briscola/bresciane/coppe/06.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Seven) => {
            include_str!("../assets/briscola/bresciane/coppe/07.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Knave) => {
            include_str!("../assets/briscola/bresciane/coppe/fante.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::Knight) => {
            include_str!("../assets/briscola/bresciane/coppe/cavallo.svg")
        }
        (card::CardSuit::Cups, card::CardNumber::King) => {
            include_str!("../assets/briscola/bresciane/coppe/re.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Ace) => {
            include_str!("../assets/briscola/bresciane/denari/asso.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Two) => {
            include_str!("../assets/briscola/bresciane/denari/02.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Three) => {
            include_str!("../assets/briscola/bresciane/denari/03.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Four) => {
            include_str!("../assets/briscola/bresciane/denari/04.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Five) => {
            include_str!("../assets/briscola/bresciane/denari/05.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Six) => {
            include_str!("../assets/briscola/bresciane/denari/06.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Seven) => {
            include_str!("../assets/briscola/bresciane/denari/07.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Knave) => {
            include_str!("../assets/briscola/bresciane/denari/fante.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::Knight) => {
            include_str!("../assets/briscola/bresciane/denari/cavallo.svg")
        }
        (card::CardSuit::Coins, card::CardNumber::King) => {
            include_str!("../assets/briscola/bresciane/denari/re.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Ace) => {
            include_str!("../assets/briscola/bresciane/spade/asso.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Two) => {
            include_str!("../assets/briscola/bresciane/spade/02.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Three) => {
            include_str!("../assets/briscola/bresciane/spade/03.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Four) => {
            include_str!("../assets/briscola/bresciane/spade/04.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Five) => {
            include_str!("../assets/briscola/bresciane/spade/05.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Six) => {
            include_str!("../assets/briscola/bresciane/spade/06.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Seven) => {
            include_str!("../assets/briscola/bresciane/spade/07.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Knave) => {
            include_str!("../assets/briscola/bresciane/spade/fante.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::Knight) => {
            include_str!("../assets/briscola/bresciane/spade/cavallo.svg")
        }
        (card::CardSuit::Swords, card::CardNumber::King) => {
            include_str!("../assets/briscola/bresciane/spade/re.svg")
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    let state = Rc::new(RefCell::new(BrowserGame::new()));
    render_dashboard(&document(), Rc::clone(&state));
    schedule_next_tick(state, 500);
}
