use std::cell::RefCell;
use std::rc::Rc;

use rand::Rng;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use web_sys::{window, Document, Element, Event, EventTarget};

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
const COIN_TOSS_DISPLAY_MS: i32 = 2200;

#[derive(Clone, Copy)]
enum Phase {
    Setup,
    CoinToss,
    Dealing,
    Lead,
    TrumpReveal,
    Follow,
    Resolve,
    Collect,
    DrawPlayer1,
    DrawPlayer2,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GameMode {
    HumanVsAi,
    AiVsAi,
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
    mode: GameMode,
    player1_difficulty: ai::AiDifficulty,
    player2_difficulty: ai::AiDifficulty,
    fill_screen: bool,
    last_dealt_player: Option<usize>,
    last_played_player: Option<usize>,
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
            phase: Phase::Setup,
            status: "choose game mode".to_string(),
            trick_number: 1,
            deck_size: FULL_DECK_SIZE,
            draw_first_player: 1,
            draw_second_player: 2,
            mode: GameMode::HumanVsAi,
            player1_difficulty: ai::AiDifficulty::Random,
            player2_difficulty: ai::AiDifficulty::Challenger,
            fill_screen: false,
            last_dealt_player: None,
            last_played_player: None,
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

    fn player_label(&self, player_index: usize) -> &'static str {
        match (self.mode, player_index) {
            (GameMode::HumanVsAi, 1) => "you",
            (GameMode::HumanVsAi, _) => "ai",
            (GameMode::AiVsAi, 1) => "ai 1",
            (GameMode::AiVsAi, _) => "ai 2",
        }
    }

