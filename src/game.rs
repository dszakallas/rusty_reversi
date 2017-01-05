use std::fmt;

/// A `Color` is either black or white.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    /// Color of the black player.
    Black,
    /// Color of the white player.
    White
}
pub trait Flip { fn flip(&self) -> Color; }
impl Flip for Color {
    fn flip(&self) -> Color {
        match *self {
            Color::Black => Color::White,
            Color::White => Color::Black
        }
    }
}
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Color::Black => write!(f, "Black"),
            Color::White => write!(f, "White")
        }
    }
}

/// `Coord` is a position on the board.
pub type Coord = (i8, i8);

/// The constant `DIRECTIONS` enumerates all possible flip direction deltas on the board.
pub const DIRECTIONS: [Coord; 8] = [
    (0, 1), // N
    (1, 1), // NE
    (1, 0), // E
    (1, -1), // SE
    (0, -1), // S
    (-1, -1), // SW
    (-1, 0), // W
    (-1, 1) // NW
];

/// `IllegalMove` lists the reasons why a move is illegal.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IllegalMove {
    /// The cell is already occupied by `Color`
    Occupied(Color),
    /// The move does not cause any disks to change color
    Ineffective
}
impl fmt::Display for IllegalMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IllegalMove::Occupied(color) => write!(f, "Occupied by {}", color),
            IllegalMove::Ineffective => write!(f, "Ineffective")
        }
    }
}

/// `LegalMove` holds the description of a legal move on the board.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LegalMove {
    pub color: Color,
    pub flips: [i8; 8],
    pub position: Coord
}
impl LegalMove {
    /// Applying a legal move returns a changed board.
    fn apply(&self, board: Board) -> Board {
        let &(x, y) = &self.position;
        let mut fresh_board = board;
        // closures cannot be recursive, so I used a regular function
        // rustc cannot do tail calls as of now, see https://github.com/rust-lang/rust/issues/217
        // but as this will always reenter exactly 8 times, no need to worry about overflowing the stack
        fn flip_direction(cells: &mut [[Option<Color>; 8]; 8], position: Coord, direction: Coord, n: i8) {
            match n {
                0 => return,
                n => {
                    let &(x, y) = &position;
                    let &(dx, dy) = &direction;
                    let next = (x + dx, y + dy);
                    cells[next.0 as usize][next.1 as usize] = Some(cells[next.0 as usize][next.1 as usize].unwrap().flip());
                    flip_direction(cells, next, direction, n - 1);
                }
            }
        }
        for (i, &(h, v)) in DIRECTIONS.into_iter().enumerate() {
            flip_direction(&mut fresh_board.cells, (x, y), (h, v), self.flips[i]);
        }
        fresh_board.cells[x as usize][y as usize] = Some(self.color);
        fresh_board
    }
}

/// 'Board' is a 8x8 matrix of cells which can hold disks.
#[derive(Debug, PartialEq)]
pub struct Board {
    pub cells: [[Option<Color>; 8]; 8] // 8x8 board
}
impl Board {
    pub fn new() -> Board {
        let mut cells = [[None; 8]; 8];
        // Central cells occupied
        cells[3][3] = Some(Color::Black);
        cells[3][4] = Some(Color::White);
        cells[4][3] = Some(Color::White);
        cells[4][4] = Some(Color::Black);
        Board { cells: cells }
    }

    /// Tests all the moves a given color can take on the board.
    fn test(&self, color: Color) -> Vec<Vec<Result<LegalMove, IllegalMove>>> {

        // Tests whether there are valid flips on the position and returns them.
        // Otherwise it gives an error with the reason.
        fn test_position(cells: &[[Option<Color>; 8]; 8], color: Color, position: Coord) -> Result<LegalMove, IllegalMove> {
            let mut sum = 0;
            let mut flips = [0; 8];
            // Finds the number of opponent's pieces sandwiched in a straight line between our color. Recursive
            fn test_direction(cells: &[[Option<Color>; 8]; 8], color: Color, position: Coord, direction: Coord, n: i8) -> i8 {
                let &(x, y) = &position;
                let &(dx, dy) = &direction;
                let next = (x + dx, y + dy);
                let &(nx, ny) = &next;
                if nx == 8 || nx == -1 || ny == 8 || ny == -1 { // out of range
                    0
                } else {
                    match cells[nx as usize][ny as usize] {
                        None => 0,
                        Some(found) if found == color => n,
                        Some(_) => test_direction(cells, color, next, direction, n + 1)
                    }
                }
            };
            let &(x, y) = &position;
            match cells[x as usize][y as usize] {
                // can't put on already occupied field
                Some(color) => Err(IllegalMove::Occupied(color)),
                None => {
                    for (i, &(h, v)) in DIRECTIONS.into_iter().enumerate() {
                        let n = test_direction(cells, color, (x, y), (h, v), 0);
                        sum += n;
                        flips[i] = n;
                    }
                    // illegal move because it didn't flip anything
                    if sum == 0 {
                        Err(IllegalMove::Ineffective)
                    } else {
                        Ok(LegalMove {
                            color: color,
                            flips: flips,
                            position: position
                        })
                    }
                }
            }
        }
        (0..7).map(|i| (0..7).map(|j| test_position(&self.cells, color, (i, j))).collect()).collect()
    }
}
/// Enumerates possible states of the game.
pub enum Game {
    /// The player should place a piece on the board.
    Place(Place),
    /// The player should skip their turn.
    Skip(Skip),
    /// The game has ended. Moves are no longer possible.
    End
}

/// Place is state that has the place move as continuation.
pub struct Place {
    pub player: Color,
    pub retry_reason: Option<IllegalMove>,
    pub moves: Vec<Vec<Result<LegalMove, IllegalMove>>>,
    pub board: Board
}
impl Place {
    /// Place your disk on the selected coordinate. Returns new game state.
    pub fn place(self, selected_cell: Coord) -> Game {
        let (x, y) = selected_cell;
        match self.moves[x as usize][y as usize]  {
            Err(illegal_move) => {
                let mut new_self = self;
                new_self.retry_reason = Some(illegal_move);
                Game::Place(new_self)
            }
            Ok(legal_move) => {
                let next_board = legal_move.apply(self.board);
                let next_player = self.player.flip();
                let next_moves = next_board.test(next_player);
                let has_valid_move = next_moves.iter().any(|x| x.iter().any(|x| x.is_ok()));
                if has_valid_move {
                    Game::Place(Place {
                        player: next_player,
                        board: next_board,
                        moves: next_moves,
                        retry_reason: None
                    })
                } else {
                    Game::Skip(Skip {
                        player: next_player,
                        board: next_board
                    })
                }
            }
        }
    }
}

/// Skip is state that has the skip move as continuation.
pub struct Skip {
    pub player: Color,
    pub board: Board
}
impl Skip {
    /// Skip the next move. Returns new game state.
    pub fn skip(self) -> Game {
        let next_player = self.player.flip();
        let next_moves = self.board.test(next_player);
        let has_valid_move = next_moves.iter().any(|x| x.iter().any(|x| x.is_ok()));
        if has_valid_move {
            Game::Place(Place {
                player: next_player,
                board: self.board,
                moves: next_moves,
                retry_reason: None
            })
        } else {
            Game::End
        }
    }
}

pub fn new_game() -> Game {

    let board = Board::new();
    let player = Color::Black;
    let moves = board.test(player);

    Game::Place(Place {
        player: player,
        board: board,
        moves: moves,
        retry_reason: None
    })
}
