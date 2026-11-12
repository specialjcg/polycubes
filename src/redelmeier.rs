//! Redelmeier's algorithm, generalised to the cubic lattice.
//!
//! Every fixed polycube is generated exactly once by growing it from its
//! lexicographically smallest cell, placed at the origin. Cells that precede
//! the origin in (z, y, x) order are forbidden, which removes translations.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crate::lattice::{Lattice, NEIGHBOUR_COUNT};

/// Number of final cells counted in closed form instead of being enumerated.
const CLOSED_FORM_CELL_COUNT: usize = 3;
/// Size of the polycubes whose subtrees are shared out between threads.
const FORK_CELL_COUNT: usize = 6;

/// Counts fixed polycubes (distinct up to translation only) of every size
/// from 1 to `max_cell_count`.
///
/// Element `size - 1` of the result is the number of fixed polycubes made of
/// `size` cubes (OEIS A001931).
#[must_use]
pub fn count_fixed_polycubes(max_cell_count: usize) -> Vec<u64> {
    if max_cell_count == 0 {
        return Vec::new();
    }
    let mut enumerator = Enumerator::new(max_cell_count, None);
    enumerator.extend(0, 0);
    enumerator.counts
}

/// Same result as [`count_fixed_polycubes`], computed on `thread_count` threads.
#[must_use]
pub fn count_fixed_polycubes_in_parallel(
    max_cell_count: usize,
    thread_count: NonZeroUsize,
) -> Vec<u64> {
    if max_cell_count < FORK_CELL_COUNT + CLOSED_FORM_CELL_COUNT {
        return count_fixed_polycubes(max_cell_count);
    }
    let claims = SubtreeClaims::new(fork_subtree_count());
    let counts_per_thread: Vec<Vec<u64>> = thread::scope(|scope| {
        let workers: Vec<_> = (0..thread_count.get())
            .map(|_| scope.spawn(|| count_claimed_subtrees(max_cell_count, &claims)))
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("worker thread panicked"))
            .collect()
    });
    merge_thread_counts(&counts_per_thread)
}

fn fork_subtree_count() -> usize {
    let fork_count = count_fixed_polycubes(FORK_CELL_COUNT)[FORK_CELL_COUNT - 1];
    usize::try_from(fork_count).expect("fork subtree count fits in usize")
}

fn count_claimed_subtrees(max_cell_count: usize, claims: &SubtreeClaims) -> Vec<u64> {
    let mut enumerator = Enumerator::new(max_cell_count, Some(claims));
    enumerator.extend(0, 0);
    enumerator.counts
}

/// Every thread walks the whole tree down to the fork size, so those counts
/// are complete in each thread; beyond it each thread only holds the subtrees
/// it claimed.
fn merge_thread_counts(counts_per_thread: &[Vec<u64>]) -> Vec<u64> {
    let mut merged_counts = counts_per_thread[0].clone();
    for thread_counts in &counts_per_thread[1..] {
        let partial_counts = merged_counts.iter_mut().zip(thread_counts);
        for (merged_count, thread_count) in partial_counts.skip(FORK_CELL_COUNT) {
            *merged_count += thread_count;
        }
    }
    merged_counts
}

/// Shared register of the fork-size subtrees already taken by a thread.
///
/// Threads walk the tree in the same order, so a subtree is identified by its
/// rank among the polycubes of the fork size.
struct SubtreeClaims {
    is_claimed: Vec<AtomicBool>,
}

impl SubtreeClaims {
    fn new(subtree_count: usize) -> Self {
        Self {
            is_claimed: (0..subtree_count).map(|_| AtomicBool::new(false)).collect(),
        }
    }

