//! This module encapsulates the core game logic.

use std::fmt;

/// Associates a piece with a player.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    Black,
    White
}
impl Color {
    /// Flipping a piece results in a piece of opposite color.
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

/// Denotes a position indexed by colum and row on the board.
pub type Coord = (i8, i8);

/// Enumerates all possible flip directions on the board.
///
/// The first element of the product denotes delta in column index, the second delta in row index.
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

/// Lists the reasons why a move is illegal.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IllegalMove {
    /// The cell is already occupied by the given color.
    Occupied(Color),
    /// The move does not cause any disks to be flipped over.
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

/// Holds the description of a legal move on the board.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LegalMove {
    /// The color being placed.
    pub color: Color,
    /// Number of flipped pieced by direction enumerated in the same sequence as [`DIRECTIONS`](constant.DIRECTIONS.html).
    pub flips: [i8; 8],
    /// The position on the board where to place the color.
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

/// An 8x8 matrix of cells holding disks.
///
/// It is immutable and represents a constellation of pieces.
/// If and only if two boards have the same constellation, are they considered
/// equal.
#[derive(Debug, PartialEq)]
pub struct Board {
    pub cells: [[Option<Color>; 8]; 8]
}
impl Board {
    /// Creates a board for the starting constellation.
    pub fn new() -> Board {
        let mut cells = [[None; 8]; 8];
        // Central cells occupied
        cells[3][3] = Some(Color::Black);
        cells[3][4] = Some(Color::White);
        cells[4][3] = Some(Color::White);
        cells[4][4] = Some(Color::Black);
        Board { cells: cells }
    }

    /// Tests all the moves a given player can take on the board.
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

/// The game state that has a placing move as continuation.
pub struct Place {
    /// The player who should place a piece next.
    pub player: Color,
    /// If this move is retry, it contains the reason why the original move is illegal.
    pub retry_reason: Option<IllegalMove>,
    /// Contains the fact for each cell on `board` whether placing the piece by `player` in that cell is legal or illegal.
    /// Column is the primary index.
    pub moves: Vec<Vec<Result<LegalMove, IllegalMove>>>,
    pub board: Board
}
impl Place {
    /// Place a piece with `self`'s color on the selected coordinate of `self`'s board.
    ///
    /// A legal move will result in a new board, as the constellation if pieces always change this way.
    /// A move may be illegal, in which case the board remains the same and the player is signalled
    /// by setting `retry_reason`to some [`IllegalMove`](enum.IllegalMove.html).
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

/// The game state that has a skipping move as continuation.
///
/// Making explicit that a given player has no valid step in theircurrent
/// turn is a simple way of notifying the player of this situation. Also
/// makes a turn consist of exactly one move by each player, which can simplify
/// logic in the UI.
pub struct Skip {
    pub player: Color,
    pub board: Board
}
impl Skip {
    /// Skip the next move. Returns new game state. Board remains the same.
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

/// Initializes a game to the starting state.
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