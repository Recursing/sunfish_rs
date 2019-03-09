use crate::board::{after_move, gen_moves, move_value, nullmove, zobrist_hash, BoardState};
use crate::pieces::{Piece, Square};
use std::cmp::max;
use std::time::{Duration, Instant};

extern crate lru;

use lru::LruCache;

pub const MATE_UPPER: i32 = 60_000 + 10 * 929; // TODO move somewhere else, do we need MATE_UPPER?
pub const MATE_LOWER: i32 = 60_000 - 10 * 929;
const TRANSPOSITION_TABLE_SIZE: usize = 10_000_000; // TODO explain, TODO why does it use so much memory?? more than py?
const QUIESCENCE_SEARCH_LIMIT: i32 = 150;
const EVAL_ROUGHNESS: i32 = 20;

#[derive(Clone, Copy)]
struct Entry {
    lower: i32,
    upper: i32,
}

const DEFAULT_ENTRY: Entry = Entry {
    lower: -MATE_UPPER,
    upper: MATE_UPPER,
};

pub struct Searcher {
    score_transposition_table: LruCache<(u64, i32, bool), Entry>,
    move_transposition_table: LruCache<u64, (usize, usize)>,
    nodes: u32,
}
impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            score_transposition_table: LruCache::new(TRANSPOSITION_TABLE_SIZE),
            move_transposition_table: LruCache::new(TRANSPOSITION_TABLE_SIZE),
            nodes: 0,
        }
    }

    fn bound(&mut self, board_state: &BoardState, gamma: i32, depth: i32, root: bool) -> i32 {
        let hash = zobrist_hash(board_state);
        self.nodes += 1;

        // Sunfish is a king-capture engine, so we should always check if we
        // still have a king. Notice since this is the only termination check,
        // the remaining code has to be comfortable with being mated, stalemated
        // or able to capture the opponent king.
        if board_state.score <= -MATE_LOWER {
            return -MATE_UPPER;
        }

        // Look into the table if we have already searched this position before.
        // We also need to be sure, that the stored search was over the same
        // nodes as the current search.
        // Depth <= 0 is Quiescence Search. Here any position is searched as deeply as is needed
        // for calmness, and so there is no reason to keep different depths in the
        // transposition table.

        let entry = *self
            .score_transposition_table
            .get(&(hash, max(depth, 0), root))
            .unwrap_or(&DEFAULT_ENTRY);

        if entry.lower >= gamma && (!root || self.move_transposition_table.get(&hash).is_some()) {
            return entry.lower;
        } else if entry.upper < gamma {
            return entry.upper;
        }

        let mut best = -MATE_UPPER;
        // First try not moving at all
        if depth > 0
            && !root
            && (vec![
                Square::MyPiece(Piece::Rook),
                Square::MyPiece(Piece::Knight),
                Square::MyPiece(Piece::Bishop),
                Square::MyPiece(Piece::Queen),
            ]
            .iter()
            .any(|s| board_state.board.contains(&s)))
        {
            let score = -self.bound(&nullmove(board_state), 1 - gamma, depth - 3, false);
            best = std::cmp::max(best, score);
        } else {
            // For QSearch we have a different kind of null-move
            let score = board_state.score;
            best = std::cmp::max(best, score);
        }
        // Then killer move. We search it twice, but the tp will fix things for
        // us. Note, we don't have to check for legality, since we've already
        // done it before. Also note that in QS the killer must be a capture,
        // otherwise we will be non deterministic.
        let killer: Option<&(usize, usize)> = self.move_transposition_table.get(&hash);

        if best < gamma && killer.is_some() {
            let killer_move = killer.unwrap().clone();
            if depth > 0 || move_value(board_state, &killer_move) >= QUIESCENCE_SEARCH_LIMIT {
                let score = -self.bound(
                    &after_move(board_state, &killer_move),
                    1 - gamma,
                    depth - 1,
                    false,
                );
                best = std::cmp::max(best, score);
                // should I add it again to the move_transposition_table?
            }
        }

        if best < gamma {
            // Then all the other moves
            let mut others = gen_moves(board_state);
            others.sort_unstable_by_key(|m| -move_value(board_state, m));
            for m in &others {
                if depth > 0 || move_value(board_state, m) >= QUIESCENCE_SEARCH_LIMIT {
                    let score =
                        -self.bound(&after_move(board_state, m), 1 - gamma, depth - 1, false);
                    best = std::cmp::max(best, score);
                    if best >= gamma {
                        // Save the move for pv construction and killer heuristic
                        self.move_transposition_table.put(hash, *m);
                        //println!("other: {}", best);
                        break;
                    }
                }
            }
        }

        // Stalemate checking is a bit tricky: Say we failed low, because
        // we can't (legally) move and so the (real) score is -infty.
        // At the next depth we are allowed to just return r, -infty <= r < gamma,
        // which is normally fine.
        // However, what if gamma = -10 and we don't have any legal moves?
        // Then the score is actaully a draw and we should fail high!
        // Thus, if best < gamma and best < 0 we need to double check what we are doing.
        // This doesn't prevent sunfish from making a move that results in stalemate,
        // but only if depth == 1, so that's probably fair enough.
        // (Btw, at depth 1 we can also mate without realizing.)
        if best < gamma && best < 0 && depth > 0 {
            let is_dead = |pos: BoardState| {
                gen_moves(&pos)
                    .iter()
                    .any(|m| move_value(&pos, m) >= MATE_LOWER)
            };
            if gen_moves(board_state)
                .iter()
                .all(|m| is_dead(after_move(board_state, m)))
            {
                let in_check = is_dead(nullmove(board_state));
                best = if in_check { -MATE_UPPER } else { 0 };
            }
        }

        // Update score_transposition_table
        if best >= gamma {
            self.score_transposition_table.put(
                (hash, depth, root),
                Entry {
                    lower: best,
                    upper: entry.upper,
                },
            );
        } else if best < gamma {
            self.score_transposition_table.put(
                (hash, depth, root),
                Entry {
                    lower: entry.lower,
                    upper: best,
                },
            );
        }

        best
    }

    // Iterative deepening MTD-bi search
    pub fn search(&mut self, board_state: BoardState, duration: Duration) -> ((usize, usize), i32) {
        self.nodes = 0;
        let hash = zobrist_hash(&board_state);
        let mut depth = 0;
        let now = Instant::now();

        // Bound depth to avoid infinite recursion in finished games
        for _depth in 1..999 {
            depth = _depth;
            let mut lower = -MATE_UPPER;
            let mut upper = MATE_UPPER;
            while lower < upper - EVAL_ROUGHNESS {
                let gamma = (lower + upper + 1) / 2;
                let score = self.bound(&board_state, gamma, depth, true);
                if score >= gamma {
                    lower = score;
                } else {
                    upper = score;
                }
            }
            let _score = self.bound(&board_state, lower, depth, true);
            //println!("Reached depth {}", depth);
            //println!("Examined nodes {}", self.nodes);
            //println!("Searched for {:?}", now.elapsed());
            if now.elapsed() > duration {
                break;
            }
        }

        // If the game hasn't finished we can retrieve our move from the
        // transposition table.
        return (
            *self
                .move_transposition_table
                .get(&hash)
                .expect("move not in table"),
            self.score_transposition_table
                .get(&(hash, depth, true))
                .expect("score not in table")
                .lower,
        );
    }
}
