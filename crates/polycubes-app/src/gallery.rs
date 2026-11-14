//! A sample of uniform draws sorted by compactness, for the frontend.

use polycubes_sampling::{exposed_face_count, grow_by_random_neighbour};
use serde::Serialize;

use crate::drawing::{CellCount, DrawError, PolycubeDrawer};
use crate::growth::GrownPolycube;

/// Largest size sampled: beyond it the sample takes over a second.
pub const MAX_GALLERY_CELL_COUNT: usize = 10;

/// Polycubes drawn by each process to fill the histograms.
pub const SAMPLE_COUNT: usize = 200;

/// Polycubes of the uniform sample shown in 3D; each one needs its own
/// drawing surface, and a page may only hold a few.
pub const SHOWN_COUNT: usize = 8;

/// Histogram bins, each a tenth of the fill rates from empty to full.
pub const BIN_COUNT: usize = 10;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GalleryError {
    #[error("cell count must be between 1 and {MAX_GALLERY_CELL_COUNT}, got {0}")]
    CellCountOutOfRange(usize),
    #[error(transparent)]
    Draw(#[from] DrawError),
}

/// How full the bounding boxes are, for uniform draws and for polycubes
/// grown by random neighbour, with some of the uniform draws.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gallery {
    /// Uniform draws from the most compact to the least compact, evenly
    /// spread over the sorted sample.
    pub shown: Vec<GrownPolycube>,
    /// Polycubes per bin of fill rate: bin `i` holds the rates from
    /// `i / BIN_COUNT` up to the next bin, the last one including full boxes.
    pub uniform_histogram: Vec<u32>,
    pub growth_histogram: Vec<u32>,
    pub exposed_faces: ExposedFaceHistograms,
    pub sample_count: usize,
}

/// Polycubes per number of exposed faces, a second measure of compactness:
/// the fewer faces outside, the more compact. Entry `i` counts the polycubes
/// showing `smallest_face_count + 2 * i` faces; the last entry is the most
/// faces a polycube of that size can show, `4n + 2`, reached when it has no
/// loop.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExposedFaceHistograms {
    pub smallest_face_count: usize,
    pub uniform: Vec<u32>,
    pub growth: Vec<u32>,
}

/// Draws both samples; `choose_rank` and `choose_candidate` stand for the
/// random choices of the uniform draws and of the neighbour growths.
///
/// # Errors
///
/// Returns [`GalleryError::CellCountOutOfRange`] if `cell_count` is zero or
/// exceeds [`MAX_GALLERY_CELL_COUNT`], and [`GalleryError::Draw`] if a
/// chosen rank is not below the number of polycubes.
pub fn build_gallery(
    drawer: &mut PolycubeDrawer,
    cell_count: usize,
    mut choose_rank: impl FnMut(u64) -> u64,
    mut choose_candidate: impl FnMut(usize) -> usize,
) -> Result<Gallery, GalleryError> {
    if cell_count > MAX_GALLERY_CELL_COUNT {
        return Err(GalleryError::CellCountOutOfRange(cell_count));
    }
    let drawn_cell_count = CellCount::try_from(cell_count)
        .map_err(|_| GalleryError::CellCountOutOfRange(cell_count))?;
    let mut uniform_sample = Vec::with_capacity(SAMPLE_COUNT);
    let mut growth_sample = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let drawn = drawer.draw(drawn_cell_count, &mut choose_rank)?;
        uniform_sample.push(GrownPolycube::from_cells(drawn.cells));
        let grown = grow_by_random_neighbour(cell_count, &mut choose_candidate);
        growth_sample.push(GrownPolycube::from_cells(grown));
    }
    Ok(Gallery {
        uniform_histogram: fill_rate_histogram(&uniform_sample),
        growth_histogram: fill_rate_histogram(&growth_sample),
        exposed_faces: ExposedFaceHistograms::of(cell_count, &uniform_sample, &growth_sample),
        sample_count: SAMPLE_COUNT,
        shown: spread_by_compactness(uniform_sample),
    })
}

