//! Redelmeier's algorithm, generalised to the cubic lattice.
//!
//! Every fixed polycube is generated exactly once by growing it from its
//! lexicographically smallest cell, placed at the origin. Cells that precede
//! the origin in (z, y, x) order are forbidden, which removes translations.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crate::lattice::{Lattice, NEIGHBOUR_COUNT};

mod cursor;
mod trace;

pub use cursor::EnumerationCursor;
pub use trace::{EnumerationStep, trace_enumeration};

/// Number of final cells counted in closed form instead of being enumerated.
const CLOSED_FORM_CELL_COUNT: usize = 4;
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

struct Enumerator<'claims> {
    cells: Vec<Cell>,
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
        for untried_index in first_untried..untried_end {
            let cell = self.untried_cells[untried_index];
            self.counts[cell_count - 1] += 1;
            // Closed for this subtree and for the following siblings.
            self.close(cell);
            // Only reachable when the walk starts below the closed-form depth.
            if cell_count == self.max_cell_count {
                continue;
            }
            if cell_count == FORK_CELL_COUNT && !self.claims_next_subtree() {
                continue;
            }
            self.queue_free_neighbours(cell);
            self.extend(untried_index + 1, cell_count);
            self.free_cells_queued_after(untried_end);
        }
        for untried_index in (first_untried..untried_end).rev() {
            self.reopen(self.untried_cells[untried_index]);
        }
    }

    fn claims_next_subtree(&mut self) -> bool {
        let Some(claims) = self.claims else {
            return true;
        };
        let subtree_rank = self.next_subtree_rank;
        self.next_subtree_rank += 1;
        claims.try_claim(subtree_rank)
    }

    /// Counts, without visiting them, every way to add one to four cells to
    /// the current polycube.
    fn count_final_extensions(&mut self, first_untried: usize, placed_cell_count: usize) {
        let extension_counts = self.frontier_sums(first_untried).extension_counts();
        let final_counts = &mut self.counts[placed_cell_count..];
        for (final_count, extension_count) in final_counts.iter_mut().zip(extension_counts) {
            *final_count += extension_count;
        }
    }

    fn frontier_sums(&self, first_untried: usize) -> FrontierSums {
        let untried_cells = &self.untried_cells[first_untried..];
        let mut sums = FrontierSums {
            untried_count: untried_cells.len() as i64,
            ..FrontierSums::default()
        };
        for &untried_cell in untried_cells {
            let untried = self.cells[untried_cell];
            sums.add_untried_cell(untried);
            for first_offset in self.neighbour_offsets {
                let free_cell = untried_cell.wrapping_add_signed(first_offset);
                let free = self.cells[free_cell];
                if free.state != CellState::Free {
                    continue;
                }
                sums.add_untried_free_edge(untried, free);
                for second_offset in self.neighbour_offsets {
                    let path_end_cell = free_cell.wrapping_add_signed(second_offset);
                    let path_end = self.cells[path_end_cell];
                    let is_straight = second_offset == first_offset;
                    // When the path turns, it spans a unit square whose fourth
                    // corner is the other neighbour of the untried cell.
                    let fourth_corner = self.cells[untried_cell.wrapping_add_signed(second_offset)];
                    let is_fourth_corner_free = fourth_corner.state == CellState::Free;
                    match path_end.state {
                        CellState::Free => {
                            sums.add_two_step_path(path_end);
                            if !is_straight && is_fourth_corner_free {
                                sums.twice_single_untried_square_count += 1;
                            }
                        }
                        CellState::Untried => {
                            let is_turning = !is_straight && path_end_cell != untried_cell;
                            if is_turning && is_fourth_corner_free {
                                sums.four_times_double_untried_square_count += 1;
                            }
                        }
                        CellState::Closed => {}
                    }
                }
            }
        }
        sums
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
        self.cells[free_cell].state = CellState::Untried;
        for offset in self.neighbour_offsets {
            let neighbour = &mut self.cells[free_cell.wrapping_add_signed(offset)];
            neighbour.untried_neighbour_count += 1;
            neighbour.free_neighbour_count -= 1;
        }
        self.untried_cells.push(free_cell);
    }

    /// Undoes [`Self::queue`] for every cell queued after `untried_end`.
    fn free_cells_queued_after(&mut self, untried_end: usize) {
        for freed_cell in self.untried_cells.drain(untried_end..) {
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
        self.cells[untried_cell].state = CellState::Closed;
        for offset in self.neighbour_offsets {
            self.cells[untried_cell.wrapping_add_signed(offset)].untried_neighbour_count -= 1;
        }
    }

    /// Undoes [`Self::close`].
    fn reopen(&mut self, closed_cell: usize) {
        self.cells[closed_cell].state = CellState::Untried;
        for offset in self.neighbour_offsets {
            self.cells[closed_cell.wrapping_add_signed(offset)].untried_neighbour_count += 1;
        }
    }
}

