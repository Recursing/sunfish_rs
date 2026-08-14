use crate::board::{BOARD_SIDE, BOARD_SIZE, PADDING};

pub struct Direction {}

impl Direction {
    pub const NORTH: i32 = -(BOARD_SIDE as i32);
    pub const EAST: i32 = 1;
    pub const SOUTH: i32 = BOARD_SIDE as i32;
    pub const WEST: i32 = -1;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(u8)]
// The low nibble identifies the piece and the high nibble identifies its side.
pub enum Square {
    MyPawn = 0x01,
    MyKnight = 0x02,
    MyBishop = 0x03,
    MyRook = 0x04,
    MyQueen = 0x05,
    MyKing = 0x06,
    OpponentPawn = 0x11,
    OpponentKnight = 0x12,
    OpponentBishop = 0x13,
    OpponentRook = 0x14,
    OpponentQueen = 0x15,
    OpponentKing = 0x16,
    Empty = 0xFE,
    Wall = 0xFF, // Here to simplify detection of out of board moves
}

impl Square {
    pub fn is_my_piece(self) -> bool {
        matches!(self as u8, 0x01..=0x06)
    }

    pub fn is_opponent_piece(self) -> bool {
        matches!(self as u8, 0x11..=0x16)
    }

    pub fn swap_color(self) -> Square {
        match self {
            Square::Empty => Square::Empty,
            Square::Wall => Square::Wall,
            Square::MyPawn => Square::OpponentPawn,
            Square::MyKing => Square::OpponentKing,
            Square::MyRook => Square::OpponentRook,
            Square::MyKnight => Square::OpponentKnight,
            Square::MyBishop => Square::OpponentBishop,
            Square::MyQueen => Square::OpponentQueen,
            Square::OpponentPawn => Square::MyPawn,
            Square::OpponentKing => Square::MyKing,
            Square::OpponentRook => Square::MyRook,
            Square::OpponentKnight => Square::MyKnight,
            Square::OpponentBishop => Square::MyBishop,
            Square::OpponentQueen => Square::MyQueen,
        }
    }

