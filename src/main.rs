use std::fmt;

/// A Disk is a piece owned by either black or white player.
/// It is flippable!
#[derive(Debug, Copy, Clone, PartialEq)]
enum Disk { Black, White }
trait Flip { fn flip(&self) -> Disk; }
impl Flip for Disk {
    fn flip(&self) -> Disk {
        match *self {
            Disk::Black => Disk::White,
            Disk::White => Disk::Black
        }
    }
}
impl fmt::Display for Disk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Disk::Black => write!(f, "Black"),
            Disk::White => write!(f, "White")
        }
    }
}

/// All directions and their deltas
const DIRECTIONS: [(i8, i8); 8] = [
    (0, 1), // N
    (1, 1), // NE
    (1, 0), // E
    (1, -1), // SE
    (0, -1), // S
    (-1, -1), // SW
    (-1, 0), // W
    (-1, 1) // NW
];

/// Reasons why placing a disk on a cell can be illegal
#[derive(Debug, Copy, Clone, PartialEq)]
enum IllegalMoveReason {
    Occupied,
    Ineffective
}
impl fmt::Display for IllegalMoveReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IllegalMoveReason::Occupied => write!(f, "Occupied"),
            IllegalMoveReason::Ineffective => write!(f, "Ineffective")
        }
    }
}

struct Board {
    cells: [[Option<Disk>; 8]; 8] // 8x8 board
}
impl Board {
    fn new () -> Board {
        let mut cells = [[None; 8]; 8];
        // Central cells occupied
        cells[3][3] = Some(Disk::Black);
        cells[3][4] = Some(Disk::White);
        cells[4][3] = Some(Disk::White);
        cells[4][4] = Some(Disk::Black);
        Board { cells: cells }
    }

    /// Tests whether there are valid flips on the position and returns them.
    /// Otherwise it gives an error with the reason.
    fn test(&self, disk: Disk, position: (i8, i8)) -> Result<[i8; 8], IllegalMoveReason> {
        let &(x, y) = &position;
        // can't put on already occupied field
        if self.cells[x as usize][y as usize] != None {
            return Err(IllegalMoveReason::Occupied)
        }
        let mut sum = 0;
        let mut result = [0; 8];

        // find the number of opponent's pieces sandwiched into our color, in a straight line
        // closures cannot be recursive, so I used a regular function
        fn test_direction(cells: &[[Option<Disk>; 8]; 8], disk: Disk, position: (i8, i8), direction: (i8, i8), n: i8) -> i8 {
            let &(x, y) = &position;
            let &(dx, dy) = &direction;
            let next = (x + dx, y + dy);
            let &(nx, ny) = &next;
            if nx == 8 || nx == -1 || ny == 8 || ny == -1 { // out of range
                return 0
            }
            match cells[nx as usize][ny as usize] {
                None => 0,
                Some(found) if found == disk => n,
                Some(_) => test_direction(cells, disk, next, direction, n + 1)
            }
        };
        for (i, &(h, v)) in DIRECTIONS.into_iter().enumerate() {
            let n = test_direction(&self.cells, disk, (x, y), (h, v), 0);
            sum += n;
            result[i] = n;
        }
        // illegal move, because no flip is possible on this cell
        if sum == 0 {
            Err(IllegalMoveReason::Ineffective)
        } else {
            Ok(result)
        }
    }

    /// Puts a disk in place and flips with the given flip definitions
    fn put (&mut self, disk: Disk, position: (i8, i8), flips: [i8; 8]) {
        let &(x, y) = &position;

        // closures cannot be recursive, so I used a regular function
        fn flip_direction(cells: &mut [[Option<Disk>; 8]; 8], position: (i8, i8), direction: (i8, i8), n: i8) {
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
            flip_direction(&mut self.cells, (x, y), (h, v), flips[i]);
        }
        self.cells[x as usize][y as usize] = Some(disk);
    }
}

fn main () {
    let mut board = Board::new();
     match board.test(Disk::Black, (3, 5)) {
        Ok(flips) => {
            board.put(Disk::Black, (3, 5), flips);
            assert_eq!(board.cells[3][4].unwrap(), Disk::Black);
            println!("Success!")
        },
        Err(err) => panic!("Should not happen: {}", err)
    }
}