/// Sums over the frontier of a polycube from which the number of ways to add
/// up to four cells is derived.
///
/// Notations: `U` the untried cells, `d(u)` the number of free neighbours of
/// the untried cell `u`, `m(f)` the number of untried neighbours of the free
/// cell `f`, `e(f)` its number of free neighbours. An "edge" is an untried
/// cell with one of its free neighbours; a "path" is an edge prolonged by a
/// free neighbour of its free cell.
#[derive(Default)]
struct FrontierSums {
    /// `|U|`
    untried_count: i64,
    /// `sum d(u)`, which is also `sum m(f)`
    edge_count: i64,
    /// `sum C(d(u), 2)`
    free_pairs_around_untried: i64,
    /// `sum C(d(u), 3)`
    free_triples_around_untried: i64,
    /// `sum over edges of m(f) - 1`, that is `2 * sum C(m(f), 2)`
    twice_untried_pairs_around_free: i64,
    /// `sum over edges of C(m(f) - 1, 2)`, that is `3 * sum C(m(f), 3)`
    thrice_untried_triples_around_free: i64,
    /// `sum over edges of e(f)`, that is `sum m(f) * e(f)`
    path_count: i64,
    /// `sum over edges of C(e(f), 2)`
    forked_path_count: i64,
    /// `sum over edges of e(f) * (m(f) - 1)`
    twice_doubly_rooted_path_count: i64,
    /// `sum over edges of d(u) * m(f)`
    edge_degree_product_total: i64,
    /// `sum over edges of d(u) * e(f)`
    edge_free_degree_product_total: i64,
    /// `sum over paths of e(g)`, `g` being the far end of the path
    path_end_free_neighbour_total: i64,
    /// `sum over paths of m(g)`
    path_end_untried_neighbour_total: i64,
    /// Unit squares with exactly one untried corner and three free ones, counted twice.
    twice_single_untried_square_count: i64,
    /// Unit squares with two opposite untried corners and two free ones, counted four times.
    four_times_double_untried_square_count: i64,
}

impl FrontierSums {
    fn add_untried_cell(&mut self, untried: Cell) {
        let free_neighbour_count = i64::from(untried.free_neighbour_count);
        self.edge_count += free_neighbour_count;
        self.free_pairs_around_untried += choose_two(free_neighbour_count);
        self.free_triples_around_untried += choose_three(free_neighbour_count);
    }

    fn add_untried_free_edge(&mut self, untried: Cell, free: Cell) {
        let untried_degree = i64::from(untried.free_neighbour_count);
        let other_untried_count = i64::from(free.untried_neighbour_count) - 1;
        let free_neighbour_count = i64::from(free.free_neighbour_count);
        self.twice_untried_pairs_around_free += other_untried_count;
        self.thrice_untried_triples_around_free += choose_two(other_untried_count);
        self.path_count += free_neighbour_count;
        self.forked_path_count += choose_two(free_neighbour_count);
        self.twice_doubly_rooted_path_count += free_neighbour_count * other_untried_count;
        self.edge_degree_product_total += untried_degree * (other_untried_count + 1);
        self.edge_free_degree_product_total += untried_degree * free_neighbour_count;
    }

    fn add_two_step_path(&mut self, path_end: Cell) {
        self.path_end_free_neighbour_total += i64::from(path_end.free_neighbour_count);
        self.path_end_untried_neighbour_total += i64::from(path_end.untried_neighbour_count);
    }

