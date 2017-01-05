extern crate game;

use game::new_game;
use game::Game;
use game::Place;
use game::Board;

#[test]
fn new_game_initializes_board() {
    let game = new_game();

    match game {
        Game::Place(place) => {
            assert_eq!(place.board.cells, Board::new().cells);
        }
        _ => panic!("should be a Game::Place")
    }
}
