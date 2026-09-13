use std::cell::RefCell;
use std::rc::Rc;

use rand::Rng;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use web_sys::{window, Document, Element};

pub mod ai;
pub mod card;
pub mod console;
pub mod constants;
pub mod deck;
pub mod game;
pub mod hand;
pub mod painter;
pub mod player;
pub mod rules;

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
    ai_difficulty: ai::AiDifficulty,
    fill_screen: bool,
    last_dealt_player: Option<usize>,
    tick_generation: u32,
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
            status: "shuffle / deal".to_string(),
            trick_number: 1,
            deck_size: FULL_DECK_SIZE,
            draw_first_player: 1,
            draw_second_player: 2,
            ai_difficulty: ai::AiDifficulty::Challenger,
            fill_screen: false,
            last_dealt_player: None,
            tick_generation: 0,
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
            "challenger"
        }
    }

    fn player_sentence_label(player_index: usize) -> &'static str {
        if player_index == 1 {
            "You"
        } else {
            "Challenger"
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
            self.last_dealt_player = Some(player_index);
        }
    }

    fn play_ai_card(&mut self, player_index: usize) {
        let lead_card = self.trick_cards.first().map(|(_, card)| *card);
        let visible_state = ai::VisibleGameState {
            briscola_suit: self.briscola.suit,
            lead_card,
        };
        let difficulty = self.ai_difficulty;
        let hand = self.hand_mut(player_index);
        let selected = ai::choose_card(difficulty, visible_state, hand);
        let card = hand.remove(selected);
        self.trick_cards.push((player_index, card));
    }

    fn restart(&mut self) {
        let ai_difficulty = self.ai_difficulty;
        let fill_screen = self.fill_screen;
        let tick_generation = self.tick_generation.wrapping_add(1);
        *self = BrowserGame::new();
        self.ai_difficulty = ai_difficulty;
        self.fill_screen = fill_screen;
        self.tick_generation = tick_generation;
    }

    fn set_ai_difficulty(&mut self, ai_difficulty: ai::AiDifficulty) {
        self.ai_difficulty = ai_difficulty;
        self.restart();
    }

    fn toggle_fill_screen(&mut self) {
        self.fill_screen = !self.fill_screen;
    }

    fn tick_generation(&self) -> u32 {
        self.tick_generation
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
                self.status = format!("trick {} / you lead", self.trick_number);
                Some(1050)
            }
            Phase::Follow => {
                self.phase = Phase::Resolve;
                self.status = format!("trick {} / you answer", self.trick_number);
                Some(1150)
            }
            _ => None,
        }
    }

    fn wins_first(&self, card1: card::Card, card2: card::Card) -> bool {
        rules::wins_first(card1, card2, self.briscola.suit)
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
        rules::calculate_score(cards)
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
                        "deal complete / {} leads",
                        Self::player_sentence_label(self.leader_index())
                    );
                    Some(1200)
                } else {
                    let hand_size = if next_player == 1 {
                        self.player1_hand.len()
                    } else {
                        self.player2_hand.len()
                    };
                    self.status = format!(
                        "deal / {} / {} cards / {}/{}",
                        Self::player_label(next_player),
                        hand_size,
                        dealt_total,
                        constants::HAND_SIZE * 2,
                    );
                    Some(320)
                }
            }
            Phase::Lead => {
                let player = self.leader_index();
                if player == 1 {
                    self.status = format!("trick {} / select lead", self.trick_number);
                    return None;
                }
                self.play_ai_card(player);
                self.phase = Phase::Follow;
                self.status = format!("trick {} / challenger leads", self.trick_number);
                Some(1050)
            }
            Phase::Follow => {
                let player = Self::other_player(self.leader_index());
                if player == 1 {
                    self.status = format!("trick {} / select answer", self.trick_number);
                    return None;
                }
                self.play_ai_card(player);
                self.phase = Phase::Resolve;
                self.status = format!("trick {} / challenger answers", self.trick_number);
                Some(1150)
            }
            Phase::Resolve => {
                let winner = self.preview_trick_winner();
                self.pending_trick_winner = Some(winner);
                self.phase = Phase::Collect;
                self.status = format!(
                    "trick {} / {} wins",
                    self.trick_number,
                    Self::player_sentence_label(winner)
                );
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
                        "game / {} wins / {}-{}",
                        Self::player_sentence_label(self.game_winner()),
                        score1,
                        score2
                    );
                    None
                } else if self.deck_size > 0 {
                    self.draw_first_player = winner;
                    self.draw_second_player = Self::other_player(winner);
                    self.phase = Phase::DrawPlayer1;
                    self.status = format!(
                        "{} collects / winner draws first",
                        Self::player_sentence_label(winner)
                    );
                    Some(600)
                } else {
                    self.phase = Phase::Lead;
                    self.status =
                        format!("deck empty / {} leads", Self::player_sentence_label(winner));
                    Some(1100)
                }
            }
            Phase::DrawPlayer1 => {
                let player = self.draw_first_player;
                self.deal_card_to(player);
                self.phase = Phase::DrawPlayer2;
                self.status = format!("{} draws first.", Self::player_sentence_label(player));
                Some(600)
            }
            Phase::DrawPlayer2 => {
                if self.deck_size > 0 {
                    let player = self.draw_second_player;
                    self.deal_card_to(player);
                    self.status = format!("{} draws second.", Self::player_sentence_label(player));
                } else {
                    self.status = "deck empty / no second draw".to_string();
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
        .briscola-app {
            --terminal-bg: #000;
            --terminal-panel: #050505;
            --terminal-border: #303030;
            --terminal-active: #457294;
            --terminal-cyan: #00ffff;
            --terminal-green: #1cba22;
            --terminal-amber: #d8a100;
            --terminal-text: #ffffff;
            --terminal-muted: #9a9a9a;
            --card-width: clamp(48px, min(12vw, 13vh), 78px);
            box-sizing: border-box;
            width: 100%;
            min-height: min(620px, 100vh);
            background: var(--terminal-bg);
            color: var(--terminal-text);
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
        }

        .briscola-app.theme-white {
            --terminal-bg: #fff;
            --terminal-panel: #fff;
            --terminal-border: #d8d8d8;
            --terminal-active: #bcbcbc;
            --terminal-cyan: #111;
            --terminal-green: #111;
            --terminal-amber: #444;
            --terminal-text: #111;
            --terminal-muted: #666;
        }

        .briscola-app.fill-screen {
            position: fixed;
            inset: 0;
            z-index: 2147483647;
            height: 100vh;
            min-height: 100vh;
            overflow: hidden;
            --card-width: clamp(54px, min(13vw, 16vh), 104px);
        }

        .briscola-app *,
        .briscola-app *::before,
        .briscola-app *::after {
            box-sizing: border-box;
        }

        .briscola-app .dashboard {
            min-height: min(620px, 100vh);
            padding: 10px;
            display: grid;
            grid-template-rows: auto minmax(0, 1fr) auto;
            gap: 10px;
            background: var(--terminal-bg);
        }

        .briscola-app.fill-screen .dashboard {
            height: 100vh;
            min-height: 100vh;
        }

        .briscola-app .table-header {
            display: grid;
            grid-template-columns: minmax(0, 1fr) minmax(0, auto);
            gap: 8px;
            align-items: start;
            padding: 8px 10px;
            border: 1px solid var(--terminal-active);
            background: var(--terminal-panel);
        }

        .briscola-app .table-title {
            margin: 0;
            color: var(--terminal-green);
            font-size: 18px;
            line-height: 1.2;
            letter-spacing: 0;
            text-transform: lowercase;
        }

        .briscola-app .table-title::before {
            content: "$ ";
            color: var(--terminal-cyan);
        }

        .briscola-app .table-subtitle {
            margin: 2px 0 0;
            color: var(--terminal-muted);
            font-size: 12px;
            line-height: 1.35;
        }

        .briscola-app .header-rail,
        .briscola-app .footer-bar,
        .briscola-app .control-row {
            display: flex;
            flex-wrap: wrap;
            gap: 6px;
            align-items: center;
        }

        .briscola-app .header-rail {
            justify-content: flex-end;
        }

        .briscola-app .header-pill,
        .briscola-app .footer-pill {
            padding: 2px 6px;
            border: 1px solid var(--terminal-border);
            background: transparent;
            color: var(--terminal-text);
            font-size: 12px;
            line-height: 1.35;
        }

        .briscola-app .header-pill:first-child,
        .briscola-app .footer-pill.action {
            color: var(--terminal-amber);
            border-color: var(--terminal-amber);
        }

        .briscola-app .terminal-button {
            min-height: 24px;
            padding: 2px 8px;
            border: 1px solid var(--terminal-border);
            border-radius: 0;
            background: var(--terminal-bg);
            color: var(--terminal-cyan);
            cursor: pointer;
            font: inherit;
            font-size: 12px;
            line-height: 1.2;
        }

        .briscola-app .terminal-button:hover,
        .briscola-app .terminal-button:focus-visible,
        .briscola-app .terminal-button.active {
            outline: none;
            border-color: var(--terminal-cyan);
            background: #052323;
        }

        .briscola-app .terminal-button.active {
            color: var(--terminal-green);
        }

        .briscola-app.theme-white .terminal-button:hover,
        .briscola-app.theme-white .terminal-button:focus-visible,
        .briscola-app.theme-white .terminal-button.active {
            background: #f4f4f4;
        }

        .briscola-app .table-board {
            min-height: 0;
            padding: 10px;
            display: grid;
            grid-template-rows: auto auto auto;
            align-content: center;
            gap: 12px;
            border: 1px solid var(--terminal-border);
            background: var(--terminal-bg);
            overflow: auto;
        }

        .briscola-app .player-zone {
            display: grid;
            gap: 6px;
            justify-items: center;
        }

        .briscola-app .player-label,
        .briscola-app .stack-label,
        .briscola-app .trick-slot-label {
            margin: 0;
            color: var(--terminal-green);
            font-size: 12px;
            line-height: 1.25;
            letter-spacing: 0;
            text-transform: uppercase;
        }

        .briscola-app .player-hand {
            min-height: calc((var(--card-width) * 1.5) + 12px);
            display: flex;
            flex-wrap: wrap;
            justify-content: center;
            align-items: center;
            gap: 8px;
        }

        .briscola-app .table-middle {
            display: grid;
            grid-template-columns:
                minmax(calc(var(--card-width) * 1.55), calc(var(--card-width) * 2.15))
                minmax(calc(var(--card-width) * 2.45), calc(var(--card-width) * 3))
                minmax(calc(var(--card-width) * 1.55), calc(var(--card-width) * 2.15));
            justify-content: center;
            align-items: stretch;
            gap: 10px;
        }

        .briscola-app .stack-panel {
            padding: 8px;
            border: 1px solid var(--terminal-border);
            background: var(--terminal-panel);
            display: grid;
            justify-items: center;
            align-content: center;
            min-width: 0;
        }

        .briscola-app .stack-label {
            margin-bottom: 8px;
            color: var(--terminal-cyan);
        }

        .briscola-app .deck-stack,
        .briscola-app .trick-stack {
            position: relative;
            width: var(--card-width);
            height: calc(var(--card-width) * 1.5);
            margin: 0 auto;
        }

        .briscola-app .deck-stack .card-shell {
            position: absolute;
            inset: 0;
            width: 100%;
            min-width: 0;
        }

        .briscola-app .deck-stack .card-back {
            transform: none;
        }

        .briscola-app .deck-stack .card-back:nth-last-child(2) {
            transform: translate(4px, 3px);
        }

        .briscola-app .deck-stack .card-back:nth-last-child(3) {
            transform: translate(8px, 6px);
        }

        .briscola-app .trick-slots {
            display: grid;
            grid-template-columns: repeat(2, var(--card-width));
            justify-content: center;
            gap: 8px;
            justify-items: center;
            width: 100%;
        }

        .briscola-app .trick-slot {
            display: grid;
            gap: 6px;
            justify-items: center;
        }

        .briscola-app .trick-slot-label {
            grid-row: 2;
        }

        .briscola-app .trick-slot .card-shell,
        .briscola-app .trick-slot .card-placeholder {
            grid-row: 1;
        }

        .briscola-app .stack-count {
            margin: 8px 0 0;
            text-align: center;
            color: var(--terminal-green);
            font-size: 18px;
            line-height: 1.1;
            font-weight: 700;
        }

        .briscola-app .stack-caption {
            margin: 4px 0 0;
            text-align: center;
            color: var(--terminal-muted);
            font-size: 11px;
            line-height: 1.3;
        }

        .briscola-app .trump-panel::after {
            content: "";
            display: block;
            height: 1.1em;
            margin-top: 8px;
            font-size: 18px;
            line-height: 1.1;
        }

        .briscola-app .card-shell,
        .briscola-app .card-button,
        .briscola-app .card-placeholder {
            width: var(--card-width);
            min-width: var(--card-width);
            aspect-ratio: 2 / 3;
            border-radius: 2px;
            overflow: hidden;
            box-shadow: none;
            background: transparent;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .briscola-app .card-button {
            padding: 0;
            border: 1px solid transparent;
            cursor: pointer;
            transition: transform 140ms ease;
            background: transparent;
        }

        .briscola-app .card-button:hover,
        .briscola-app .card-button:focus-visible {
            outline: 1px solid var(--terminal-cyan);
            outline-offset: 2px;
            transform: translateY(-3px);
        }

        .briscola-app .card-shell svg,
        .briscola-app .card-button svg {
            width: 100%;
            height: 100%;
            display: block;
        }

        .briscola-app .card-back {
            border: 1px solid var(--terminal-border);
            background: transparent;
        }

        .briscola-app .dealt-card-to-challenger {
            animation: deal-to-challenger 280ms cubic-bezier(.2, .8, .2, 1) both;
        }

        .briscola-app .dealt-card-to-you {
            animation: deal-to-you 280ms cubic-bezier(.2, .8, .2, 1) both;
        }

        @keyframes deal-to-challenger {
            from {
                opacity: .35;
                transform: translateY(145px) scale(.92);
            }
            to {
                opacity: 1;
                transform: translateY(0) scale(1);
            }
        }

        @keyframes deal-to-you {
            from {
                opacity: .35;
                transform: translateY(-145px) scale(.92);
            }
            to {
                opacity: 1;
                transform: translateY(0) scale(1);
            }
        }

        .briscola-app .card-placeholder::after {
            content: "";
            width: calc(100% - 12px);
            height: calc(100% - 12px);
            border: 1px solid var(--terminal-border);
        }

        .briscola-app .card-placeholder {
            border: 1px dashed var(--terminal-border);
            background: var(--terminal-panel);
        }

        @media (max-width: 720px) {
            .briscola-app {
                --card-width: clamp(30px, min(10vw, 9vh), 42px);
            }

            .briscola-app.fill-screen {
                --card-width: clamp(32px, min(11vw, 10vh), 46px);
            }

            .briscola-app .dashboard {
                min-height: 100%;
                padding: 6px;
                gap: 8px;
            }

            .briscola-app .table-header {
                grid-template-columns: 1fr;
            }

            .briscola-app .table-middle {
                grid-template-columns: repeat(3, minmax(0, 1fr));
                justify-content: stretch;
                width: 100%;
                gap: 4px;
            }

            .briscola-app .stack-panel {
                padding: 4px;
                width: 100%;
            }

            .briscola-app .stack-label {
                margin-bottom: 4px;
            }

            .briscola-app .trick-slots {
                gap: 4px;
            }

            .briscola-app .player-label,
            .briscola-app .stack-label,
            .briscola-app .trick-slot-label {
                font-size: 10px;
            }

            .briscola-app .stack-count {
                margin-top: 4px;
                font-size: 14px;
            }

            .briscola-app .stack-caption {
                margin-top: 2px;
                font-size: 10px;
            }

            .briscola-app .trump-panel::after {
                margin-top: 4px;
                font-size: 14px;
            }

            .briscola-app .header-rail {
                justify-content: flex-start;
            }

            .briscola-app .player-hand {
                min-height: calc((var(--card-width) * 1.5) + 10px);
            }

            .briscola-app .card-shell,
            .briscola-app .card-button,
            .briscola-app .card-placeholder {
                width: var(--card-width);
                min-width: var(--card-width);
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

fn mount_element(document: &Document) -> Element {
    if let Some(element) = document.get_element_by_id("briscola-app") {
        return element;
    }

    let element = create_element(document, "div", "briscola-app");
    element
        .set_attribute("id", "briscola-app")
        .expect("mount id should be set");
    document
        .body()
        .expect("document body should exist")
        .append_child(&element)
        .expect("mount should be appended");
    element
}

fn production_theme(document: &Document) -> String {
    document
        .body()
        .and_then(|body| body.get_attribute("data-production-theme"))
        .unwrap_or_else(|| "terminal".to_string())
}

fn app_class_name(document: &Document, fill_screen: bool) -> String {
    let mut class_name = "briscola-app".to_string();
    if production_theme(document) == "white" {
        class_name.push_str(" theme-white");
    }
    if fill_screen {
        class_name.push_str(" fill-screen");
    }
    class_name
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

fn render_action_button<F>(document: &Document, label: &str, active: bool, handler: F) -> Element
where
    F: 'static + FnMut(),
{
    let class_name = if active {
        "terminal-button active"
    } else {
        "terminal-button"
    };
    let button = create_element(document, "button", class_name);
    button
        .set_attribute("type", "button")
        .expect("button type should be set");
    button.set_text_content(Some(label));

    let callback = Closure::<dyn FnMut()>::wrap(Box::new(handler) as Box<dyn FnMut()>);
    button
        .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
        .expect("button click handler should be attached");
    callback.forget();
    button
}

fn build_controls(doc: &Document, state: Rc<RefCell<BrowserGame>>) -> Element {
    let controls = create_element(doc, "div", "control-row");

    let restart_state = Rc::clone(&state);
    controls
        .append_child(&render_action_button(doc, "NEW", false, move || {
            {
                let mut game = restart_state.borrow_mut();
                game.restart();
            }
            render_dashboard(&document(), Rc::clone(&restart_state));
            let tick_generation = restart_state.borrow().tick_generation();
            schedule_next_tick(Rc::clone(&restart_state), 500, tick_generation);
        }))
        .expect("restart button should be appended");

    let fill_state = Rc::clone(&state);
    controls
        .append_child(&render_action_button(
            doc,
            "BIGGER VIEW",
            state.borrow().fill_screen,
            move || {
                {
                    let mut game = fill_state.borrow_mut();
                    game.toggle_fill_screen();
                }
                render_dashboard(&document(), Rc::clone(&fill_state));
            },
        ))
        .expect("fill button should be appended");

    for (label, difficulty) in [
        ("RANDOM", ai::AiDifficulty::Random),
        ("CHALLENGER", ai::AiDifficulty::Challenger),
    ] {
        let difficulty_state = Rc::clone(&state);
        let active = state.borrow().ai_difficulty == difficulty;
        controls
            .append_child(&render_action_button(doc, label, active, move || {
                {
                    let mut game = difficulty_state.borrow_mut();
                    game.set_ai_difficulty(difficulty);
                }
                render_dashboard(&document(), Rc::clone(&difficulty_state));
                let tick_generation = difficulty_state.borrow().tick_generation();
                schedule_next_tick(Rc::clone(&difficulty_state), 500, tick_generation);
            }))
            .expect("difficulty button should be appended");
    }

    controls
}

fn render_card_back(document: &Document) -> Element {
    let card_shell = create_element(document, "div", "card-shell card-back");
    card_shell.set_inner_html(include_str!("../assets/briscola/bresciane/back.svg"));
    card_shell
}

fn render_placeholder_card(document: &Document) -> Element {
    create_element(document, "div", "card-placeholder")
}

fn build_player_zone(
    document: &Document,
    label: &str,
    player_index: usize,
    cards: &[card::Card],
    face_up: bool,
    selectable: bool,
    animate_dealt_card: bool,
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
        if animate_dealt_card && index + 1 == cards.len() {
            let dealt_class = if player_index == 1 {
                "dealt-card-to-you"
            } else {
                "dealt-card-to-challenger"
            };
            card_element.set_class_name(&format!("{} {}", card_element.class_name(), dealt_class));
        }
        hand.append_child(&card_element)
            .expect("player card should be appended");
    }

    zone.append_child(&hand)
        .expect("player hand should be appended");
    zone
}

fn build_deck_panel(document: &Document, deck_size: usize) -> Element {
    let panel = create_element(document, "section", "stack-panel deck-panel");
    append_text(document, &panel, "p", "stack-label", "DECK");

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
    append_text(document, &panel, "p", "stack-caption", "remaining");
    panel
}

fn build_trick_panel(document: &Document, trick_cards: &[(usize, card::Card)]) -> Element {
    let panel = create_element(document, "section", "stack-panel trick-panel");
    append_text(document, &panel, "p", "stack-label", "TRICK");

    let slots = create_element(document, "div", "trick-slots");
    for player_index in [2usize, 1usize] {
        let slot = create_element(document, "div", "trick-slot");
        let card_node = trick_cards
            .iter()
            .find(|(owner, _)| *owner == player_index)
            .map(|(_, card)| render_card_svg(document, *card))
            .unwrap_or_else(|| render_placeholder_card(document));

        slot.append_child(&card_node)
            .expect("trick card should be appended");
        append_text(
            document,
            &slot,
            "p",
            "trick-slot-label",
            if player_index == 1 {
                "YOU"
            } else {
                "CHALLENGER"
            },
        );
        slots
            .append_child(&slot)
            .expect("trick slot should be appended");
    }

    panel
        .append_child(&slots)
        .expect("trick slots should be appended");
    append_text(document, &panel, "p", "stack-caption", "current");
    panel
}

fn build_trump_panel(document: &Document, briscola: card::Card, deck_size: usize) -> Element {
    let panel = create_element(document, "section", "stack-panel trump-panel");
    append_text(document, &panel, "p", "stack-label", "TRUMP");
    if deck_size == 0 {
        panel
            .append_child(&render_placeholder_card(document))
            .expect("briscola placeholder should be appended");
        append_text(document, &panel, "p", "stack-caption", "drawn");
    } else {
        panel
            .append_child(&render_card_svg(document, briscola))
            .expect("briscola card should be appended");
        append_text(document, &panel, "p", "stack-caption", "last card");
    }
    panel
}

fn render_dashboard(document: &Document, state: Rc<RefCell<BrowserGame>>) {
    set_styles(document);
    let mount = mount_element(document);
    mount.set_inner_html("");

    let state_ref = state.borrow();
    mount.set_class_name(&app_class_name(document, state_ref.fill_screen));
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
        "wasm / browser game",
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
        &format!("YOU: {} pts", score1),
    );
    append_text(
        document,
        &header_rail,
        "div",
        "header-pill",
        &format!("CHALLENGER: {} pts", score2),
    );
    header_rail
        .append_child(&build_controls(document, Rc::clone(&state)))
        .expect("controls should be appended");
    header
        .append_child(&header_rail)
        .expect("header rail should be appended");

    let board = create_element(document, "section", "table-board");
    board
        .append_child(&build_player_zone(
            document,
            &format!(
                "CHALLENGER [{}] / {} cards",
                state_ref.ai_difficulty.label(),
                state_ref.player2_hand.len()
            ),
            2,
            &state_ref.player2_hand,
            false,
            false,
            state_ref.last_dealt_player == Some(2),
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
            &format!("YOU / {} cards", state_ref.player1_hand.len()),
            1,
            &state_ref.player1_hand,
            true,
            state_ref.waiting_for_player1(),
            state_ref.last_dealt_player == Some(1),
            Rc::clone(&state),
        ))
        .expect("player 1 zone should be appended");

    let footer = create_element(document, "footer", "footer-bar");
    append_text(
        document,
        &footer,
        "div",
        "footer-pill",
        &format!(
            "LEADER: {}",
            if state_ref.leader_index() == 1 {
                "YOU"
            } else {
                "CHALLENGER"
            }
        ),
    );
    append_text(
        document,
        &footer,
        "div",
        "footer-pill",
        &format!(
            "WON: {} / {}",
            state_ref.player1_won.len(),
            state_ref.player2_won.len()
        ),
    );
    if state_ref.waiting_for_player1() {
        append_text(document, &footer, "div", "footer-pill action", "YOUR MOVE");
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

    mount
        .append_child(&dashboard)
        .expect("dashboard should be appended");

    drop(state_ref);
    state.borrow_mut().last_dealt_player = None;
}

fn schedule_next_tick(state: Rc<RefCell<BrowserGame>>, delay_ms: i32, tick_generation: u32) {
    let callback_state = Rc::clone(&state);
    let callback = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        tick(Rc::clone(&callback_state), tick_generation);
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
        let tick_generation = state.borrow().tick_generation();
        schedule_next_tick(state, delay_ms, tick_generation);
    }
}

fn tick(state: Rc<RefCell<BrowserGame>>, tick_generation: u32) {
    if state.borrow().tick_generation() != tick_generation {
        return;
    }

    let next_delay = {
        let mut game = state.borrow_mut();
        let delay = game.advance();
        delay
    };

    render_dashboard(&document(), Rc::clone(&state));

    if let Some(delay_ms) = next_delay {
        schedule_next_tick(state, delay_ms, tick_generation);
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
    schedule_next_tick(state, 500, 0);
}