    fn try_claim(&self, subtree_rank: usize) -> bool {
        !self.is_claimed[subtree_rank].swap(true, Ordering::Relaxed)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellState {
    /// Neither in the polycube nor queued.
    Free,
    /// Queued as a candidate for extending the polycube.
    Untried,
    /// In the polycube, discarded by an ancestor, or forbidden.
    Closed,
}

#[derive(Clone, Copy)]
struct Cell {
    state: CellState,
    free_neighbour_count: u8,
    untried_neighbour_count: u8,
}

/// Sums over the untried cells `u` and the free cells `f`, kept up to date at
/// every state change so that the final extensions are counted in O(1).
///
/// `d(u)` is the number of free neighbours of `u`, `m(f)` the number of
/// untried neighbours of `f` and `e(f)` its number of free neighbours.
#[derive(Clone, Copy, Default)]
struct FrontierTallies {
    /// `sum d(u)`
    free_neighbour_total: u64,
    /// `sum C(d(u), 2)`
    free_neighbour_pair_total: u64,
    /// `sum C(m(f), 2)`
    shared_free_neighbour_pair_total: u64,
    /// `sum m(f) * e(f)`
    two_step_path_total: u64,
}

struct Enumerator<'claims> {
    cells: Vec<Cell>,
    tallies: FrontierTallies,
    /// Append-only queue of candidate cells; each recursion level owns a suffix.
    untried_cells: Vec<usize>,
    neighbour_offsets: [isize; NEIGHBOUR_COUNT],
    counts: Vec<u64>,
    max_cell_count: usize,
    /// `None` when this enumerator explores every subtree itself.
    claims: Option<&'claims SubtreeClaims>,
    next_subtree_rank: usize,
}

impl<'claims> Enumerator<'claims> {
    fn new(max_cell_count: usize, claims: Option<&'claims SubtreeClaims>) -> Self {
        let lattice = Lattice::new(max_cell_count);
        let origin = lattice.origin();
        let is_forbidden = lattice.forbidden_cells();
        let allowed_neighbour_counts = lattice.allowed_neighbour_counts(&is_forbidden);
        let cells = is_forbidden
            .iter()
            .zip(allowed_neighbour_counts)
            .map(|(&is_cell_forbidden, free_neighbour_count)| Cell {
                state: if is_cell_forbidden {
                    CellState::Closed
                } else {
                    CellState::Free
                },
                free_neighbour_count,
                untried_neighbour_count: 0,
            })
            .collect();
        let mut enumerator = Self {
            cells,
            tallies: FrontierTallies::default(),
            untried_cells: Vec::new(),
            neighbour_offsets: lattice.neighbour_offsets(),
            counts: vec![0; max_cell_count],
            max_cell_count,
            claims,
            next_subtree_rank: 0,
        };
        enumerator.queue(origin);
        enumerator
    }

    /// Extends a polycube of `placed_cell_count` cells with every untried
    /// cell from `first_untried` onwards.
    fn extend(&mut self, first_untried: usize, placed_cell_count: usize) {
        if placed_cell_count + CLOSED_FORM_CELL_COUNT == self.max_cell_count {
            self.count_final_extensions(first_untried, placed_cell_count);
            return;
        }
        let untried_end = self.untried_cells.len();
        let cell_count = placed_cell_count + 1;
        let tallies_before_level = self.tallies;
        for untried_index in first_untried..untried_end {
            let cell = self.untried_cells[untried_index];
            self.counts[cell_count - 1] += 1;
            // Closed for this subtree and for the following siblings.
            self.close(cell);
            // Only reachable when the maximum size is below the closed-form depth.
            if cell_count == self.max_cell_count {
                continue;
            }
            if cell_count == FORK_CELL_COUNT && !self.claims_next_subtree() {
                continue;
            }
            let tallies_before_subtree = self.tallies;
            self.queue_free_neighbours(cell);
            self.extend(untried_index + 1, cell_count);
            self.free_cells_queued_after(untried_end);
            self.tallies = tallies_before_subtree;
        }
        for untried_index in (first_untried..untried_end).rev() {
            self.reopen(self.untried_cells[untried_index]);
        }
        self.tallies = tallies_before_level;
    }

    fn claims_next_subtree(&mut self) -> bool {
        let Some(claims) = self.claims else {
            return true;
        };
        let subtree_rank = self.next_subtree_rank;
        self.next_subtree_rank += 1;
        claims.try_claim(subtree_rank)
    }

    /// Counts, without visiting them, every way to add one, two or three
    /// cells to the current polycube.
    ///
    /// A set of added cells is valid when each cell is untried, or free and
    /// linked to an untried cell of the set through free cells of the set.
    /// With `U` the untried cells and the notations of [`FrontierTallies`]:
    ///
    /// - one cell: `|U|`
    /// - two cells: `C(|U|, 2) + sum d(u)`
    /// - three cells: `C(|U|, 3)` (all untried)
    ///   `+ (|U| - 1) * sum d(u) - sum C(m(f), 2)` (two untried, one free)
    ///   `+ sum C(d(u), 2) + sum m(f) * e(f)` (one untried, two free; the
    ///   lattice is bipartite, so a free cell two steps away from `u` is
    ///   never also a neighbour of `u`).
    fn count_final_extensions(&mut self, first_untried: usize, placed_cell_count: usize) {
        let untried_count = (self.untried_cells.len() - first_untried) as u64;
        let tallies = self.tallies;
        let two_untried_one_free = untried_count.saturating_sub(1) * tallies.free_neighbour_total
            - tallies.shared_free_neighbour_pair_total;
        let one_untried_two_free = tallies.free_neighbour_pair_total + tallies.two_step_path_total;
        self.counts[placed_cell_count] += untried_count;
        self.counts[placed_cell_count + 1] +=
            choose_two(untried_count) + tallies.free_neighbour_total;
        self.counts[placed_cell_count + 2] +=
            choose_three(untried_count) + two_untried_one_free + one_untried_two_free;
    }