/// Two faces vanish at each contact between cubes, so face counts of
/// polycubes of the same size differ by multiples of this step.
const FACE_COUNT_STEP: usize = 2;

impl ExposedFaceHistograms {
    fn of(
        cell_count: usize,
        uniform_sample: &[GrownPolycube],
        growth_sample: &[GrownPolycube],
    ) -> Self {
        let uniform_face_counts = face_counts(uniform_sample);
        let growth_face_counts = face_counts(growth_sample);
        // A polycube without loop has one contact less than cubes.
        let largest_face_count = 4 * cell_count + 2;
        let smallest_face_count = uniform_face_counts
            .iter()
            .chain(&growth_face_counts)
            .copied()
            .min()
            .unwrap_or(largest_face_count);
        let bin_count = (largest_face_count - smallest_face_count) / FACE_COUNT_STEP + 1;
        let histogram = |face_counts: &[usize]| {
            let mut histogram = vec![0; bin_count];
            for face_count in face_counts {
                histogram[(face_count - smallest_face_count) / FACE_COUNT_STEP] += 1;
            }
            histogram
        };
        Self {
            smallest_face_count,
            uniform: histogram(&uniform_face_counts),
            growth: histogram(&growth_face_counts),
        }
    }
}

fn face_counts(sample: &[GrownPolycube]) -> Vec<usize> {
    sample
        .iter()
        .map(|polycube| exposed_face_count(&polycube.cells))
        .collect()
}

fn box_volume(polycube: &GrownPolycube) -> usize {
    polycube.bounding_box.iter().product()
}

fn fill_rate_histogram(sample: &[GrownPolycube]) -> Vec<u32> {
    let mut histogram = vec![0; BIN_COUNT];
    for polycube in sample {
        // A full box would land one bin too far: it joins the last one.
        let bin = (polycube.cells.len() * BIN_COUNT / box_volume(polycube)).min(BIN_COUNT - 1);
        histogram[bin] += 1;
    }
    histogram
}