    /// Number of ways to add one, two, three and four cells.
    ///
    /// A set of added cells is valid when each cell is untried, or free and
    /// linked to an untried cell of the set through free cells of the set.
    /// Sets are counted by number of untried cells they contain. The lattice
    /// is bipartite: two cells two steps apart are never neighbours, which
    /// keeps the cases below disjoint.
    fn extension_counts(&self) -> [u64; CLOSED_FORM_CELL_COUNT] {
        self.extension_counts_by_free_cell_count()
            .map(|counts_by_free_cell_count| counts_by_free_cell_count.iter().sum())
    }

    /// Ways to add one to four cells, split by how many of them are free
    /// cells rather than untried ones: row `added - 1`, column free cells.
    fn extension_counts_by_free_cell_count(
        &self,
    ) -> [[u64; CLOSED_FORM_CELL_COUNT]; CLOSED_FORM_CELL_COUNT] {
        let untried_count = self.untried_count;
        // Pairs of untried cells with a free neighbour of either.
        let two_untried_one_free =
            (untried_count - 1) * self.edge_count - self.untried_pairs_around_free();
        // Two free neighbours of the untried cell, or a path.
        let one_untried_two_free = self.free_pairs_around_untried + self.path_count;
        [
            [untried_count, 0, 0, 0],
            [choose_two(untried_count), self.edge_count, 0, 0],
            [
                choose_three(untried_count),
                two_untried_one_free,
                one_untried_two_free,
                0,
            ],
            [
                choose_four(untried_count),
                self.three_untried_one_free_count(),
                self.two_untried_two_free_count(),
                self.one_untried_three_free_count(),
            ],
        ]
        .map(|counts| {
            counts.map(|extension_count| {
                u64::try_from(extension_count).expect("extension counts are never negative")
            })
        })
    }

    fn untried_pairs_around_free(&self) -> i64 {
        self.twice_untried_pairs_around_free / 2
    }

    /// Triples of untried cells hitting the untried neighbours of a free
    /// cell, by inclusion-exclusion on those neighbours.
    fn three_untried_one_free_count(&self) -> i64 {
        choose_two(self.untried_count - 1) * self.edge_count
            - (self.untried_count - 2) * self.untried_pairs_around_free()
            + self.thrice_untried_triples_around_free / 3
    }

    fn two_untried_two_free_count(&self) -> i64 {
        let untried_count = self.untried_count;
        let untried_degree_square_total = self.twice_untried_pairs_around_free + self.edge_count;
        let double_untried_square_count = self.four_times_double_untried_square_count / 4;
        // Both free cells next to an untried cell of the pair: for free cells
        // with a, b untried neighbours of which c are shared, the number of
        // untried pairs hitting both is a*b - c*(a+b) + c*(2|U|-1+c)/2.
        let both_free_next_to_untried =
            (self.edge_count * self.edge_count - untried_degree_square_total) / 2
                - (self.edge_degree_product_total - untried_degree_square_total)
                + untried_count * self.free_pairs_around_untried
                + double_untried_square_count;
        // One free cell next to the pair, the other only next to that free
        // cell: a*(|U|-1) - C(a,2) - a*b untried pairs per ordered free pair.
        let free_cells_in_a_row = (untried_count - 1) * self.path_count
            - self.twice_doubly_rooted_path_count / 2
            - self.path_end_untried_neighbour_total;
        both_free_next_to_untried + free_cells_in_a_row
    }

    fn one_untried_three_free_count(&self) -> i64 {
        let single_untried_square_count = self.twice_single_untried_square_count / 2;
        // Two free neighbours of the untried cell, plus a free neighbour of
        // either; a square's far corner is a neighbour of both.
        let two_branches_one_prolonged =
            self.edge_free_degree_product_total - self.path_count - single_untried_square_count;
        // One free neighbour of the untried cell, then a three-cell path that
        // does not come back next to the untried cell through a square.
        let three_cell_paths =
            self.path_end_free_neighbour_total - self.path_count - 2 * single_untried_square_count;
        self.free_triples_around_untried
            + two_branches_one_prolonged
            + self.forked_path_count
            + three_cell_paths
    }
}

fn choose_two(item_count: i64) -> i64 {
    item_count * (item_count - 1) / 2
}

fn choose_three(item_count: i64) -> i64 {
    choose_two(item_count) * (item_count - 2) / 3
}

fn choose_four(item_count: i64) -> i64 {
    choose_three(item_count) * (item_count - 3) / 4
}
