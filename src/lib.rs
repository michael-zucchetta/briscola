use wasm_bindgen::prelude::*;
use web_sys::{window, Document, Element};

pub mod card;
pub mod hand;
pub mod constants;
pub mod deck;
pub mod game;
pub mod painter;
pub mod player;
pub mod console;

struct TableState {
    player1_hand: Vec<card::Card>,
    player2_hand: Vec<card::Card>,
    briscola: card::Card,
    deck_size: usize,
}

fn document() -> Document {
    window()
        .expect("window should exist")
        .document()
        .expect("document should exist")
}

fn set_styles(document: &Document) {
    let style = document
        .create_element("style")
        .expect("style element should be created");
    style.set_inner_html(
        r#"
        :root {
            --felt-dark: #0f5132;
            --felt-mid: #1f7a4d;
            --felt-light: #53a574;
            --ink: #0d1b12;
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
                radial-gradient(circle at top, rgba(255,255,255,0.12), transparent 30%),
                linear-gradient(180deg, var(--felt-light) 0%, var(--felt-mid) 35%, var(--felt-dark) 100%);
        }

        .dashboard {
            min-height: 100vh;
            padding: 24px;
            display: grid;
            grid-template-rows: auto 1fr auto;
            gap: 24px;
        }

        .table-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
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

        .status-chip {
            padding: 10px 14px;
            border-radius: 999px;
            background: rgba(244, 235, 215, 0.16);
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
        }

        .table-middle {
            position: relative;
            z-index: 1;
            display: grid;
            grid-template-columns: repeat(2, minmax(160px, 220px));
            justify-content: center;
            gap: 24px;
            align-items: center;
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

        .deck-stack {
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

        .card-shell {
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

        .card-shell svg,
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

        .card-back::after {
            content: "";
            width: calc(100% - 18px);
            height: calc(100% - 18px);
            border-radius: 10px;
            border: 1px solid rgba(244, 235, 215, 0.45);
            background: rgba(255,255,255,0.06);
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

            .table-middle {
                grid-template-columns: 1fr;
            }

            .table-header {
                align-items: flex-start;
                flex-direction: column;
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

fn deal_initial_state() -> TableState {
    let deck = deck::Deck::new();
    let briscola = deck.get_briscola();

    let player1_hand = (0..3)
        .map(|_| deck.get_card().expect("player 1 card should exist"))
        .collect::<Vec<card::Card>>();
    let player2_hand = (0..3)
        .map(|_| deck.get_card().expect("player 2 card should exist"))
        .collect::<Vec<card::Card>>();

    TableState {
        player1_hand,
        player2_hand,
        briscola,
        deck_size: 40 - 6,
    }
}

fn render_card_svg(document: &Document, card: card::Card) -> Element {
    let card_shell = create_element(document, "div", "card-shell");
    card_shell.set_inner_html(card_svg(card));
    card_shell
}

fn render_card_back(document: &Document) -> Element {
    let card_back = create_element(document, "div", "card-shell card-back");
    card_back
}

fn build_player_zone(
    document: &Document,
    label: &str,
    cards: &[card::Card],
    face_up: bool,
) -> Element {
    let zone = create_element(document, "section", "player-zone");
    append_text(document, &zone, "p", "player-label", label);

    let hand = create_element(document, "div", "player-hand");
    for &card in cards {
        let card_element = if face_up {
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
    for _ in 0..3 {
        stack.append_child(&render_card_back(document))
            .expect("deck card should be appended");
    }
    panel.append_child(&stack)
        .expect("deck stack should be appended");

    append_text(document, &panel, "p", "stack-count", &deck_size.to_string());
    append_text(document, &panel, "p", "stack-caption", "cards remaining after the deal");
    panel
}

fn build_trump_panel(document: &Document, briscola: card::Card) -> Element {
    let panel = create_element(document, "section", "stack-panel");
    append_text(document, &panel, "p", "stack-label", "Briscola");
    panel
        .append_child(&render_card_svg(document, briscola))
        .expect("briscola card should be appended");
    append_text(document, &panel, "p", "stack-caption", "trump card visible under the deck");
    panel
}

fn render_dashboard(document: &Document, state: TableState) {
    set_styles(document);
    let body = document.body().expect("document body should exist");
    body.set_inner_html("");

    let dashboard = create_element(document, "main", "dashboard");

    let header = create_element(document, "header", "table-header");
    let title_wrap = create_element(document, "div", "");
    append_text(document, &title_wrap, "h1", "table-title", "Briscola Dashboard");
    append_text(
        document,
        &title_wrap,
        "p",
        "table-subtitle",
        "Initial table state rendered with web-sys",
    );
    header
        .append_child(&title_wrap)
        .expect("header title should be appended");
    append_text(document, &header, "div", "status-chip", "Deal Complete");

    let board = create_element(document, "section", "table-board");
    board
        .append_child(&build_player_zone(
            document,
            "Player 2",
            &state.player2_hand,
            false,
        ))
        .expect("player 2 zone should be appended");

    let middle = create_element(document, "section", "table-middle");
    middle
        .append_child(&build_deck_panel(document, state.deck_size))
        .expect("deck panel should be appended");
    middle
        .append_child(&build_trump_panel(document, state.briscola))
        .expect("trump panel should be appended");
    board
        .append_child(&middle)
        .expect("middle section should be appended");

    board
        .append_child(&build_player_zone(
            document,
            "Player 1",
            &state.player1_hand,
            true,
        ))
        .expect("player 1 zone should be appended");

    dashboard
        .append_child(&header)
        .expect("header should be appended");
    dashboard
        .append_child(&board)
        .expect("board should be appended");
    body.append_child(&dashboard)
        .expect("dashboard should be appended");
}

fn card_svg(card: card::Card) -> &'static str {
    match (card.suit, card.value) {
        (card::CardSuit::Batons, card::CardNumber::Ace) => include_str!("../assets/briscola/bresciane/bastoni/asso.svg"),
        (card::CardSuit::Batons, card::CardNumber::Two) => include_str!("../assets/briscola/bresciane/bastoni/02.svg"),
        (card::CardSuit::Batons, card::CardNumber::Three) => include_str!("../assets/briscola/bresciane/bastoni/03.svg"),
        (card::CardSuit::Batons, card::CardNumber::Four) => include_str!("../assets/briscola/bresciane/bastoni/04.svg"),
        (card::CardSuit::Batons, card::CardNumber::Five) => include_str!("../assets/briscola/bresciane/bastoni/05.svg"),
        (card::CardSuit::Batons, card::CardNumber::Six) => include_str!("../assets/briscola/bresciane/bastoni/06.svg"),
        (card::CardSuit::Batons, card::CardNumber::Seven) => include_str!("../assets/briscola/bresciane/bastoni/07.svg"),
        (card::CardSuit::Batons, card::CardNumber::Knave) => include_str!("../assets/briscola/bresciane/bastoni/fante.svg"),
        (card::CardSuit::Batons, card::CardNumber::Knight) => include_str!("../assets/briscola/bresciane/bastoni/cavallo.svg"),
        (card::CardSuit::Batons, card::CardNumber::King) => include_str!("../assets/briscola/bresciane/bastoni/re.svg"),
        (card::CardSuit::Cups, card::CardNumber::Ace) => include_str!("../assets/briscola/bresciane/coppe/asso.svg"),
        (card::CardSuit::Cups, card::CardNumber::Two) => include_str!("../assets/briscola/bresciane/coppe/02.svg"),
        (card::CardSuit::Cups, card::CardNumber::Three) => include_str!("../assets/briscola/bresciane/coppe/03.svg"),
        (card::CardSuit::Cups, card::CardNumber::Four) => include_str!("../assets/briscola/bresciane/coppe/04.svg"),
        (card::CardSuit::Cups, card::CardNumber::Five) => include_str!("../assets/briscola/bresciane/coppe/05.svg"),
        (card::CardSuit::Cups, card::CardNumber::Six) => include_str!("../assets/briscola/bresciane/coppe/06.svg"),
        (card::CardSuit::Cups, card::CardNumber::Seven) => include_str!("../assets/briscola/bresciane/coppe/07.svg"),
        (card::CardSuit::Cups, card::CardNumber::Knave) => include_str!("../assets/briscola/bresciane/coppe/fante.svg"),
        (card::CardSuit::Cups, card::CardNumber::Knight) => include_str!("../assets/briscola/bresciane/coppe/cavallo.svg"),
        (card::CardSuit::Cups, card::CardNumber::King) => include_str!("../assets/briscola/bresciane/coppe/re.svg"),
        (card::CardSuit::Coins, card::CardNumber::Ace) => include_str!("../assets/briscola/bresciane/denari/asso.svg"),
        (card::CardSuit::Coins, card::CardNumber::Two) => include_str!("../assets/briscola/bresciane/denari/02.svg"),
        (card::CardSuit::Coins, card::CardNumber::Three) => include_str!("../assets/briscola/bresciane/denari/03.svg"),
        (card::CardSuit::Coins, card::CardNumber::Four) => include_str!("../assets/briscola/bresciane/denari/04.svg"),
        (card::CardSuit::Coins, card::CardNumber::Five) => include_str!("../assets/briscola/bresciane/denari/05.svg"),
        (card::CardSuit::Coins, card::CardNumber::Six) => include_str!("../assets/briscola/bresciane/denari/06.svg"),
        (card::CardSuit::Coins, card::CardNumber::Seven) => include_str!("../assets/briscola/bresciane/denari/07.svg"),
        (card::CardSuit::Coins, card::CardNumber::Knave) => include_str!("../assets/briscola/bresciane/denari/fante.svg"),
        (card::CardSuit::Coins, card::CardNumber::Knight) => include_str!("../assets/briscola/bresciane/denari/cavallo.svg"),
        (card::CardSuit::Coins, card::CardNumber::King) => include_str!("../assets/briscola/bresciane/denari/re.svg"),
        (card::CardSuit::Swords, card::CardNumber::Ace) => include_str!("../assets/briscola/bresciane/spade/asso.svg"),
        (card::CardSuit::Swords, card::CardNumber::Two) => include_str!("../assets/briscola/bresciane/spade/02.svg"),
        (card::CardSuit::Swords, card::CardNumber::Three) => include_str!("../assets/briscola/bresciane/spade/03.svg"),
        (card::CardSuit::Swords, card::CardNumber::Four) => include_str!("../assets/briscola/bresciane/spade/04.svg"),
        (card::CardSuit::Swords, card::CardNumber::Five) => include_str!("../assets/briscola/bresciane/spade/05.svg"),
        (card::CardSuit::Swords, card::CardNumber::Six) => include_str!("../assets/briscola/bresciane/spade/06.svg"),
        (card::CardSuit::Swords, card::CardNumber::Seven) => include_str!("../assets/briscola/bresciane/spade/07.svg"),
        (card::CardSuit::Swords, card::CardNumber::Knave) => include_str!("../assets/briscola/bresciane/spade/fante.svg"),
        (card::CardSuit::Swords, card::CardNumber::Knight) => include_str!("../assets/briscola/bresciane/spade/cavallo.svg"),
        (card::CardSuit::Swords, card::CardNumber::King) => include_str!("../assets/briscola/bresciane/spade/re.svg"),
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    let document = document();
    let state = deal_initial_state();
    render_dashboard(&document, state);
}