    fn player_display_label(&self, player_index: usize) -> &'static str {
        Self::player_display_label_for(self.mode, player_index)
    }

    fn player_display_label_for(mode: GameMode, player_index: usize) -> &'static str {
        match (mode, player_index) {
            (GameMode::HumanVsAi, 1) => "YOU",
            (GameMode::HumanVsAi, _) => "AI",
            (GameMode::AiVsAi, 1) => "AI 1",
            (GameMode::AiVsAi, _) => "AI 2",
        }
    }

    fn player_leads_label(&self, player_index: usize) -> String {
        match self.mode {
            GameMode::HumanVsAi if player_index == 1 => "You lead".to_string(),
            _ => format!("{} leads", self.player_display_label(player_index)),
        }
    }

    fn player_wins_label(&self, player_index: usize) -> String {
        match self.mode {
            GameMode::HumanVsAi if player_index == 1 => "You win".to_string(),
            _ => format!("{} wins", self.player_display_label(player_index)),
        }
    }

    fn player_collects_label(&self, player_index: usize) -> String {
        match self.mode {
            GameMode::HumanVsAi if player_index == 1 => "You collect".to_string(),
            _ => format!("{} collects", self.player_display_label(player_index)),
        }
    }

    fn player_draws_label(&self, player_index: usize, draw_order: &'static str) -> String {
        match self.mode {
            GameMode::HumanVsAi if player_index == 1 => format!("You draw {draw_order}."),
            _ => format!(
                "{} draws {draw_order}.",
                self.player_display_label(player_index)
            ),
        }
    }

    fn is_human_player(&self, player_index: usize) -> bool {
        self.mode == GameMode::HumanVsAi && player_index == 1
    }

    fn ai_difficulty_for(&self, player_index: usize) -> ai::AiDifficulty {
        if player_index == 1 {
            self.player1_difficulty
        } else {
            self.player2_difficulty
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
        let difficulty = self.ai_difficulty_for(player_index);
        let hand = self.hand_mut(player_index);
        let selected = ai::choose_card(difficulty, visible_state, hand);
        let card = hand.remove(selected);
        self.trick_cards.push((player_index, card));
        self.last_played_player = Some(player_index);
    }

    fn restart(&mut self) {
        let mode = self.mode;
        let player1_difficulty = self.player1_difficulty;
        let player2_difficulty = self.player2_difficulty;
        let fill_screen = self.fill_screen;
        let tick_generation = self.tick_generation.wrapping_add(1);
        *self = BrowserGame::new();
        self.mode = mode;
        self.player1_difficulty = player1_difficulty;
        self.player2_difficulty = player2_difficulty;
        self.fill_screen = fill_screen;
        self.tick_generation = tick_generation;
    }

    fn set_player_difficulty(&mut self, player_index: usize, difficulty: ai::AiDifficulty) {
        if player_index == 1 {
            self.player1_difficulty = difficulty;
        } else {
            self.player2_difficulty = difficulty;
        }
    }

    fn start_game(&mut self) {
        if matches!(self.phase, Phase::Setup) {
            self.phase = Phase::CoinToss;
            self.status = "coin toss / deciding who plays first".to_string();
        }
    }

    fn toggle_fill_screen(&mut self) {
        self.fill_screen = !self.fill_screen;
    }

    fn tick_generation(&self) -> u32 {
        self.tick_generation
    }

    fn waiting_for_player1(&self) -> bool {
        if !self.is_human_player(1) {
            return false;
        }
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
        self.last_played_player = Some(1);

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

    fn game_winner(&self) -> Option<usize> {
        let (score1, score2) = self.scores();
        rules::winner_from_scores(score1, score2)
    }

    fn result_label(&self) -> &'static str {
        match self.game_winner() {
            Some(1) if self.mode == GameMode::HumanVsAi => "You win!",
            Some(1) => "AI 1 wins!",
            Some(_) if self.mode == GameMode::HumanVsAi => "AI wins!",
            Some(_) => "AI 2 wins!",
            None => "Draw",
        }
    }

    fn advance(&mut self) -> Option<i32> {
        match self.phase {
            Phase::Setup => None,
            Phase::CoinToss => {
                self.phase = Phase::TrumpReveal;
                self.status = format!("trump reveal / {}", trump_suit_name(self.briscola.suit));
                Some(3200)
            }
            Phase::Dealing => {
                let dealt_cards = self.player1_hand.len() + self.player2_hand.len();
                let next_player = if dealt_cards % 2 == 0 { 1 } else { 2 };
                self.deal_card_to(next_player);

                let dealt_total = dealt_cards + 1;
                if dealt_total == constants::HAND_SIZE * 2 {
                    self.phase = Phase::Lead;
                    self.status = format!(
                        "deal complete / {}",
                        self.player_leads_label(self.leader_index())
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
                        self.player_label(next_player),
                        hand_size,
                        dealt_total,
                        constants::HAND_SIZE * 2,
                    );
                    Some(320)
                }
            }
            Phase::Lead => {
                let player = self.leader_index();
                if self.is_human_player(player) {
                    self.status = format!("trick {} / select lead", self.trick_number);
                    return None;
                }
                self.play_ai_card(player);
                self.phase = Phase::Follow;
                self.status = format!(
                    "trick {} / {}",
                    self.trick_number,
                    self.player_leads_label(player)
                );
                Some(1600)
            }
            Phase::TrumpReveal => {
                self.phase = Phase::Dealing;
                self.status = "shuffle / deal".to_string();
                Some(320)
            }
            Phase::Follow => {
                let player = Self::other_player(self.leader_index());
                if self.is_human_player(player) {
                    self.status = format!("trick {} / select answer", self.trick_number);
                    return None;
                }
                self.play_ai_card(player);
                self.phase = Phase::Resolve;
                self.status = format!(
                    "trick {} / {} answers",
                    self.trick_number,
                    self.player_display_label(player)
                );
                Some(1300)
            }
            Phase::Resolve => {
                let winner = self.preview_trick_winner();
                self.pending_trick_winner = Some(winner);
                self.phase = Phase::Collect;
                self.status = format!(
                    "trick {} / {}",
                    self.trick_number,
                    self.player_wins_label(winner)
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
                    self.status = format!("game / {} / {}-{}", self.result_label(), score1, score2);
                    None
                } else if self.deck_size > 0 {
                    self.draw_first_player = winner;
                    self.draw_second_player = Self::other_player(winner);
                    self.phase = Phase::DrawPlayer1;
                    self.status = format!(
                        "{} / winner draws first",
                        self.player_collects_label(winner)
                    );
                    Some(600)
                } else {
                    self.phase = Phase::Lead;
                    self.status = format!("deck empty / {}", self.player_leads_label(winner));
                    Some(1100)
                }
            }
            Phase::DrawPlayer1 => {
                let player = self.draw_first_player;
                self.deal_card_to(player);
                self.phase = Phase::DrawPlayer2;
                self.status = self.player_draws_label(player, "first");
                Some(600)
            }
            Phase::DrawPlayer2 => {
                if self.deck_size > 0 {
                    let player = self.draw_second_player;
                    self.deal_card_to(player);
                    self.status = self.player_draws_label(player, "second");
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
            --reveal-card-width: 128px;
            box-sizing: border-box;
            width: 100%;
            min-height: min(620px, 100vh);
            background: var(--terminal-bg);
            color: var(--terminal-text);
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
        }

        .briscola-app .table-board { position: relative; }
        .briscola-app .victory-layer {
            position: absolute; inset: 0; overflow: hidden; pointer-events: none;
            z-index: 2; display: grid; place-items: center;
        }
        .briscola-app .coin-toss-layer {
            position: absolute; inset: 0; z-index: 3; display: grid; place-items: center;
            padding: 20px; background: color-mix(in srgb, var(--terminal-bg) 82%, transparent);
        }
        .briscola-app .coin-toss-modal {
            width: min(320px, 100%); padding: 24px; text-align: center;
            border: 1px solid var(--terminal-cyan); background: var(--terminal-panel);
            box-shadow: 0 0 28px color-mix(in srgb, var(--terminal-cyan) 28%, transparent);
        }
        .briscola-app .coin-toss-modal h2 { margin: 0 0 14px; font-size: 16px; color: var(--terminal-cyan); }
        .briscola-app .coin-toss-coin {
            width: var(--reveal-card-width); height: calc(var(--reveal-card-width) * 1.5);
            margin: 0 auto 16px; perspective: 700px;
            animation: briscola-coin-toss 1500ms cubic-bezier(.2, .75, .3, 1) both;
        }
        .briscola-app .trump-reveal-layer {
            position: absolute; inset: 0; z-index: 3; display: grid; place-items: center;
            padding: 20px; background: color-mix(in srgb, var(--terminal-bg) 82%, transparent);
        }
        .briscola-app .trump-reveal-modal {
            width: min(320px, 100%); padding: 24px; text-align: center;
            border: 1px solid var(--terminal-cyan); background: var(--terminal-panel);
            box-shadow: 0 0 28px color-mix(in srgb, var(--terminal-cyan) 28%, transparent);
        }
        .briscola-app .trump-reveal-modal h2 { margin: 0 0 14px; font-size: 16px; color: var(--terminal-cyan); }
        .briscola-app .trump-reveal-modal .card-shell {
            width: var(--reveal-card-width); min-width: var(--reveal-card-width);
            margin: 0 auto 14px;
        }
        .briscola-app .trump-reveal-modal p { margin: 0; font-size: 15px; }
        .briscola-app .game-setup-layer {
            position: absolute; inset: 0; z-index: 4; display: grid; place-items: center;
            padding: 16px; background: var(--terminal-bg);
        }
        .briscola-app .game-setup-modal {
            width: min(420px, 100%); max-height: 100%; overflow-y: auto;
            padding: 20px; border: 1px solid var(--terminal-cyan);
            background: var(--terminal-panel);
            box-shadow: 0 0 28px color-mix(in srgb, var(--terminal-cyan) 24%, transparent);
        }
        .briscola-app .game-setup-modal h2 {
            margin: 0 0 16px; color: var(--terminal-cyan); font-size: 17px; text-align: center;
        }
        .briscola-app .setup-mode-options,
        .briscola-app .setup-difficulty-options {
            display: flex; flex-wrap: wrap; justify-content: center; gap: 8px;
        }
        .briscola-app .setup-section-label {
            margin: 16px 0 7px; color: var(--terminal-muted); font-size: 12px; text-align: center;
        }
        .briscola-app .game-setup-modal > .terminal-button {
            display: block; margin: 18px auto 0;
        }
        .briscola-app .coin-toss-card {
            position: relative; width: var(--reveal-card-width);
            height: calc(var(--reveal-card-width) * 1.5); transform-style: preserve-3d;
            transform-origin: center center;
            animation: briscola-coin-flip-ai 1500ms ease-in-out both;
        }
        .briscola-app .coin-toss-face {
            position: absolute; inset: 0; width: var(--reveal-card-width);
            height: calc(var(--reveal-card-width) * 1.5); overflow: hidden;
            backface-visibility: hidden; -webkit-backface-visibility: hidden;
            transform: translateZ(.5px);
        }
        .briscola-app .coin-toss-face .card-shell {
            width: var(--reveal-card-width); min-width: var(--reveal-card-width);
            height: calc(var(--reveal-card-width) * 1.5); aspect-ratio: auto;
        }
        .briscola-app .coin-toss-face.back { transform: rotateY(180deg) translateZ(.5px); }
        .briscola-app .coin-toss-card.player-opens { animation-name: briscola-coin-flip-player; }
        @keyframes briscola-coin-flip-ai {
            0% { transform: rotateY(0deg); }
            100% { transform: rotateY(1440deg); }
        }
        @keyframes briscola-coin-flip-player {
            0% { transform: rotateY(0deg); }
            100% { transform: rotateY(1620deg); }
        }
        .briscola-app .coin-toss-modal p { margin: 0; }
        @keyframes briscola-coin-toss {
            0% { opacity: 0; transform: translateY(85px) scale(.7); }
            20% { opacity: 1; }
            55% { transform: translateY(-72px) scale(1.08); }
            100% { opacity: 1; transform: translateY(0) scale(1); }
        }
        .briscola-app .victory-result {
            text-align: center; padding: 20px; border: 1px solid var(--terminal-active);
            background: var(--terminal-panel); color: var(--terminal-text);
            position: relative; z-index: 1;
        }
        .briscola-app .victory-result h2 { margin: 0 0 10px; }
        .briscola-app .victory-result p { margin: 0; }
        .briscola-app .firework-burst { position: absolute; width: 8px; height: 8px; }
        .briscola-app .firework-burst:nth-child(1) { left: 24%; top: 24%; }
        .briscola-app .firework-burst:nth-child(2) { left: 52%; top: 18%; }
        .briscola-app .firework-burst:nth-child(3) { left: 76%; top: 30%; }
        .briscola-app .firework-burst span {
            position: absolute; width: 6px; height: 6px; border-radius: 50%;
            background: var(--terminal-green); opacity: 0;
            animation: briscola-spark-pop 1600ms ease-out 3 both;
        }
        .briscola-app .firework-burst span:nth-child(3n + 1) { background: var(--terminal-cyan); }
        .briscola-app .firework-burst span:nth-child(3n + 2) { background: var(--terminal-amber); }
        @keyframes briscola-spark-pop {
            0% { opacity: 0; transform: rotate(var(--angle)) translateX(0) scale(.4); }
            16% { opacity: .95; }
            100% { opacity: 0; transform: rotate(var(--angle)) translateX(var(--distance)) scale(.8); }
        }
        @media (prefers-reduced-motion: reduce) {
            .briscola-app .firework-burst { display: none; }
            .briscola-app .dealt-card-to-you, .briscola-app .dealt-card-to-challenger,
            .briscola-app .played-card-from-you, .briscola-app .played-card-from-challenger,
            .briscola-app .coin-toss-coin, .briscola-app .coin-toss-card {
                animation: none;
            }
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
            --card-width: clamp(54px, min(15vw, 18vh), 128px);
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

        .briscola-app .control-label {
            color: var(--terminal-muted);
            font-size: 11px;
            line-height: 1.2;
            white-space: nowrap;
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

        .briscola-app .trick-status {
            padding: 7px 12px;
            font-size: 16px;
            font-weight: 700;
            line-height: 1.4;
            border-width: 2px;
            background: #211900;
        }

        .briscola-app.theme-white .trick-status {
            background: #fff5d6;
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
            border: 0;
            background: transparent;
        }

        .briscola-app .card-zoom {
            position: fixed;
            z-index: 100;
            pointer-events: none;
            filter: drop-shadow(0 8px 12px #0009);
            aspect-ratio: 2 / 3;
        }

        .briscola-app .card-zoom-image svg {
            display: block;
            width: 100%;
            height: 100%;
        }

        .briscola-app .card-zoom-hint {
            padding: 2px;
            background: var(--terminal-panel);
            color: var(--terminal-text);
            border: 1px solid var(--terminal-active);
            font: 11px/14px ui-monospace, monospace;
            text-align: center;
        }

        .briscola-app .card-zoom-instruction {
            display: block;
            margin-top: 1px;
            color: var(--terminal-muted);
        }

        .briscola-app .card-zoom-name {
            display: block;
            padding: 0 2px;
            background: #180605;
            color: #ff554d;
            font-weight: 700;
            letter-spacing: .08em;
            text-shadow: 0 0 5px #ff251e;
            text-transform: uppercase;
        }

        .briscola-app.theme-white .card-zoom-name {
            background: #fff;
            color: #a32620;
            text-shadow: none;
        }

        .briscola-app .dealt-card-to-challenger {
            animation: deal-to-challenger 280ms cubic-bezier(.2, .8, .2, 1) both;
        }

        .briscola-app .dealt-card-to-you {
            animation: deal-to-you 280ms cubic-bezier(.2, .8, .2, 1) both;
        }

        .briscola-app .played-card-from-challenger {
            animation: play-from-challenger 320ms cubic-bezier(.16, .9, .24, 1) both;
        }

        .briscola-app .played-card-from-you {
            animation: play-from-you 320ms cubic-bezier(.16, .9, .24, 1) both;
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

        @keyframes play-from-challenger {
            from {
                opacity: .35;
                transform: translateY(-90px) scale(.9);
            }
            to {
                opacity: 1;
                transform: translateY(0) scale(1);
            }
        }

        @keyframes play-from-you {
            from {
                opacity: .35;
                transform: translateY(90px) scale(.9);
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
            .briscola-app .bigger-view-toggle {
                display: none;
            }

            .briscola-app {
                --card-width: clamp(40px, min(16vw, 13vh), 64px);
                --reveal-card-width: clamp(140px, 43vw, 168px);
                min-height: 100vh;
                min-height: 100dvh;
            }

            .briscola-app.fill-screen {
                --card-width: clamp(60px, 24vw, 96px);
                --middle-card-width: clamp(68px, min(22vw, 13dvh), 92px);
            }

            .briscola-app .dashboard {
                min-height: 100vh;
                min-height: 100dvh;
                padding: 6px;
                gap: 8px;
            }

            .briscola-app.fill-screen,
            .briscola-app.fill-screen .dashboard {
                height: 100vh;
                height: 100dvh;
                min-height: 100vh;
                min-height: 100dvh;
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

            .briscola-app.fill-screen .table-middle {
                grid-template-columns: minmax(0, 1.1fr) minmax(0, 2.3fr) minmax(0, 1.1fr);
            }

            .briscola-app.fill-screen .table-board {
                padding: 8px;
                gap: 8px;
                align-content: space-evenly;
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
                align-items: flex-start;
                min-width: 0;
                flex-wrap: nowrap;
                gap: 4px;
            }

            .briscola-app .trick-status {
                order: 1;
                flex: 1 1 auto;
                min-width: 0;
                text-align: center;
                overflow-wrap: anywhere;
            }

            .briscola-app .header-rail .control-row {
                order: 0;
                flex: 0 0 auto;
                flex-wrap: nowrap;
            }

            .briscola-app .footer-bar {
                flex-wrap: nowrap;
                justify-content: space-between;
                gap: 4px;
            }

            .briscola-app .footer-pill {
                padding: 2px 4px;
                font-size: 10px;
                white-space: nowrap;
            }

            .briscola-app .won-count {
                display: none;
            }

            .briscola-app .player-hand {
                min-height: calc((var(--card-width) * 1.5) + 10px);
            }

            .briscola-app .player-zone {
                display: grid;
                grid-template-columns: 34px minmax(0, 1fr);
                align-items: center;
                justify-items: stretch;
                gap: 4px;
                min-width: 0;
            }

            .briscola-app .player-zone .player-label {
                text-align: left;
            }

            .briscola-app .player-hand {
                min-width: 0;
                flex-wrap: nowrap;
                justify-content: center;
                gap: 6px;
            }

            .briscola-app .player-hand > .card-shell,
            .briscola-app .player-hand > .card-button {
                width: calc((100% - 12px) / 3);
                min-width: calc((100% - 12px) / 3);
            }

            .briscola-app .card-shell,
            .briscola-app .card-button,
            .briscola-app .card-placeholder {
                width: var(--card-width);
                min-width: var(--card-width);
            }

            .briscola-app.fill-screen .player-hand > .card-shell,
            .briscola-app.fill-screen .player-hand > .card-button {
                width: min(104px, calc((100% - 12px) / 3));
                min-width: min(104px, calc((100% - 12px) / 3));
            }

            .briscola-app.fill-screen .deck-stack,
            .briscola-app.fill-screen .trick-stack {
                width: var(--middle-card-width);
                height: calc(var(--middle-card-width) * 1.5);
            }

            .briscola-app.fill-screen .deck-stack .card-shell {
                width: 100%;
                min-width: 0;
            }

            .briscola-app.fill-screen .deck-stack .card-back:nth-last-child(2) {
                transform: translateY(2px);
            }

            .briscola-app.fill-screen .deck-stack .card-back:nth-last-child(3) {
                transform: translateY(4px);
            }

            .briscola-app.fill-screen .trick-slots {
                grid-template-columns: repeat(2, var(--middle-card-width));
                gap: 2px;
            }

            .briscola-app.fill-screen .trick-slot > .card-shell,
            .briscola-app.fill-screen .trick-slot > .card-placeholder,
            .briscola-app.fill-screen .trick-slot > .card-button {
                width: var(--middle-card-width);
                min-width: var(--middle-card-width);
            }

            .briscola-app.fill-screen .trump-panel > .card-shell,
            .briscola-app.fill-screen .trump-panel > .card-placeholder {
                width: var(--middle-card-width);
                min-width: var(--middle-card-width);
            }

            .briscola-app.fill-screen .stack-panel {
                padding: 6px 3px;
            }

            .briscola-app.fill-screen .stack-label,
            .briscola-app.fill-screen .trick-slot-label {
                font-size: 14px;
            }

            .briscola-app.fill-screen .stack-count {
                font-size: 18px;
            }

            .briscola-app.fill-screen .stack-caption {
                font-size: 12px;
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

fn closest_preview_card(target: &EventTarget, app: &Element) -> Option<Element> {
    let element = target.dyn_ref::<Element>()?;
    let card = element.closest("[data-preview-card='true']").ok()??;
    if app.contains(Some(&card)) {
        Some(card)
    } else {
        None
    }
}

fn dismiss_card_preview(app: &Element) {
    if let Ok(Some(preview)) = app.query_selector(".card-zoom") {
        preview.remove();
    }
}

fn show_card_preview(app: &Element, card: &Element, pinned: bool) {
    let Some(svg) = card.query_selector("svg").ok().flatten() else {
        return;
    };
    let Some(document) = app.owner_document() else {
        return;
    };
    dismiss_card_preview(app);

    let preview = create_element(&document, "div", "card-zoom");
    let _ = preview.set_attribute("aria-hidden", "true");
    let _ = preview.set_attribute("data-pinned", if pinned { "true" } else { "false" });
    let picture = create_element(&document, "div", "card-zoom-image");
    if let Ok(svg_clone) = svg.clone_node_with_deep(true) {
        let _ = picture.append_child(&svg_clone);
    }
    let hint = create_element(&document, "div", "card-zoom-hint");
    let name = create_element(&document, "span", "card-zoom-name");
    name.set_text_content(Some(
        &card
            .get_attribute("data-card-name")
            .unwrap_or_else(|| "Card".to_string()),
    ));
    let _ = hint.append_child(&name);
    if pinned {
        let instruction = create_element(&document, "span", "card-zoom-instruction");
        instruction.set_text_content(Some(if card.class_list().contains("card-button") {
            "Tap card again to play"
        } else {
            "Tap outside to close"
        }));
        let _ = hint.append_child(&instruction);
    }
    let _ = preview.append_child(&picture);
    let _ = preview.append_child(&hint);

    let rect = card.get_bounding_client_rect();
    let width = rect.width().max(1.0);
    let image_width = (width * 2.2).min(240.0).max(1.0);
    let caption_height = if pinned { 38.0 } else { 22.0 };
    let visual_viewport = window().and_then(|window| window.visual_viewport());
    let viewport_left = visual_viewport
        .as_ref()
        .map_or(0.0, |viewport| viewport.offset_left());
    let viewport_top = visual_viewport
        .as_ref()
        .map_or(0.0, |viewport| viewport.offset_top());
    let viewport_width = visual_viewport
        .as_ref()
        .map(|viewport| viewport.width())
        .or_else(|| window().and_then(|window| window.inner_width().ok()?.as_f64()))
        .unwrap_or(1024.0);
    let viewport_height = visual_viewport
        .as_ref()
        .map(|viewport| viewport.height())
        .or_else(|| window().and_then(|window| window.inner_height().ok()?.as_f64()))
        .unwrap_or(768.0);
    let image_width = image_width
        .min((viewport_width - 24.0).max(1.0))
        .min(((viewport_height - 24.0 - caption_height) / 1.5).max(1.0));
    let total_height = image_width * 1.5 + caption_height;
    let left = (rect.left() + width / 2.0 - image_width / 2.0)
        .max(viewport_left + 12.0)
        .min((viewport_left + viewport_width - image_width - 12.0).max(viewport_left + 12.0));
    let top = (rect.bottom() - total_height)
        .max(viewport_top + 12.0)
        .min((viewport_top + viewport_height - total_height - 12.0).max(viewport_top + 12.0));
    let style = format!(
        "width:{image_width}px;height:{}px;left:{left}px;top:{top}px",
        image_width * 1.5 + caption_height
    );
    let _ = preview.set_attribute("style", &style);
    let _ = picture.set_attribute("style", &format!("height:{}px", image_width * 1.5));
    let _ = app.append_child(&preview);
}

#[derive(Default)]
struct PreviewGesture {
    active: Option<(i32, f64, f64, String)>,
    pending_click: Option<(String, bool)>,
}

impl PreviewGesture {
    fn pointer_down(&mut self, pointer_id: i32, x: f64, y: f64, card_name: Option<String>) {
        self.pending_click = None;
        self.active = card_name.map(|name| (pointer_id, x, y, name));
    }

    fn pointer_move(&mut self, pointer_id: i32, x: f64, y: f64) {
        if let Some((active_id, start_x, start_y, _)) = &self.active {
            let dx = x - start_x;
            let dy = y - start_y;
            if pointer_id == *active_id && (dx * dx + dy * dy).sqrt() > 10.0 {
                self.active = None;
            }
        }
    }

    fn pointer_up(
        &mut self,
        pointer_id: i32,
        card_name: Option<&str>,
        pinned_card_name: Option<&str>,
    ) -> Option<(String, bool)> {
        let (active_id, _, _, original_name) = self.active.take()?;
        if pointer_id != active_id || card_name != Some(original_name.as_str()) {
            return None;
        }
        let show_preview = pinned_card_name != Some(original_name.as_str());
        self.pending_click = Some((original_name.clone(), show_preview));
        Some((original_name, show_preview))
    }

    fn take_click(&mut self, card_name: &str) -> Option<bool> {
        self.pending_click
            .take()
            .and_then(|(pending, suppress)| (pending == card_name).then_some(suppress))
    }

    fn cancel(&mut self) {
        self.active = None;
        self.pending_click = None;
    }
}

fn should_suppress_touch_click(gesture: &mut PreviewGesture, card_name: &str) -> bool {
    gesture.take_click(card_name).unwrap_or(false)
}

fn should_dismiss_hover_preview(is_mouse: bool, leaving_card: bool, pinned: bool) -> bool {
    is_mouse && leaving_card && !pinned
}

fn install_card_preview_handlers(app: &Element) {
    if app.get_attribute("data-preview-handlers").is_some() {
        return;
    }
    let _ = app.set_attribute("data-preview-handlers", "true");

    let hover_app = app.clone();
    let hover = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
        let Some(pointer) = event.dyn_ref::<web_sys::PointerEvent>() else {
            return;
        };
        if pointer.pointer_type() != "mouse" {
            return;
        }
        let Some(card) = event
            .target()
            .and_then(|target| closest_preview_card(&target, &hover_app))
        else {
            return;
        };
        if let Some(related) = pointer
            .related_target()
            .and_then(|target| closest_preview_card(&target, &hover_app))
        {
            if related == card {
                return;
            }
        }
        show_card_preview(&hover_app, &card, false);
    }));
    let _ = app.add_event_listener_with_callback("pointerover", hover.as_ref().unchecked_ref());
    let _ = hover.into_js_value();

    let hover_out_app = app.clone();
    let hover_out = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
        let Some(pointer) = event.dyn_ref::<web_sys::PointerEvent>() else {
            return;
        };
        if pointer.pointer_type() != "mouse" {
            return;
        }
        let Some(card) = event
            .target()
            .and_then(|target| closest_preview_card(&target, &hover_out_app))
        else {
            return;
        };
        let moved_to_same_card = pointer
            .related_target()
            .and_then(|target| closest_preview_card(&target, &hover_out_app))
            .map_or(false, |related| related == card);
        let pinned = hover_out_app
            .query_selector(".card-zoom[data-pinned='true']")
            .ok()
            .flatten()
            .is_some();
        if should_dismiss_hover_preview(true, !moved_to_same_card, pinned) {
            dismiss_card_preview(&hover_out_app);
        }
    }));
    let _ = app.add_event_listener_with_callback("pointerout", hover_out.as_ref().unchecked_ref());
    let _ = hover_out.into_js_value();

    let focus_app = app.clone();
    let focus = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
        if let Some(card) = event
            .target()
            .and_then(|target| closest_preview_card(&target, &focus_app))
        {
            show_card_preview(&focus_app, &card, false);
        }
    }));
    let _ = app.add_event_listener_with_callback("focusin", focus.as_ref().unchecked_ref());
    let _ = focus.into_js_value();

    let blur_app = app.clone();
    let blur = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
        if event
            .target()
            .and_then(|target| closest_preview_card(&target, &blur_app))
            .is_some()
        {
            dismiss_card_preview(&blur_app);
        }
    }));
    let _ = app.add_event_listener_with_callback("focusout", blur.as_ref().unchecked_ref());
    let _ = blur.into_js_value();

    if let Some(document) = app.owner_document() {
        let event_app = app.clone();
        let gesture = Rc::new(RefCell::new(PreviewGesture::default()));

        let down_gesture = Rc::clone(&gesture);
        let down_app = event_app.clone();
        let down = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
            let Some(pointer) = event.dyn_ref::<web_sys::PointerEvent>() else {
                return;
            };
            if pointer.is_primary() && pointer.pointer_type() != "mouse" {
                let card_name = event
                    .target()
                    .and_then(|target| closest_preview_card(&target, &down_app))
                    .and_then(|card| card.get_attribute("data-card-name"));
                down_gesture.borrow_mut().pointer_down(
                    pointer.pointer_id(),
                    pointer.client_x() as f64,
                    pointer.client_y() as f64,
                    card_name,
                );
            } else {
                down_gesture.borrow_mut().cancel();
            }
        }));
        let _ = document.add_event_listener_with_callback_and_bool(
            "pointerdown",
            down.as_ref().unchecked_ref(),
            true,
        );
        let _ = down.into_js_value();

        let move_gesture = Rc::clone(&gesture);
        let moved = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
            let Some(pointer) = event.dyn_ref::<web_sys::PointerEvent>() else {
                return;
            };
            move_gesture.borrow_mut().pointer_move(
                pointer.pointer_id(),
                pointer.client_x() as f64,
                pointer.client_y() as f64,
            );
        }));
        let _ = document.add_event_listener_with_callback_and_bool(
            "pointermove",
            moved.as_ref().unchecked_ref(),
            true,
        );
        let _ = moved.into_js_value();

        let cancel_gesture = Rc::clone(&gesture);
        let cancel = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            cancel_gesture.borrow_mut().cancel();
        }));
        let _ = document.add_event_listener_with_callback_and_bool(
            "pointercancel",
            cancel.as_ref().unchecked_ref(),
            true,
        );
        let _ = cancel.into_js_value();

        let up_gesture = Rc::clone(&gesture);
        let up_app = event_app.clone();
        let up = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
            let Some(pointer) = event.dyn_ref::<web_sys::PointerEvent>() else {
                return;
            };
            if pointer.pointer_type() == "mouse" {
                return;
            }
            let card = event
                .target()
                .and_then(|target| closest_preview_card(&target, &up_app));
            let tapped_name = card
                .as_ref()
                .and_then(|card| card.get_attribute("data-card-name"));
            let pinned_name = up_app
                .query_selector(".card-zoom[data-pinned='true']")
                .ok()
                .flatten()
                .and_then(|preview| preview.query_selector(".card-zoom-name").ok().flatten())
                .and_then(|name| name.text_content());
            let Some((_, show_preview)) = up_gesture.borrow_mut().pointer_up(
                pointer.pointer_id(),
                tapped_name.as_deref(),
                pinned_name.as_deref(),
            ) else {
                return;
            };
            if let Some(card) = card {
                if show_preview {
                    show_card_preview(&up_app, &card, true);
                } else {
                    // Second tap: leave the generated click for the Rust play handler.
                    dismiss_card_preview(&up_app);
                }
            }
        }));
        let _ = document.add_event_listener_with_callback_and_bool(
            "pointerup",
            up.as_ref().unchecked_ref(),
            true,
        );
        let _ = up.into_js_value();

        let click_gesture = Rc::clone(&gesture);
        let click_app = event_app.clone();
        let click = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
            let Some(click_event) = event.dyn_ref::<web_sys::MouseEvent>() else {
                return;
            };
            if click_event.detail() == 0 {
                return;
            }
            let Some(card) = event
                .target()
                .and_then(|target| closest_preview_card(&target, &click_app))
            else {
                dismiss_card_preview(&click_app);
                return;
            };
            let Some(tapped_name) = card.get_attribute("data-card-name") else {
                return;
            };
            if should_suppress_touch_click(&mut click_gesture.borrow_mut(), &tapped_name) {
                event.prevent_default();
                event.stop_immediate_propagation();
            }
        }));
        let _ = document.add_event_listener_with_callback_and_bool(
            "click",
            click.as_ref().unchecked_ref(),
            true,
        );
        let _ = click.into_js_value();

        let outside_app = event_app.clone();
        let outside = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
            if event.dyn_ref::<web_sys::PointerEvent>().is_none() {
                return;
            }
            if event
                .target()
                .and_then(|target| closest_preview_card(&target, &outside_app))
                .is_none()
            {
                dismiss_card_preview(&outside_app);
            }
        }));
        let _ = document.add_event_listener_with_callback_and_bool(
            "pointerdown",
            outside.as_ref().unchecked_ref(),
            true,
        );
        let _ = outside.into_js_value();

        let escape_app = event_app;
        let key = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
            if event
                .dyn_ref::<web_sys::KeyboardEvent>()
                .map_or(false, |key| key.key() == "Escape")
            {
                dismiss_card_preview(&escape_app);
            }
        }));
        let _ = document.add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref());
        let _ = key.into_js_value();

        let scroll_app = app.clone();
        let scroll_gesture = Rc::clone(&gesture);
        let scroll = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            dismiss_card_preview(&scroll_app);
            scroll_gesture.borrow_mut().cancel();
        }));
        let _ = document.add_event_listener_with_callback_and_bool(
            "scroll",
            scroll.as_ref().unchecked_ref(),
            true,
        );
        let _ = scroll.into_js_value();
    }

    if let Some(window) = window() {
        let resize_app = app.clone();
        let resize = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            dismiss_card_preview(&resize_app);
        }));
        let _ = window.add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref());
        let _ = resize.into_js_value();

        if let Some(viewport) = window.visual_viewport() {
            let viewport_app = app.clone();
            let viewport_change = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                dismiss_card_preview(&viewport_app);
            }));
            let _ = viewport.add_event_listener_with_callback(
                "resize",
                viewport_change.as_ref().unchecked_ref(),
            );
            let _ = viewport.add_event_listener_with_callback(
                "scroll",
                viewport_change.as_ref().unchecked_ref(),
            );
            let _ = viewport_change.into_js_value();
        }
    }
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
    let name = card_name(card);
    card_shell
        .set_attribute("data-card-name", &name)
        .expect("card name should be set");
    card_shell
        .set_attribute("tabindex", "0")
        .expect("visible card should be focusable");
    card_shell
        .set_attribute("aria-label", &format!("Enlarge {}", name))
        .expect("visible card should have a preview label");
    card_shell
        .set_attribute("aria-describedby", "card-zoom-help")
        .expect("visible card should describe preview controls");
    card_shell
        .set_attribute("data-preview-card", "true")
        .expect("visible card should be marked for preview");
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
        .set_attribute("aria-label", &format!("Play {}", card_name(card)))
        .expect("button label should be set");
    button
        .set_attribute("data-card-name", &card_name(card))
        .expect("card name should be set");
    button
        .set_attribute("aria-describedby", "card-zoom-help")
        .expect("playable card should describe preview controls");
    button
        .set_attribute("data-preview-card", "true")
        .expect("playable card should be marked for preview");
    button.set_inner_html(card_svg(card));

    let callback_state = Rc::clone(&state);
    let callback = Closure::<dyn FnMut()>::wrap(Box::new(move || {
        select_player1_card(Rc::clone(&callback_state), selected);
    }) as Box<dyn FnMut()>);

    button
        .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
        .expect("card click handler should be attached");
    // Let JavaScript reclaim the callback when its DOM node is removed.
    let _ = callback.into_js_value();
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
    // Let JavaScript reclaim the callback when its DOM node is removed.
    let _ = callback.into_js_value();
    button
}

