//! How a growth fills space: its fractal dimension.

use polycubes_core::Coordinates;

/// Smallest mass the measure starts from: below, the lattice shows more
/// than the growth does.
const SMALLEST_MASS: usize = 40;

/// Masses the radius is measured at grow by that factor, so that they are
/// evenly spread on the logarithmic scale the fit is made on.
const MASS_GROWTH: f64 = 1.25;

/// Fewest masses a slope may be fitted on.
const FEWEST_MEASURES: usize = 4;

/// The fractal dimension of a growth, from its cells in the order they
/// came: the exponent `d` such that the first `m` cells spread over a
/// radius `r` with `m` proportional to `r^d`. A growth that fills space
/// has dimension 3, one that fills a plane 2, a line 1; branching growths
/// fall in between.
///
/// The radius is the radius of gyration, the root mean square distance of
/// the cells to their centre. The exponent is the slope fitted by least
/// squares on the logarithms. `None` when the cells are too few to
/// measure.
#[must_use]
pub fn growth_dimension(cells_in_arrival_order: &[Coordinates]) -> Option<f64> {
    let mut measures = Vec::new();
    let mut spread = Spread::default();
    let mut next_mass = SMALLEST_MASS;
    for (rank, &cell) in cells_in_arrival_order.iter().enumerate() {
        spread.add(cell);
        let mass = rank + 1;
        if mass == next_mass {
            measures.push((spread.radius_of_gyration().ln(), as_real(mass).ln()));
            next_mass = grown(mass);
        }
    }
    if measures.len() < FEWEST_MEASURES {
        return None;
    }
    Some(slope(&measures))
}

/// Running sums over cells, enough to give their radius of gyration.
#[derive(Default)]
struct Spread {
    cell_count: usize,
    coordinate_sums: [f64; 3],
    squared_distance_sum: f64,
}

impl Spread {
    fn add(&mut self, cell: Coordinates) {
        self.cell_count += 1;
        for (axis, &coordinate) in cell.iter().enumerate() {
            let coordinate = f64::from(coordinate);
            self.coordinate_sums[axis] += coordinate;
            self.squared_distance_sum += coordinate * coordinate;
        }
    }

    fn radius_of_gyration(&self) -> f64 {
        let cell_count = as_real(self.cell_count);
        let squared_centre_distance: f64 = self
            .coordinate_sums
            .iter()
            .map(|sum| (sum / cell_count).powi(2))
            .sum();
        (self.squared_distance_sum / cell_count - squared_centre_distance).sqrt()
    }
}

/// The next mass to measure at, at least one cell more.
fn grown(mass: usize) -> usize {
    let mut next_mass = mass + 1;
    while as_real(next_mass) < as_real(mass) * MASS_GROWTH {
        next_mass += 1;
    }
    next_mass
}

/// Slope of the line fitted by least squares through `(x, y)` points.
fn slope(points: &[(f64, f64)]) -> f64 {
    let point_count = as_real(points.len());
    let mean_x = points.iter().map(|(x, _)| x).sum::<f64>() / point_count;
    let mean_y = points.iter().map(|(_, y)| y).sum::<f64>() / point_count;
    let covariance: f64 = points
        .iter()
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum();
    let variance: f64 = points.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
    covariance / variance
}

/// Counts here stay far below what a real number holds exactly.
fn as_real(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}
