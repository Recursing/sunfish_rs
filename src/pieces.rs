use std::collections::HashMap;
extern crate rand;
use crate::board::{BOARD_SIDE, BOARD_SIZE, PADDING};
use lazy_static::lazy_static;
use rand::random;

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

lazy_static! {
    pub static ref PIECE_SQUARE_TABLES: HashMap<Piece, [i32; BOARD_SIZE]> =
        get_piece_square_tables();

    // See https://en.wikipedia.org/wiki/Zobrist_hashing
    pub static ref ZOBRIST_MAP: HashMap<(usize, Square), u64> = get_zobrist_map();
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
}

pub fn get_piece_square_tables() -> HashMap<Piece, [i32; BOARD_SIZE]> {
    // Piece square tables: piece value in different positions
    // Values from https://github.com/official-stockfish/Stockfish/blob/05f7d59a9a27d9f8bce8bde4e9fed7ecefeb03b9
    // For now just middle game, could add endgames

    // From stockfish /src/types.h#L182,
    let mut piece_values: HashMap<Piece, i32> = HashMap::new();
    piece_values.insert(Piece::Pawn, 136); // 208
    piece_values.insert(Piece::Knight, 782); // 865
    piece_values.insert(Piece::Bishop, 830); // 918
    piece_values.insert(Piece::Rook, 1289); // 1378
    piece_values.insert(Piece::Queen, 2529); // 2687
    piece_values.insert(Piece::King, 32000);

    // From stockfish /src/psqt.cpp#L31
    let mut piece_position_values: HashMap<Piece, [i32; 64]> = HashMap::new();
    piece_position_values.insert(
        Piece::Pawn,
        [
            0, 0, 0, 0, 0, 0, 0, 0, // Last rank, no pawns
            -10, 6, -5, -11, -2, -14, 12, -1, //
            -6, -8, 5, 11, -14, 0, -12, -14, //
            6, -3, -10, 1, 12, 6, -12, 1, //
            -9, -18, 8, 22, 33, 25, -4, -16, //
            -11, -10, -35, 22, 26, -35, 4, -24, //
            0, -5, 10, 13, 21, 17, 6, -3, //
            0, 0, 0, 0, 0, 0, 0, 0, //
        ],
    );
    piece_position_values.insert(
        Piece::Knight,
        [
            -200, -80, -53, -32, -32, -53, -80, -200, //
            -67, -21, 6, 37, 37, 6, -21, -67, //
            -11, 28, 63, 55, 55, 63, 28, -11, //
            -29, 13, 42, 52, 52, 42, 13, -29, //
            -28, 5, 41, 47, 47, 41, 5, -28, //
            -64, -20, 4, 19, 19, 4, -20, -64, //
            -79, -39, -24, -9, -9, -24, -39, -79, //
            -169, -96, -80, -79, -79, -80, -96, -169, //
        ],
    );
    piece_position_values.insert(
        Piece::Bishop,
        [
            -48, -3, -12, -25, -25, -12, -3, -48, //
            -21, -19, 10, -6, -6, 10, -19, -21, //
            -17, 4, -1, 8, 8, -1, 4, -17, //
            -7, 30, 23, 28, 28, 23, 30, -7, //
            1, 8, 26, 37, 37, 26, 8, 1, //
            -8, 24, -3, 15, 15, -3, 24, -8, //
            -18, 7, 14, 3, 3, 14, 7, -18, //
            -44, -4, -11, -28, -28, -11, -4, -44, //
        ],
    );
    piece_position_values.insert(
        Piece::Rook,
        [
            -22, -24, -6, 4, 4, -6, -24, -22, //
            -8, 6, 10, 12, 12, 10, 6, -8, //
            -24, -4, 4, 10, 10, 4, -4, -24, //
            -24, -12, -1, 6, 6, -1, -12, -24, //
            -13, -5, -4, -6, -6, -4, -5, -13, //
            -21, -7, 3, -1, -1, 3, -7, -21, //
            -18, -10, -5, 9, 9, -5, -10, -18, //
            -24, -13, -7, 2, 2, -7, -13, -24, //
        ],
    );
    piece_position_values.insert(
        Piece::Queen,
        [
            -2, -2, 1, -2, -2, 1, -2, -2, //
            -5, 6, 10, 8, 8, 10, 6, -5, //
            -4, 10, 6, 8, 8, 6, 10, -4, //
            0, 14, 12, 5, 5, 12, 14, 0, //
            4, 5, 9, 8, 8, 9, 5, 4, //
            -3, 6, 13, 7, 7, 13, 6, -3, //
            -3, 5, 8, 12, 12, 8, 5, -3, //
            3, -5, -5, 4, 4, -5, -5, 3, //
        ],
    );
    piece_position_values.insert(
        Piece::King,
        [
            64, 87, 49, 0, 0, 49, 87, 64, //
            87, 120, 64, 25, 25, 64, 120, 87, //
            122, 159, 85, 36, 36, 85, 159, 122, //
            145, 176, 112, 69, 69, 112, 176, 145, //
            169, 191, 136, 108, 108, 136, 191, 169, //
            198, 253, 168, 120, 120, 168, 253, 198, //
            277, 305, 241, 183, 183, 241, 305, 277, //
            272, 325, 273, 190, 190, 273, 325, 272, //
        ],
    );

    // TODO link to piece square tables, add explanation
    let mut temp_piece_square_tables: HashMap<Piece, [i32; BOARD_SIZE]> = HashMap::new();

    for (piece, position_values) in piece_position_values {
        let piece_value = piece_values[&piece];
        let mut piece_square_table = [0; BOARD_SIZE];

        for position in 0..(PADDING * BOARD_SIDE) {
            piece_square_table[position] = 0;
            piece_square_table[BOARD_SIZE - position - 1] = 0;
        }
        for rank in 0..8 {
            let first_of_rank = (rank + PADDING) * BOARD_SIDE;
            for pad in 0..PADDING {
                piece_square_table[first_of_rank + pad] = 0;
                piece_square_table[first_of_rank + BOARD_SIDE - pad - 1] = 0;
            }
            for file in 0..8 {
                piece_square_table[first_of_rank + PADDING + file] =
                    position_values[rank * 8 + file] + piece_value;
            }
        }
        temp_piece_square_tables.insert(piece, piece_square_table);
    }

    temp_piece_square_tables
}

pub fn get_zobrist_map() -> HashMap<(usize, Square), u64> {
    let mut zobrist_map_temp: HashMap<(usize, Square), u64> = HashMap::new();
    let pieces = &[
        Piece::Pawn,
        Piece::Bishop,
        Piece::Knight,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];
    for position in 0..BOARD_SIZE {
        for &piece in pieces {
            zobrist_map_temp.insert((position, Square::MyPiece(piece)), random());
            zobrist_map_temp.insert((position, Square::OpponentPiece(piece)), random());
        }
        // Used for en passant, castling and king passant
        zobrist_map_temp.insert((position, Square::Empty), random());
    }
    zobrist_map_temp
}