    pub fn move_directions(self) -> &'static [i32] {
        match self {
            Square::MyPawn => &[
                Direction::NORTH,
                Direction::NORTH + Direction::NORTH,
                Direction::NORTH + Direction::WEST,
                Direction::NORTH + Direction::EAST,
            ],
            Square::MyKnight => &[
                Direction::NORTH + Direction::NORTH + Direction::EAST,
                Direction::NORTH + Direction::NORTH + Direction::WEST,
                Direction::WEST + Direction::WEST + Direction::NORTH,
                Direction::WEST + Direction::WEST + Direction::SOUTH,
                Direction::SOUTH + Direction::SOUTH + Direction::WEST,
                Direction::SOUTH + Direction::SOUTH + Direction::EAST,
                Direction::EAST + Direction::EAST + Direction::SOUTH,
                Direction::EAST + Direction::EAST + Direction::NORTH,
            ],
            Square::MyBishop => &[
                Direction::NORTH + Direction::EAST,
                Direction::NORTH + Direction::WEST,
                Direction::WEST + Direction::SOUTH,
                Direction::SOUTH + Direction::EAST,
            ],
            Square::MyRook => &[
                Direction::NORTH,
                Direction::WEST,
                Direction::SOUTH,
                Direction::EAST,
            ],
            Square::MyQueen | Square::MyKing => &[
                Direction::NORTH,
                Direction::WEST,
                Direction::SOUTH,
                Direction::EAST,
                Direction::NORTH + Direction::EAST,
                Direction::NORTH + Direction::WEST,
                Direction::WEST + Direction::SOUTH,
                Direction::SOUTH + Direction::EAST,
            ],
            _ => panic!(),
        }
    }

    pub fn midgame_value(self, position: usize) -> i32 {
        debug_assert!(
            (BOARD_SIDE * PADDING + PADDING..BOARD_SIZE - BOARD_SIDE * PADDING - PADDING)
                .contains(&position)
                && position % BOARD_SIDE >= PADDING
                && position % BOARD_SIDE < BOARD_SIDE - PADDING
        );

        // Tuned piece-square tables from Sunfish.
        let piece_value = match self {
            Square::MyPawn => 100,
            Square::MyKnight => 280,
            Square::MyBishop => 320,
            Square::MyRook => 479,
            Square::MyQueen => 929,
            Square::MyKing => 32000,
            _ => panic!(),
        };

        let piece_position_value = match self {
            Square::MyPawn => &[
                0, 0, 0, 0, 0, 0, 0, 0, // Last rank, no pawns
                78, 83, 86, 73, 102, 82, 85, 90, //
                7, 29, 21, 44, 40, 31, 44, 7, //
                -17, 16, -2, 15, 14, 0, 15, -13, //
                -26, 3, 10, 9, 6, 1, 0, -23, //
                -22, 9, 5, -11, -10, -2, 3, -19, //
                -31, 8, -7, -37, -36, -14, 3, -31, //
                0, 0, 0, 0, 0, 0, 0, 0,
            ],
            Square::MyKnight => &[
                -66, -53, -75, -75, -10, -55, -58, -70, //
                -3, -6, 100, -36, 4, 62, -4, -14, //
                10, 67, 1, 74, 73, 27, 62, -2, //
                24, 24, 45, 37, 33, 41, 25, 17, //
                -1, 5, 31, 21, 22, 35, 2, 0, //
                -18, 10, 13, 22, 18, 15, 11, -14, //
                -23, -15, 2, 0, 2, 0, -23, -20, //
                -74, -23, -26, -24, -19, -35, -22, -69, //
            ],
            Square::MyBishop => &[
                -59, -78, -82, -76, -23, -107, -37, -50, //
                -11, 20, 35, -42, -39, 31, 2, -22, //
                -9, 39, -32, 41, 52, -10, 28, -14, //
                25, 17, 20, 34, 26, 25, 15, 10, //
                13, 10, 17, 23, 17, 16, 0, 7, //
                14, 25, 24, 15, 8, 25, 20, 15, //
                19, 20, 11, 6, 7, 6, 20, 16, //
                -7, 2, -15, -12, -14, -15, -10, -10, //
            ],
            Square::MyRook => &[
                35, 29, 33, 4, 37, 33, 56, 50, //
                55, 29, 56, 67, 55, 62, 34, 60, //
                19, 35, 28, 33, 45, 27, 25, 15, //
                0, 5, 16, 13, 18, -4, -9, -6, //
                -28, -35, -16, -21, -13, -29, -46, -30, //
                -42, -28, -42, -25, -25, -35, -26, -46, //
                -53, -38, -31, -26, -29, -43, -44, -53, //
                -30, -24, -2, 5, -2, -2, -31, -32, //
            ],
            Square::MyQueen => &[
                6, 1, -8, -104, 69, 24, 88, 26, //
                14, 32, 60, -10, 20, 76, 57, 24, //
                -2, 43, 32, 60, 72, 63, 43, 2, //
                1, -16, 22, 17, 25, 20, -13, -6, //
                -14, -15, -2, -5, -1, -10, -20, -22, //
                -30, -6, -13, -11, -16, -11, -16, -27, //
                -36, -18, 0, -19, -15, -15, -21, -38, //
                -39, -30, -31, -13, -31, -36, -34, -42, //
            ],
            Square::MyKing => &[
                4, 54, 47, -99, -99, 60, 83, -62, //
                -32, 10, 55, 56, 56, 55, 10, 3, //
                -62, 12, -57, 44, -67, 28, 37, -31, //
                -55, 50, 11, -4, -19, 13, 0, -49, //
                -55, -43, -52, -28, -51, -47, -8, -50, //
                -47, -42, -43, -79, -64, -32, -29, -32, //
                -4, 3, -14, -50, -57, -18, 13, 4, //
                17, 30, -3, -14, 6, -1, 40, 18, //
            ],
            _ => &[0; 64],
        };
        let real_position = position - PADDING * BOARD_SIDE;
        let row_number = real_position / BOARD_SIDE;
        let square_index = real_position - PADDING * (2 * row_number + 1);
        let knight_on_rim_penalty = if self == Square::MyKnight && matches!(square_index % 8, 0 | 7)
        {
            50
        } else {
            0
        };
        piece_value + piece_position_value[square_index] - knight_on_rim_penalty
    }
}
