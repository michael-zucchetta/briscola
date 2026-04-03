use crate::card;

pub trait Painter {
    fn print_card(card: card::Card);

    fn draw_beginning();

    fn update_game();
}