    fn queue_free_neighbours(&mut self, cell: usize) {
        for offset in self.neighbour_offsets {
            let neighbour = cell.wrapping_add_signed(offset);
            if self.cells[neighbour].state == CellState::Free {
                self.queue(neighbour);
            }
        }
    }

    /// Turns a free cell into an untried one.
    fn queue(&mut self, free_cell: usize) {
        let tallies = &mut self.tallies;
        let queued = &mut self.cells[free_cell];
        let free_neighbour_count = u64::from(queued.free_neighbour_count);
        let untried_neighbour_count = u64::from(queued.untried_neighbour_count);
        queued.state = CellState::Untried;
        tallies.shared_free_neighbour_pair_total -= choose_two(untried_neighbour_count);
        tallies.two_step_path_total -= untried_neighbour_count * free_neighbour_count;
        tallies.free_neighbour_total += free_neighbour_count;
        tallies.free_neighbour_pair_total += choose_two(free_neighbour_count);
        for offset in self.neighbour_offsets {
            let neighbour = &mut self.cells[free_cell.wrapping_add_signed(offset)];
            let untried_neighbours_before = u64::from(neighbour.untried_neighbour_count);
            neighbour.untried_neighbour_count += 1;
            neighbour.free_neighbour_count -= 1;
            let free_neighbours_after = u64::from(neighbour.free_neighbour_count);
            match neighbour.state {
                CellState::Free => {
                    tallies.shared_free_neighbour_pair_total += untried_neighbours_before;
                    tallies.two_step_path_total += free_neighbours_after;
                    tallies.two_step_path_total -= untried_neighbours_before;
                }
                CellState::Untried => {
                    tallies.free_neighbour_total -= 1;
                    tallies.free_neighbour_pair_total -= free_neighbours_after;
                }
                CellState::Closed => {}
            }
        }
        self.untried_cells.push(free_cell);
    }

    /// Undoes [`Self::queue`] for every cell queued after `untried_end`.
    /// Tallies are restored by the caller from a snapshot.
    fn free_cells_queued_after(&mut self, untried_end: usize) {
        for freed_cell in self.untried_cells.drain(untried_end..).rev() {
            self.cells[freed_cell].state = CellState::Free;
            for offset in self.neighbour_offsets {
                let neighbour = &mut self.cells[freed_cell.wrapping_add_signed(offset)];
                neighbour.untried_neighbour_count -= 1;
                neighbour.free_neighbour_count += 1;
            }
        }
    }

    /// Turns an untried cell into a closed one.
    fn close(&mut self, untried_cell: usize) {
        let tallies = &mut self.tallies;
        let closed = &mut self.cells[untried_cell];
        let free_neighbour_count = u64::from(closed.free_neighbour_count);
        closed.state = CellState::Closed;
        tallies.free_neighbour_total -= free_neighbour_count;
        tallies.free_neighbour_pair_total -= choose_two(free_neighbour_count);
        for offset in self.neighbour_offsets {
            let neighbour = &mut self.cells[untried_cell.wrapping_add_signed(offset)];
            neighbour.untried_neighbour_count -= 1;
            if neighbour.state == CellState::Free {
                tallies.shared_free_neighbour_pair_total -=
                    u64::from(neighbour.untried_neighbour_count);
                tallies.two_step_path_total -= u64::from(neighbour.free_neighbour_count);
            }
        }
    }

    /// Undoes [`Self::close`]. Tallies are restored by the caller from a snapshot.
    fn reopen(&mut self, closed_cell: usize) {
        self.cells[closed_cell].state = CellState::Untried;
        for offset in self.neighbour_offsets {
            let neighbour = &mut self.cells[closed_cell.wrapping_add_signed(offset)];
            neighbour.untried_neighbour_count += 1;
        }
    }
}

fn choose_two(item_count: u64) -> u64 {
    item_count * item_count.saturating_sub(1) / 2
}

fn choose_three(item_count: u64) -> u64 {
    choose_two(item_count) * item_count.saturating_sub(2) / 3
}