fn card_name(card: card::Card) -> String {
    let number = match card.value {
        card::CardNumber::Ace => "Ace".to_string(),
        card::CardNumber::Two => "2".to_string(),
        card::CardNumber::Three => "3".to_string(),
        card::CardNumber::Four => "4".to_string(),
        card::CardNumber::Five => "5".to_string(),
        card::CardNumber::Six => "6".to_string(),
        card::CardNumber::Seven => "7".to_string(),
        card::CardNumber::Knave => "Knave".to_string(),
        card::CardNumber::Knight => "Knight".to_string(),
        card::CardNumber::King => "King".to_string(),
    };
    format!("{} of {}", number, trump_suit_name(card.suit))
}

fn trump_suit_name(suit: card::CardSuit) -> &'static str {
    match suit {
        card::CardSuit::Cups => "Cups",
        card::CardSuit::Batons => "Batons",
        card::CardSuit::Coins => "Denari",
        card::CardSuit::Swords => "Swords",
    }
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
        }))
        .expect("restart button should be appended");

    let fill_state = Rc::clone(&state);
    let bigger_view =
        render_action_button(doc, "BIGGER VIEW", state.borrow().fill_screen, move || {
            {
                let mut game = fill_state.borrow_mut();
                game.toggle_fill_screen();
            }
            render_dashboard(&document(), Rc::clone(&fill_state));
        });
    bigger_view
        .set_attribute(
            "class",
            if state.borrow().fill_screen {
                "terminal-button active bigger-view-toggle"
            } else {
                "terminal-button bigger-view-toggle"
            },
        )
        .expect("bigger view class should be set");
    controls
        .append_child(&bigger_view)
        .expect("fill button should be appended");

    controls
}

