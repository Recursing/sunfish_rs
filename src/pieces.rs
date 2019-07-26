use crate::board::{BOARD_SIDE, BOARD_SIZE, PADDING};

pub struct Direction {}

impl Direction {
    pub const NORTH: i32 = -(BOARD_SIDE as i32);
    pub const EAST: i32 = 1;
    pub const SOUTH: i32 = BOARD_SIDE as i32;
    pub const WEST: i32 = -1;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Square {
    MyPiece(Piece),
    OpponentPiece(Piece),
    Empty,
    Wall, // Here to simplify detection of out of board moves
}

impl Piece {
    pub fn moves(self) -> &'static [i32] {
        match self {
            Piece::Pawn => &[
                Direction::NORTH,
                Direction::NORTH + Direction::NORTH,
                Direction::NORTH + Direction::WEST,
                Direction::NORTH + Direction::EAST,
            ],
            Piece::Knight => &[
                Direction::NORTH + Direction::NORTH + Direction::EAST,
                Direction::NORTH + Direction::NORTH + Direction::WEST,
                Direction::WEST + Direction::WEST + Direction::NORTH,
                Direction::WEST + Direction::WEST + Direction::SOUTH,
                Direction::SOUTH + Direction::SOUTH + Direction::WEST,
                Direction::SOUTH + Direction::SOUTH + Direction::EAST,
                Direction::EAST + Direction::EAST + Direction::SOUTH,
                Direction::EAST + Direction::EAST + Direction::NORTH,
            ],
            Piece::Bishop => &[
                Direction::NORTH + Direction::EAST,
                Direction::NORTH + Direction::WEST,
                Direction::WEST + Direction::SOUTH,
                Direction::SOUTH + Direction::EAST,
            ],
            Piece::Rook => &[
                Direction::NORTH,
                Direction::WEST,
                Direction::SOUTH,
                Direction::EAST,
            ],
            Piece::Queen | Piece::King => &[
                Direction::NORTH,
                Direction::WEST,
                Direction::SOUTH,
                Direction::EAST,
                Direction::NORTH + Direction::EAST,
                Direction::NORTH + Direction::WEST,
                Direction::WEST + Direction::SOUTH,
                Direction::SOUTH + Direction::EAST,
            ],
        }
    }

    pub fn midgame_value(self, position: usize) -> i32 {
        debug_assert!(
            position >= BOARD_SIDE * PADDING + PADDING
                && position < BOARD_SIZE - BOARD_SIDE * PADDING - PADDING
                && position % BOARD_SIDE >= PADDING
                && position % BOARD_SIDE < BOARD_SIDE - PADDING
        );

        // Piece square tables: piece value in different positions
        // Values from https://github.com/official-stockfish/Stockfish/blob/05f7d59a9a27d9f8bce8bde4e9fed7ecefeb03b9

        // From stockfish /src/types.h#L182,
        let piece_value = match self {
            Piece::Pawn => 136,
            Piece::Knight => 782,
            Piece::Bishop => 830,
            Piece::Rook => 1289,
            Piece::Queen => 2529,
            Piece::King => 32000,
        };

        // From stockfish /src/psqt.cpp#L31
        let piece_position_value = match self {
            Piece::Pawn => [
                0, 0, 0, 0, 0, 0, 0, 0, // Last rank, no pawns
                15, 31, 20, 14, 23, 11, 37, 24, //
                -1, -3, 15, 26, 1, 10, -7, -9, //
                8, -1, -5, 13, 24, 11, -10, 3, //
                -9, -18, 8, 32, 43, 25, -4, -16, //
                -9, -13, -40, 22, 26, -40, 1, -22, //
                2, 0, 15, 3, 11, 22, 11, -1, //
                0, 0, 0, 0, 0, 0, 0, 0,
            ],
            Piece::Knight => [
                -200, -80, -53, -32, -32, -53, -80, -200, //
                -67, -21, 6, 37, 37, 6, -21, -67, //
                -11, 28, 63, 55, 55, 63, 28, -11, //
                -29, 13, 42, 52, 52, 42, 13, -29, //
                -28, 5, 41, 47, 47, 41, 5, -28, //
                -64, -20, 4, 19, 19, 4, -20, -64, //
                -79, -39, -24, -9, -9, -24, -39, -79, //
                -169, -96, -80, -79, -79, -80, -96, -169, //
            ],
            Piece::Bishop => [
                -48, -3, -12, -25, -25, -12, -3, -48, //
                -21, -19, 10, -6, -6, 10, -19, -21, //
                -17, 4, -1, 8, 8, -1, 4, -17, //
                -7, 30, 23, 28, 28, 23, 30, -7, //
                1, 8, 26, 37, 37, 26, 8, 1, //
                -8, 24, -3, 15, 15, -3, 24, -8, //
                -18, 7, 14, 3, 3, 14, 7, -18, //
                -44, -4, -11, -28, -28, -11, -4, -44, //
            ],
            Piece::Rook => [
                -22, -24, -6, 4, 4, -6, -24, -22, //
                -8, 6, 10, 12, 12, 10, 6, -8, //
                -24, -4, 4, 10, 10, 4, -4, -24, //
                -24, -12, -1, 6, 6, -1, -12, -24, //
                -13, -5, -4, -6, -6, -4, -5, -13, //
                -21, -7, 3, -1, -1, 3, -7, -21, //
                -18, -10, -5, 9, 9, -5, -10, -18, //
                -24, -13, -7, 2, 2, -7, -13, -24, //
            ],
            Piece::Queen => [
                -2, -2, 1, -2, -2, 1, -2, -2, //
                -5, 6, 10, 8, 8, 10, 6, -5, //
                -4, 10, 6, 8, 8, 6, 10, -4, //
                0, 14, 12, 5, 5, 12, 14, 0, //
                4, 5, 9, 8, 8, 9, 5, 4, //
                -3, 6, 13, 7, 7, 13, 6, -3, //
                -3, 5, 8, 12, 12, 8, 5, -3, //
                3, -5, -5, 4, 4, -5, -5, 3, //
            ],
            Piece::King => [
                6, 8, 4, 0, 0, 4, 8, 6, //
                8, 12, 6, 2, 2, 6, 12, 8, //
                12, 15, 8, 3, 3, 8, 15, 12, //
                14, 17, 11, 6, 6, 11, 17, 15, //
                16, 19, 13, 10, 10, 13, 19, 16, //
                19, 25, 16, 12, 12, 16, 25, 19, //
                27, 30, 24, 18, 18, 24, 30, 27, //
                27, 32, 27, 19, 19, 27, 32, 27, //
            ],
        };
        let real_position = position - PADDING * BOARD_SIDE;
        let row_number = real_position / BOARD_SIDE;
        piece_value + piece_position_value[real_position - PADDING * (2 * row_number + 1)]
    }
}
