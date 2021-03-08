use crate::board::{rotate, static_score, BoardState, A1, A8, BOARD_SIDE, BOARD_SIZE, PADDING};
use crate::pieces::Square;

pub fn parse_move(move_: &str) -> (usize, usize) {
    let from = parse_coordinates(&move_[..2]);
    let to = parse_coordinates(&move_[2..]);
    (from, to)
}

pub fn parse_coordinates(coordinates: &str) -> usize {
    let mut chars = coordinates.chars();
    let file = chars.next().expect("Failed to parse coordinates");
    let rank = chars.next().expect("Failed to parse coordinates");
    A1 + (file as i32 - 'a' as i32) as usize
        - BOARD_SIDE as usize * ((rank as i32 - '1' as i32) as usize)
}

pub fn render_move(move_: &(usize, usize)) -> String {
    render_coordinates(move_.0) + &render_coordinates(move_.1)
}

fn render_coordinates(position: usize) -> String {
    let rank = b'8' - ((position - A8) as u8 / BOARD_SIDE as u8);
    let file = (position - A8) as u8 % BOARD_SIDE as u8 + b'a';
    [file as char, rank as char].iter().collect()
}

impl Square {
    pub fn to_unicode(self) -> char {
        match self {
            Square::MyRook => '♜',
            Square::MyKnight => '♞',
            Square::MyBishop => '♝',
            Square::MyQueen => '♛',
            Square::MyKing => '♚',
            Square::MyPawn => '♟',
            Square::OpponentRook => '♖',
            Square::OpponentKnight => '♘',
            Square::OpponentBishop => '♗',
            Square::OpponentQueen => '♕',
            Square::OpponentKing => '♔',
            Square::OpponentPawn => '♙',
            Square::Empty => '·',
            Square::Wall => 'X',
        }
    }
}

pub fn render_board(board_state: &BoardState) -> String {
    let mut rendered_board: String = String::from("");

    for (i, row) in board_state
        .board
        .chunks(BOARD_SIDE)
        .skip(PADDING)
        .take(8)
        .enumerate()
    {
        rendered_board.push_str(&format!(" {} ", 8 - i));
        for p in row.iter().skip(PADDING).take(8) {
            rendered_board.push_str(&format!(" {}", p.to_unicode()));
        }
        rendered_board.push_str("\n");
    }
    rendered_board.push_str("    a b c d e f g h \n\n");
    rendered_board.push_str(&format!("Static score: {}\n", board_state.score));
    if static_score(board_state.board) != board_state.score {
        rendered_board.push_str(&format!(
            "STATIC SCORE ERROR, SHOULD BE: {}\n",
            static_score(board_state.board),
        ));
    }
    if board_state.en_passant_position.is_some() {
        rendered_board.push_str(&format!(
            "En passant is {:?}\n",
            board_state.en_passant_position
        ));
    }
    rendered_board.push_str(&format!(
        "Castling rights are {:?} {:?}\n",
        board_state.my_castling_rights, board_state.opponent_castling_rights,
    ));
    rendered_board
}

// https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation#Definition
pub fn from_fen(fen: &str) -> BoardState {
    let mut new_board = [Square::Empty; BOARD_SIZE];
    let fields = fen.split(' ').collect::<Vec<_>>();
    let mut board_string: String = fields[0].into();
    let (turn, castling, en_passant, _halfmoves, _fullmoves) =
        (fields[1], fields[2], fields[3], fields[4], fields[5]);

    for (dv, dc) in "0123456789".chars().enumerate() {
        board_string = board_string.replace(dc, &"_".repeat(dv));
    }

    let board_lines: Vec<Vec<char>> = board_string
        .split('/')
        .map(|s| s.chars().collect())
        .collect();

    for rank in 0..BOARD_SIDE {
        for file in 0..BOARD_SIDE {
            let position = rank * BOARD_SIDE + file;
            new_board[position] = if rank < PADDING
                || file < PADDING
                || BOARD_SIDE - rank <= PADDING
                || BOARD_SIDE - file <= PADDING
            {
                Square::Wall
            } else {
                match board_lines[rank - PADDING][file - PADDING] {
                    'P' => Square::MyPawn,
                    'N' => Square::MyKnight,
                    'B' => Square::MyBishop,
                    'R' => Square::MyRook,
                    'Q' => Square::MyQueen,
                    'K' => Square::MyKing,
                    'p' => Square::OpponentPawn,
                    'n' => Square::OpponentKnight,
                    'b' => Square::OpponentBishop,
                    'r' => Square::OpponentRook,
                    'q' => Square::OpponentQueen,
                    'k' => Square::OpponentKing,
                    _ => Square::Empty,
                }
            }
        }
    }

    let en_passant_position = if en_passant == "-" {
        None
    } else {
        Some(parse_coordinates(en_passant))
    };

    let my_castling_rights = (castling.contains('Q'), castling.contains('K'));
    let opponent_castling_rights = (castling.contains('k'), castling.contains('q'));

    let mut boardstate = BoardState {
        board: new_board,
        score: static_score(new_board),
        my_castling_rights,
        opponent_castling_rights,
        en_passant_position,
        king_passant_position: None, // is not useful for legal board states
    };

    if turn == "b" {
        rotate(&mut boardstate);
    }

    boardstate
}