fn build_difficulty_selector(
    doc: &Document,
    parent: &Element,
    state: Rc<RefCell<BrowserGame>>,
    player_index: usize,
    label: &str,
) {
    append_text(doc, parent, "p", "setup-section-label", label);
    let selector = create_element(doc, "div", "setup-difficulty-options");
    for (button_label, difficulty) in [
        ("RANDOM", ai::AiDifficulty::Random),
        ("CHALLENGER", ai::AiDifficulty::Challenger),
    ] {
        let button_state = Rc::clone(&state);
        let active = state.borrow().ai_difficulty_for(player_index) == difficulty;
        let button = render_action_button(doc, button_label, active, move || {
            button_state
                .borrow_mut()
                .set_player_difficulty(player_index, difficulty);
            render_dashboard(&document(), Rc::clone(&button_state));
        });
        button
            .set_attribute(
                "data-setup-control",
                &format!("difficulty-{player_index}-{}", button_label.to_lowercase()),
            )
            .unwrap();
        selector
            .append_child(&button)
            .expect("difficulty option should be appended");
        if let Some(button) = selector.last_element_child() {
            button
                .set_attribute("aria-pressed", if active { "true" } else { "false" })
                .unwrap();
        }
    }
    parent
        .append_child(&selector)
        .expect("difficulty selector should be appended");
}