/// For a given number of cells, the smaller the box the more compact the
/// polycube. Keeps [`SHOWN_COUNT`] polycubes evenly spaced in that order,
/// both extremes included.
fn spread_by_compactness(mut sample: Vec<GrownPolycube>) -> Vec<GrownPolycube> {
    sample.sort_by_key(box_volume);
    let last_rank = sample.len() - 1;
    let mut kept_ranks =
        (0..SHOWN_COUNT).map(|shown_index| shown_index * last_rank / (SHOWN_COUNT - 1));
    let mut next_kept_rank = kept_ranks.next();
    let mut shown = Vec::with_capacity(SHOWN_COUNT);
    for (rank, polycube) in sample.into_iter().enumerate() {
        if next_kept_rank == Some(rank) {
            shown.push(polycube);
            next_kept_rank = kept_ranks.next();
        }
    }
    shown
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLED_CELL_COUNT: usize = 5;

    /// Ranks spread over all polycubes, so that the sample is varied.
    fn spread_ranks() -> impl FnMut(u64) -> u64 {
        let mut draw_index = 0;
        move |polycube_count| {
            draw_index += 1;
            (draw_index * 37) % polycube_count
        }
    }

    fn sample_gallery() -> Gallery {
        build_gallery(
            &mut PolycubeDrawer::default(),
            SAMPLED_CELL_COUNT,
            spread_ranks(),
            |_| 0,
        )
        .expect("five cells can be sampled")
    }

    #[test]
    fn histograms_hold_every_sampled_polycube() {
        let gallery = sample_gallery();

        assert_eq!(gallery.sample_count, SAMPLE_COUNT);
        for histogram in [&gallery.uniform_histogram, &gallery.growth_histogram] {
            assert_eq!(histogram.len(), BIN_COUNT);
            assert_eq!(histogram.iter().sum::<u32>() as usize, SAMPLE_COUNT);
        }
    }

    #[test]
    fn identical_growths_fall_in_a_single_bin() {
        let gallery = sample_gallery();

        // Always taking the first candidate grows the same polycube again.
        let filled_bin_count = gallery
            .growth_histogram
            .iter()
            .filter(|&&polycube_count| polycube_count > 0)
            .count();
        assert_eq!(filled_bin_count, 1);
    }

    #[test]
    fn shown_polycubes_go_from_most_to_least_compact() {
        let gallery = sample_gallery();

        assert_eq!(gallery.shown.len(), SHOWN_COUNT);
        for polycube in &gallery.shown {
            assert_eq!(polycube.cells.len(), SAMPLED_CELL_COUNT);
        }
        let volumes: Vec<usize> = gallery.shown.iter().map(box_volume).collect();
        assert!(volumes.is_sorted(), "{volumes:?}");
        assert!(volumes[0] < volumes[SHOWN_COUNT - 1], "{volumes:?}");
    }

    #[test]
    fn a_full_box_lands_in_the_last_bin() {
        // The single cube is the only polycube of its size: a full box.
        let gallery = build_gallery(&mut PolycubeDrawer::default(), 1, |_| 0, |_| 0)
            .expect("one cell can be sampled");

        assert_eq!(
            gallery.uniform_histogram[BIN_COUNT - 1] as usize,
            SAMPLE_COUNT
        );
        assert_eq!(
            gallery.growth_histogram[BIN_COUNT - 1] as usize,
            SAMPLE_COUNT
        );
    }

    #[test]
    fn exposed_face_histograms_hold_every_sampled_polycube() {
        let gallery = sample_gallery();

        let exposed_faces = &gallery.exposed_faces;
        assert_eq!(exposed_faces.uniform.len(), exposed_faces.growth.len());
        assert_eq!(
            exposed_faces.uniform.iter().sum::<u32>() as usize,
            SAMPLE_COUNT
        );
        assert_eq!(
            exposed_faces.growth.iter().sum::<u32>() as usize,
            SAMPLE_COUNT
        );
    }

    #[test]
    fn last_exposed_face_bin_is_the_polycube_without_loop() {
        let gallery = sample_gallery();

        let exposed_faces = &gallery.exposed_faces;
        let largest_face_count =
            exposed_faces.smallest_face_count + 2 * (exposed_faces.uniform.len() - 1);
        assert_eq!(largest_face_count, 4 * SAMPLED_CELL_COUNT + 2);
        // Most polycubes of five cells have no loop.
        assert!(exposed_faces.uniform.last().is_some_and(|&count| count > 0));
    }

    #[test]
    fn two_cubes_always_show_ten_faces() {
        let gallery = build_gallery(&mut PolycubeDrawer::default(), 2, |_| 0, |_| 0)
            .expect("two cells can be sampled");

        assert_eq!(
            gallery.exposed_faces,
            ExposedFaceHistograms {
                smallest_face_count: 10,
                uniform: vec![200],
                growth: vec![200],
            }
        );
    }

    #[test]
    fn refuses_sizes_outside_the_sampled_range() {
        for cell_count in [0, MAX_GALLERY_CELL_COUNT + 1] {
            let gallery = build_gallery(&mut PolycubeDrawer::default(), cell_count, |_| 0, |_| 0);

            assert_eq!(gallery, Err(GalleryError::CellCountOutOfRange(cell_count)));
        }
    }

    #[test]
    fn reports_a_rank_beyond_the_last_polycube() {
        let gallery = build_gallery(
            &mut PolycubeDrawer::default(),
            SAMPLED_CELL_COUNT,
            |polycube_count| polycube_count,
            |_| 0,
        );

        assert_eq!(
            gallery,
            Err(GalleryError::Draw(DrawError::RankOutOfRange {
                rank: 534,
                polycube_count: 534
            }))
        );
    }
}
