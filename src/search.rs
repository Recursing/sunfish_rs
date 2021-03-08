use log::info;
use std::cmp::max;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::board::{after_move, can_check, gen_moves, move_value, nullmove, BoardState};
use crate::pieces::Square;

pub const MATE_UPPER: i32 = 32_000 + 8 * 2529; // TODO move somewhere else, do we need MATE_UPPER?
pub const MATE_LOWER: i32 = 32_000 - 8 * 2529;
const TRANSPOSITION_TABLE_SIZE: usize = 1_000_000; // TODO explain, make more space efficient
const QUIESCENCE_SEARCH_LIMIT: i32 = 130;
const EVAL_ROUGHNESS: i32 = 10; // TODO do we need this?
const STOP_SEARCH: i32 = MATE_UPPER * 101;

#[derive(Clone, Copy)]
pub struct Entry {
    // pub for debugging TODO refactor
    lower: i32,
    upper: i32,
}

const DEFAULT_ENTRY: Entry = Entry {
    lower: -MATE_UPPER,
    upper: MATE_UPPER,
};

pub struct Searcher {
    pub score_transposition_table: HashMap<(BoardState, i32, bool), Entry>,
    pub move_transposition_table: HashMap<BoardState, (usize, usize)>,
    pub nodes: u32,
    now: Instant,
    duration: Duration,
}

impl Default for Searcher {
    fn default() -> Self {
        Searcher {
            score_transposition_table: HashMap::with_capacity(TRANSPOSITION_TABLE_SIZE),
            move_transposition_table: HashMap::with_capacity(TRANSPOSITION_TABLE_SIZE),
            nodes: 0,
            now: Instant::now(),
            duration: Duration::new(4, 0),
        }
    }
}

impl Searcher {
    fn bound(&mut self, board_state: &BoardState, gamma: i32, depth: i32, root: bool) -> i32 {
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
            .get(&(*board_state, max(depth, 0), root))
            .unwrap_or(&DEFAULT_ENTRY);

        if entry.lower >= gamma
            && (!root || self.move_transposition_table.get(board_state).is_some())
        // TODO do this last check before calling root, also remove root parameter
        {
            return entry.lower;
        } else if entry.upper < gamma {
            return entry.upper;
        }

        if self.now.elapsed() > self.duration {
            return STOP_SEARCH;
        }

        let mut best = -MATE_UPPER;
        // First try not moving at all
        if depth > 0
            && !root
            // TODO maybe base it on the board score?
            && (board_state.board.iter().any(|&s| matches!(s, Square::MyRook
                | Square::MyKnight
                | Square::MyBishop
                | Square::MyQueen)))
        {
            let score = -self.bound(&nullmove(board_state), 1 - gamma, depth - 3, false);
            if score == -STOP_SEARCH {
                return STOP_SEARCH;
            }
            best = std::cmp::max(best, score);
        } else if depth <= 0 {
            // For QSearch we have a different kind of null-move
            let score = board_state.score;
            best = std::cmp::max(best, score);
        }

        if best <= gamma {
            if let Some(killer_move) = self.move_transposition_table.get(board_state).copied() {
                // Then killer move. We search it twice, but the tp will fix things for
                // us. Note, we don't have to check for legality, since we've already
                // done it before. Also note that in QS the killer must be a capture,
                // otherwise we will be non deterministic.
                if depth > 0 || move_value(board_state, &killer_move) >= QUIESCENCE_SEARCH_LIMIT {
                    let score = -self.bound(
                        &after_move(board_state, &killer_move),
                        1 - gamma,
                        depth - 1,
                        false,
                    );
                    if score == -STOP_SEARCH {
                        return STOP_SEARCH;
                    }
                    best = std::cmp::max(best, score);
                    // should I add it again to the move_transposition_table?
                    // self.move_transposition_table.insert(*board_state, killer_move);
                }
            }
        }

        if best < gamma {
            // Then all the other moves
            let others = gen_moves(board_state);
            let check_bonus = |m| {
                if can_check(board_state, m) {
                    QUIESCENCE_SEARCH_LIMIT / 2
                } else {
                    0
                }
            };
            let mut move_vals: Vec<_> = others
                .iter()
                .map(|m| (-move_value(board_state, m) - check_bonus(m), m))
                .collect();
            move_vals.sort_unstable();
            for (val, m) in move_vals {
                if depth > 0
                    || (-val >= QUIESCENCE_SEARCH_LIMIT && (board_state.score - val > best))
                {
                    let score =
                        -self.bound(&after_move(board_state, m), 1 - gamma, depth - 1, false);
                    if score == -STOP_SEARCH {
                        return STOP_SEARCH;
                    }
                    best = std::cmp::max(best, score);
                    if best >= gamma {
                        // Save the move for pv construction and killer heuristic
                        if self.move_transposition_table.len() >= TRANSPOSITION_TABLE_SIZE {
                            self.move_transposition_table.clear();
                        }
                        self.move_transposition_table.insert(*board_state, *m);
                        break;
                    }
                } else {
                    break;
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
        if self.score_transposition_table.len() >= TRANSPOSITION_TABLE_SIZE {
            self.score_transposition_table.clear();
        }
        if best >= gamma {
            self.score_transposition_table.insert(
                (*board_state, depth, root),
                Entry {
                    lower: best,
                    upper: entry.upper,
                },
            );
        } else if best < gamma {
            self.score_transposition_table.insert(
                (*board_state, depth, root),
                Entry {
                    lower: entry.lower,
                    upper: best,
                },
            );
        }

        best
    }

    // Iterative deepening MTD-bi search
    pub fn search(
        &mut self,
        board_state: BoardState,
        duration: Duration,
    ) -> ((usize, usize), i32, i32) {
        self.nodes = 0;
        let mut reached_depth;
        self.now = Instant::now();
        self.duration = duration;
        let mut last_move = ((0, 0), 0, 0);

        // Bound depth to avoid infinite recursion in finished games
        for depth in 1..99 {
            // Realistically will reach depths around 6-12, except endgames
            let mut lower = -MATE_UPPER;
            let mut upper = MATE_UPPER;
            while lower < upper - EVAL_ROUGHNESS {
                let gamma = (lower + upper + 1) / 2;
                let score = self.bound(&board_state, gamma, depth, true);
                if score == STOP_SEARCH {
                    lower = STOP_SEARCH;
                    break;
                }
                if score >= gamma {
                    lower = score;
                } else {
                    upper = score;
                }
            }
            if lower == STOP_SEARCH {
                break;
            }
            let score = self.bound(&board_state, lower, depth, true);
            if score == STOP_SEARCH {
                break;
            }
            reached_depth = depth;
            info!(
                "Reached depth {: <2} score {: <5} nodes {: <7} time {:?}",
                depth,
                score,
                self.nodes,
                self.now.elapsed()
            );

            // If the game hasn't finished we can retrieve our move from the
            // transposition table.

            last_move = (
                *self
                    .move_transposition_table
                    .get(&board_state)
                    .expect("move not in table"),
                self.score_transposition_table
                    .get(&(board_state, reached_depth, true))
                    .expect("score not in table")
                    .lower,
                reached_depth,
            );

            if self.now.elapsed() > self.duration || score > MATE_LOWER {
                // Don't waste time if a mate is found
                break;
            }
        }

        last_move
    }

    // Done to prevent move repetitions
    pub fn set_eval_to_zero(&mut self, board_state: &BoardState) {
        // TODO there's probably a better way
        for depth in 1..30 {
            self.score_transposition_table
                .insert((*board_state, depth, false), Entry { lower: 0, upper: 0 });
        }
    }
}