fn build_setup_modal(doc: &Document, state: Rc<RefCell<BrowserGame>>) -> Element {
    let layer = create_element(doc, "section", "game-setup-layer");
    let modal = create_element(doc, "div", "game-setup-modal");
    modal.set_attribute("role", "dialog").unwrap();
    modal.set_attribute("aria-modal", "true").unwrap();
    modal
        .set_attribute("aria-labelledby", "game-setup-title")
        .unwrap();
    let title = create_element(doc, "h2", "");
    title.set_attribute("id", "game-setup-title").unwrap();
    title.set_text_content(Some("How would you like to play?"));
    modal.append_child(&title).unwrap();

    let mode_options = create_element(doc, "div", "setup-mode-options");
    for (label, mode) in [
        ("YOU VS AI", GameMode::HumanVsAi),
        ("AI VS AI", GameMode::AiVsAi),
    ] {
        let mode_state = Rc::clone(&state);
        let active = state.borrow().mode == mode;
        let button = render_action_button(doc, label, active, move || {
            mode_state.borrow_mut().mode = mode;
            render_dashboard(&document(), Rc::clone(&mode_state));
        });
        button
            .set_attribute(
                "data-setup-control",
                &format!("mode-{}", label.to_lowercase().replace(' ', "-")),
            )
            .unwrap();
        mode_options
            .append_child(&button)
            .expect("game mode option should be appended");
        if let Some(button) = mode_options.last_element_child() {
            button
                .set_attribute("aria-pressed", if active { "true" } else { "false" })
                .unwrap();
        }
    }
    modal.append_child(&mode_options).unwrap();

    match state.borrow().mode {
        GameMode::HumanVsAi => {
            build_difficulty_selector(doc, &modal, Rc::clone(&state), 2, "AI DIFFICULTY")
        }
        GameMode::AiVsAi => {
            build_difficulty_selector(doc, &modal, Rc::clone(&state), 1, "AI 1 DIFFICULTY");
            build_difficulty_selector(doc, &modal, Rc::clone(&state), 2, "AI 2 DIFFICULTY");
        }
    }

    let start_state = Rc::clone(&state);
    let start_button = render_action_button(doc, "START GAME", true, move || {
        start_state.borrow_mut().start_game();
        render_dashboard(&document(), Rc::clone(&start_state));
        let tick_generation = start_state.borrow().tick_generation();
        schedule_next_tick(
            Rc::clone(&start_state),
            COIN_TOSS_DISPLAY_MS,
            tick_generation,
        );
    });
    start_button
        .set_attribute("data-setup-control", "start")
        .unwrap();
    modal
        .append_child(&start_button)
        .expect("start button should be appended");

    layer.append_child(&modal).unwrap();
    let trap_modal = modal.clone();
    let trap = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
        let Some(key_event) = event.dyn_ref::<web_sys::KeyboardEvent>() else {
            return;
        };
        if key_event.key() != "Tab" {
            return;
        }
        let Ok(buttons) = trap_modal.query_selector_all("button") else {
            return;
        };
        if buttons.length() == 0 {
            return;
        }
        let first = buttons
            .item(0)
            .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok());
        let last = buttons
            .item(buttons.length() - 1)
            .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok());
        let active = trap_modal
            .owner_document()
            .and_then(|doc| doc.active_element());
        if key_event.shift_key()
            && active
                .as_ref()
                .map(|el| el.is_same_node(first.as_ref().map(|e| e.as_ref())))
                .unwrap_or(false)
        {
            key_event.prevent_default();
            if let Some(last) = last {
                let _ = last.focus();
            }
        } else if !key_event.shift_key()
            && active
                .as_ref()
                .map(|el| el.is_same_node(last.as_ref().map(|e| e.as_ref())))
                .unwrap_or(false)
        {
            key_event.prevent_default();
            if let Some(first) = first {
                let _ = first.focus();
            }
        }
    }));
    let _ = modal.add_event_listener_with_callback("keydown", trap.as_ref().unchecked_ref());
    let _ = trap.into_js_value();
    layer
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

fn build_trick_panel(
    document: &Document,
    trick_cards: &[(usize, card::Card)],
    last_played_player: Option<usize>,
    mode: GameMode,
) -> Element {
    let panel = create_element(document, "section", "stack-panel trick-panel");
    append_text(document, &panel, "p", "stack-label", "TRICK");

    let slots = create_element(document, "div", "trick-slots");
    for player_index in [2usize, 1usize] {
        let slot = create_element(document, "div", "trick-slot");
        let played_card = trick_cards
            .iter()
            .find(|(owner, _)| *owner == player_index)
            .map(|(_, card)| *card);
        let card_node = played_card
            .map(|card| render_card_svg(document, card))
            .unwrap_or_else(|| render_placeholder_card(document));
        if played_card.is_some() && last_played_player == Some(player_index) {
            let played_class = if player_index == 1 {
                "played-card-from-you"
            } else {
                "played-card-from-challenger"
            };
            card_node.set_class_name(&format!("{} {}", card_node.class_name(), played_class));
        }

        slot.append_child(&card_node)
            .expect("trick card should be appended");
        append_text(
            document,
            &slot,
            "p",
            "trick-slot-label",
            BrowserGame::player_display_label_for(mode, player_index),
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

fn build_result(document: &Document, game: &BrowserGame) -> Element {
    let layer = create_element(document, "div", "victory-layer");
    if game.game_winner().is_some() {
        for burst in 0..3 {
            let firework = create_element(document, "div", "firework-burst");
            firework.set_attribute("aria-hidden", "true").unwrap();
            for spark in 0..12 {
                let particle = create_element(document, "span", "");
                particle
                    .set_attribute(
                        "style",
                        &format!(
                            "--angle: {}deg; --distance: {}px; animation-delay: {}ms",
                            spark * 30,
                            34 + burst * 8,
                            burst * 160,
                        ),
                    )
                    .unwrap();
                firework.append_child(&particle).unwrap();
            }
            layer.append_child(&firework).unwrap();
        }
    }
    let result = create_element(document, "div", "victory-result");
    result.set_attribute("role", "status").unwrap();
    append_text(document, &result, "h2", "", game.result_label());
    let (you, challenger) = game.scores();
    append_text(
        document,
        &result,
        "p",
        "",
        &format!(
            "{} {} — {} {}",
            game.player_display_label(1),
            you,
            game.player_display_label(2),
            challenger
        ),
    );
    layer.append_child(&result).unwrap();
    layer
}

fn build_coin_toss(document: &Document, game: &BrowserGame) -> Element {
    let layer = create_element(document, "section", "coin-toss-layer");
    layer
        .set_attribute("role", "status")
        .expect("coin toss status role should be set");
    layer
        .set_attribute("aria-live", "polite")
        .expect("coin toss live region should be set");

    let modal = create_element(document, "div", "coin-toss-modal");
    append_text(document, &modal, "h2", "", "COIN TOSS");
    let coin = create_element(document, "div", "coin-toss-coin");
    let card = create_element(
        document,
        "div",
        if game.leader_index() == 1 {
            "coin-toss-card player-opens"
        } else {
            "coin-toss-card ai-opens"
        },
    );
    let denari_face = create_element(document, "div", "coin-toss-face front");
    denari_face
        .append_child(&render_card_svg(
            document,
            card::Card::new(card::CardNumber::Ace, card::CardSuit::Coins),
        ))
        .expect("ace of denari should be appended to coin");
    let back_face = create_element(document, "div", "coin-toss-face back");
    back_face
        .append_child(&render_card_back(document))
        .expect("card back should be appended to coin");
    card.append_child(&denari_face)
        .expect("denari face should be appended");
    card.append_child(&back_face)
        .expect("card back face should be appended");
    coin.append_child(&card)
        .expect("flipping card should be appended");
    modal
        .append_child(&coin)
        .expect("coin toss card should be appended");
    append_text(
        document,
        &modal,
        "p",
        "",
        &format!(
            "{} plays first",
            game.player_display_label(game.leader_index())
        ),
    );
    layer
        .append_child(&modal)
        .expect("coin toss modal should be appended");
    layer
}

fn build_trump_reveal(document: &Document, game: &BrowserGame) -> Element {
    let layer = create_element(document, "section", "trump-reveal-layer");
    layer.set_attribute("role", "status").unwrap();
    layer.set_attribute("aria-live", "polite").unwrap();

    let modal = create_element(document, "div", "trump-reveal-modal");
    append_text(document, &modal, "h2", "", "TRUMP REVEALED");
    modal
        .append_child(&render_card_svg(document, game.briscola))
        .expect("trump card should be appended to reveal");
    append_text(
        document,
        &modal,
        "p",
        "",
        &format!("The trump is {}", trump_suit_name(game.briscola.suit)),
    );
    layer
        .append_child(&modal)
        .expect("trump reveal modal should be appended");
    layer
}

fn render_dashboard(document: &Document, state: Rc<RefCell<BrowserGame>>) {
    set_styles(document);
    let mount = mount_element(document);
    let focused_setup_control = document
        .active_element()
        .and_then(|element| element.get_attribute("data-setup-control"));
    let was_setup = mount
        .query_selector(".game-setup-modal")
        .ok()
        .flatten()
        .is_some();
    dismiss_card_preview(&mount);
    install_card_preview_handlers(&mount);
    mount.set_inner_html("");

    let state_ref = state.borrow();
    mount.set_class_name(&app_class_name(document, state_ref.fill_screen));
    let (score1, score2) = state_ref.scores();

    let dashboard = create_element(document, "main", "dashboard");

    let header = create_element(document, "header", "table-header");
    let title_wrap = create_element(document, "div", "");
    let title = create_element(document, "h1", "table-title");
    title.set_text_content(Some("Briscola "));
    let github_reference = create_element(document, "a", "github-reference");
    github_reference.set_text_content(Some("GitHub repo"));
    github_reference
        .set_attribute("href", "https://github.com/michael-zucchetta/briscola")
        .expect("GitHub link should have a destination");
    github_reference
        .set_attribute("target", "_blank")
        .expect("GitHub link should open in a new tab");
    github_reference
        .set_attribute("rel", "noopener noreferrer")
        .expect("GitHub link should have safe external-link attributes");
    title
        .append_child(&github_reference)
        .expect("GitHub link should be appended to the title");
    title_wrap
        .append_child(&title)
        .expect("title should be appended");
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
        "header-pill trick-status",
        &state_ref.status,
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
            state_ref.player_display_label(2),
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
        .append_child(&build_trick_panel(
            document,
            &state_ref.trick_cards,
            state_ref.last_played_player,
            state_ref.mode,
        ))
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
            state_ref.player_display_label(1),
            1,
            &state_ref.player1_hand,
            true,
            state_ref.waiting_for_player1(),
            state_ref.last_dealt_player == Some(1),
            Rc::clone(&state),
        ))
        .expect("player 1 zone should be appended");

    if matches!(state_ref.phase, Phase::Setup) {
        board
            .append_child(&build_setup_modal(document, Rc::clone(&state)))
            .expect("game setup modal should be appended");
    } else if matches!(state_ref.phase, Phase::CoinToss) {
        board
            .append_child(&build_coin_toss(document, &state_ref))
            .expect("coin toss should be appended");
    } else if matches!(state_ref.phase, Phase::TrumpReveal) {
        board
            .append_child(&build_trump_reveal(document, &state_ref))
            .expect("trump reveal should be appended");
    } else if matches!(state_ref.phase, Phase::Finished) {
        board
            .append_child(&build_result(document, &state_ref))
            .unwrap();
    }

    let footer = create_element(document, "footer", "footer-bar");
    append_text(
        document,
        &footer,
        "div",
        "footer-pill won-count",
        &format!(
            "LEADER: {}",
            state_ref.player_display_label(state_ref.leader_index())
        ),
    );
    append_text(
        document,
        &footer,
        "div",
        "footer-pill",
        &format!("{}: {} PTS", state_ref.player_display_label(1), score1),
    );
    append_text(
        document,
        &footer,
        "div",
        "footer-pill",
        &format!("{}: {} PTS", state_ref.player_display_label(2), score2),
    );
    append_text(
        document,
        &footer,
        "div",
        "footer-pill",
        &format!(
            "WON: {} {} / {} {}",
            state_ref.player_display_label(1),
            state_ref.player1_won.len(),
            state_ref.player_display_label(2),
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

    if matches!(state_ref.phase, Phase::Setup) {
        let focus_target = focused_setup_control
            .as_deref()
            .and_then(|control| {
                dashboard
                    .query_selector(&format!("[data-setup-control='{control}']"))
                    .ok()
                    .flatten()
            })
            .or_else(|| {
                (!was_setup)
                    .then(|| {
                        dashboard
                            .query_selector(".game-setup-modal button")
                            .ok()
                            .flatten()
                    })
                    .flatten()
            });
        if let Some(button) =
            focus_target.and_then(|button| button.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = button.focus();
        }
    } else if was_setup {
        if let Ok(Some(status)) = dashboard.query_selector(".trick-status") {
            let _ = status.set_attribute("tabindex", "-1");
            if let Some(status) = status.dyn_ref::<web_sys::HtmlElement>() {
                let _ = status.focus();
            }
        }
    }

    drop(state_ref);
    {
        let mut game = state.borrow_mut();
        game.last_dealt_player = None;
        game.last_played_player = None;
    }
}

fn schedule_next_tick(state: Rc<RefCell<BrowserGame>>, delay_ms: i32, tick_generation: u32) {
    let callback_state = Rc::clone(&state);
    let callback = Closure::once_into_js(move || {
        tick(Rc::clone(&callback_state), tick_generation);
    });

    window()
        .expect("window should exist")
        .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), delay_ms)
        .expect("timeout should be scheduled");
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
    let mut game = BrowserGame::new();
    // Match the mobile CSS breakpoint on first load; subsequent renders and
    // restarts preserve the user's Bigger View toggle choice.
    game.fill_screen = window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .map_or(false, |width| width <= 720.0);
    let state = Rc::new(RefCell::new(game));
    render_dashboard(&document(), Rc::clone(&state));
}

#[cfg(test)]
mod browser_tests {
    use super::*;

    #[test]
    fn first_touch_tap_previews_and_suppresses_click() {
        let mut gesture = PreviewGesture::default();
        gesture.pointer_down(7, 40.0, 80.0, Some("Ace of Cups".to_string()));

        let (card_name, show_preview) = gesture
            .pointer_up(7, Some("Ace of Cups"), None)
            .expect("matching pointer-up should complete the tap");
        assert!(show_preview);
        assert_eq!(card_name, "Ace of Cups");
        // Suppress the first-tap click even if another event dismissed the preview.
        assert!(should_suppress_touch_click(&mut gesture, "Ace of Cups"));
    }

    #[test]
    fn second_touch_tap_plays_and_does_not_suppress_click() {
        let mut gesture = PreviewGesture::default();
        let mut pinned_card = None;
        gesture.pointer_down(3, 10.0, 20.0, Some("Three of Swords".to_string()));
        let (card_name, show_preview) = gesture
            .pointer_up(3, Some("Three of Swords"), pinned_card.as_deref())
            .expect("first matching tap should complete");
        assert!(show_preview);
        pinned_card = Some(card_name);
        assert!(should_suppress_touch_click(&mut gesture, "Three of Swords"));

        gesture.pointer_down(4, 10.0, 20.0, Some("Three of Swords".to_string()));
        let (card_name, show_preview) = gesture
            .pointer_up(4, Some("Three of Swords"), pinned_card.as_deref())
            .expect("second matching tap should complete");
        assert!(!show_preview);
        assert_eq!(card_name, "Three of Swords");
        assert!(!should_suppress_touch_click(
            &mut gesture,
            "Three of Swords"
        ));
    }

    #[test]
    fn touch_drag_cancels_preview_tap() {
        let mut gesture = PreviewGesture::default();
        gesture.pointer_down(1, 20.0, 30.0, Some("King of Coins".to_string()));
        gesture.pointer_move(1, 20.0, 42.0);

        assert_eq!(gesture.pointer_up(1, Some("King of Coins"), None), None);
        assert_eq!(gesture.take_click("King of Coins"), None);
    }

    #[test]
    fn touch_release_on_different_card_does_not_preview_or_suppress() {
        let mut gesture = PreviewGesture::default();
        gesture.pointer_down(2, 20.0, 30.0, Some("King of Coins".to_string()));

        assert_eq!(gesture.pointer_up(2, Some("Four of Cups"), None), None);
        assert_eq!(gesture.take_click("Four of Cups"), None);
    }

    #[test]
    fn touch_release_outside_a_card_clears_the_active_gesture() {
        let mut gesture = PreviewGesture::default();
        gesture.pointer_down(9, 20.0, 30.0, Some("King of Coins".to_string()));

        assert_eq!(gesture.pointer_up(9, None, None), None);
        assert_eq!(gesture.pointer_up(9, Some("King of Coins"), None), None);
        assert_eq!(gesture.take_click("King of Coins"), None);
    }

    #[test]
    fn pointer_cancel_clears_active_and_pending_taps() {
        let mut gesture = PreviewGesture::default();
        gesture.pointer_down(6, 12.0, 18.0, Some("Seven of Swords".to_string()));
        gesture.cancel();
        assert_eq!(gesture.pointer_up(6, Some("Seven of Swords"), None), None);

        gesture.pointer_down(8, 12.0, 18.0, Some("Seven of Swords".to_string()));
        assert!(gesture
            .pointer_up(8, Some("Seven of Swords"), None)
            .is_some());
        gesture.cancel();
        assert!(!should_suppress_touch_click(
            &mut gesture,
            "Seven of Swords"
        ));
    }

    #[test]
    fn pending_tap_does_not_suppress_a_different_card_click() {
        let mut gesture = PreviewGesture::default();
        gesture.pointer_down(5, 12.0, 18.0, Some("Seven of Swords".to_string()));
        assert!(gesture
            .pointer_up(5, Some("Seven of Swords"), None)
            .is_some());

        assert!(!should_suppress_touch_click(&mut gesture, "Four of Cups"));
        assert!(!should_suppress_touch_click(
            &mut gesture,
            "Seven of Swords"
        ));
    }

    #[test]
    fn hover_preview_dismisses_on_exit_but_survives_internal_and_pinned_transitions() {
        assert!(should_dismiss_hover_preview(true, true, false));
        assert!(!should_dismiss_hover_preview(true, false, false));
        assert!(!should_dismiss_hover_preview(true, true, true));
        assert!(!should_dismiss_hover_preview(false, true, false));
    }

    #[test]
    fn complete_games_conserve_cards_and_points() {
        for mode in [GameMode::HumanVsAi, GameMode::AiVsAi] {
            for (player1_difficulty, player2_difficulty) in [
                (ai::AiDifficulty::Random, ai::AiDifficulty::Challenger),
                (ai::AiDifficulty::Challenger, ai::AiDifficulty::Random),
            ] {
                for _ in 0..10 {
                    let mut game = BrowserGame::new();
                    game.mode = mode;
                    game.player1_difficulty = player1_difficulty;
                    game.player2_difficulty = player2_difficulty;
                    game.start_game();
                    let mut steps = 0;
                    while !matches!(game.phase, Phase::Finished) {
                        let cards = game.player1_hand.len()
                            + game.player2_hand.len()
                            + game.player1_won.len()
                            + game.player2_won.len()
                            + game.trick_cards.len()
                            + game.deck_size;
                        assert_eq!(cards, 40);
                        assert!(game.player1_hand.len() <= 3 && game.player2_hand.len() <= 3);
                        if game.waiting_for_player1() {
                            assert!(game.play_player1_card(0).is_some());
                        } else {
                            game.advance();
                        }
                        steps += 1;
                        assert!(steps < 250, "game must terminate");
                    }
                    assert_eq!(game.trick_number, 21);
                    assert_eq!(game.scores().0 + game.scores().1, 120);
                    let mut won = game.player1_won.clone();
                    won.extend(&game.player2_won);
                    for card in card::Card::load_cards() {
                        assert_eq!(won.iter().filter(|&&c| c == card).count(), 1);
                    }
                    assert!(game.advance().is_none());
                    assert!(game.play_player1_card(0).is_none());
                }
            }
        }
    }

    #[test]
    fn setup_waits_for_start_and_restart_invalidates_pending_ticks() {
        let mut game = BrowserGame::new();
        assert!(matches!(game.phase, Phase::Setup));
        assert!(game.advance().is_none());
        assert_eq!(game.tick_generation(), 0);

        game.mode = GameMode::AiVsAi;
        game.player1_difficulty = ai::AiDifficulty::Random;
        game.player2_difficulty = ai::AiDifficulty::Challenger;
        game.start_game();
        assert!(matches!(game.phase, Phase::CoinToss));

        game.restart();
        assert!(matches!(game.phase, Phase::Setup));
        assert_eq!(game.tick_generation(), 1);
        assert_eq!(game.mode, GameMode::AiVsAi);
        assert_eq!(game.player1_difficulty, ai::AiDifficulty::Random);
        assert_eq!(game.player2_difficulty, ai::AiDifficulty::Challenger);
        assert!(game.advance().is_none());
    }

    #[test]
    fn coin_toss_has_time_to_finish_before_trump_reveal() {
        assert_eq!(COIN_TOSS_DISPLAY_MS, 2200);

        let mut game = BrowserGame::new();
        game.start_game();
        assert!(matches!(game.phase, Phase::CoinToss));

        assert_eq!(game.advance(), Some(3200));
        assert!(matches!(game.phase, Phase::TrumpReveal));
        assert_eq!(game.advance(), Some(320));
        assert!(matches!(game.phase, Phase::Dealing));
    }

    #[test]
    fn ai_mode_uses_independent_difficulty_and_disables_human_input() {
        let mut game = BrowserGame::new();
        game.mode = GameMode::AiVsAi;
        game.set_player_difficulty(1, ai::AiDifficulty::Random);
        game.set_player_difficulty(2, ai::AiDifficulty::Challenger);

        assert_eq!(game.ai_difficulty_for(1), ai::AiDifficulty::Random);
        assert_eq!(game.ai_difficulty_for(2), ai::AiDifficulty::Challenger);
        assert!(!game.is_human_player(1));
        assert!(!game.is_human_player(2));
        assert!(!game.waiting_for_player1());

        game.mode = GameMode::HumanVsAi;
        assert!(game.is_human_player(1));
        assert!(!game.is_human_player(2));
    }

    #[test]
    fn tied_result_and_restart() {
        let mut game = BrowserGame::new();
        let cards = card::Card::load_cards();
        game.player1_won = cards[..20].to_vec();
        game.player2_won = cards[20..].to_vec();
        game.phase = Phase::Finished;
        assert_eq!(game.scores(), (60, 60));
        assert_eq!(game.game_winner(), None);
        assert_eq!(game.result_label(), "Draw");
        game.mode = GameMode::AiVsAi;
        game.player1_difficulty = ai::AiDifficulty::Random;
        game.player2_difficulty = ai::AiDifficulty::Challenger;
        game.fill_screen = true;
        game.restart();
        assert_eq!(game.tick_generation(), 1);
        assert!(matches!(game.phase, Phase::Setup));
        assert_eq!(game.scores(), (0, 0));
        assert_eq!(game.mode, GameMode::AiVsAi);
        assert_eq!(game.player1_difficulty, ai::AiDifficulty::Random);
        assert_eq!(game.player2_difficulty, ai::AiDifficulty::Challenger);
        assert!(game.fill_screen);
    }

    #[test]
    fn player_statuses_use_correct_verb_forms() {
        let human_game = BrowserGame::new();
        assert_eq!(human_game.player_leads_label(1), "You lead");
        assert_eq!(human_game.player_leads_label(2), "AI leads");
        assert_eq!(human_game.player_wins_label(1), "You win");
        assert_eq!(human_game.player_wins_label(2), "AI wins");
        assert_eq!(human_game.player_collects_label(1), "You collect");
        assert_eq!(human_game.player_collects_label(2), "AI collects");
        assert_eq!(human_game.player_draws_label(1, "first"), "You draw first.");
        assert_eq!(
            human_game.player_draws_label(2, "second"),
            "AI draws second."
        );

        let mut ai_game = BrowserGame::new();
        ai_game.mode = GameMode::AiVsAi;
        assert_eq!(ai_game.player_display_label(1), "AI 1");
        assert_eq!(ai_game.player_display_label(2), "AI 2");
        assert_eq!(ai_game.player_leads_label(1), "AI 1 leads");
        assert_eq!(ai_game.player_wins_label(2), "AI 2 wins");
        assert_eq!(ai_game.player_collects_label(1), "AI 1 collects");
        assert_eq!(ai_game.player_draws_label(2, "first"), "AI 2 draws first.");
    }
}
