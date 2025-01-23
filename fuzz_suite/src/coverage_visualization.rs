use plotters::prelude::*;
use std::path::Path;

pub fn plot_coverage(
    coverage_data: &[f64],
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_coverage = coverage_data.iter().cloned().fold(0./0., f64::max);
    let min_coverage = 0.0;

    let mut chart = ChartBuilder::on(&root)
        .caption("Code Coverage Over Time", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(
            0..coverage_data.len(),
            min_coverage..max_coverage,
        )?;

    chart
        .configure_mesh()
        .x_desc("Number of Test Cases")
        .y_desc("Coverage (%)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        coverage_data.iter().enumerate().map(|(x, y)| (x, *y)),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}

pub fn plot_cumulative_coverage(
    coverages: &[(usize, f64)],
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_x = coverages.last().map(|(x, _)| *x).unwrap_or(100);
    let max_y = 100.0;

    let mut chart = ChartBuilder::on(&root)
        .caption("Cumulative Code Coverage", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(
            0..max_x,
            0.0..max_y,
        )?;

    chart
        .configure_mesh()
        .x_desc("Number of Test Cases")
        .y_desc("Cumulative Coverage (%)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        coverages.iter().map(|(x, y)| (*x, *y)),
        &BLUE,
    ))?;

    root.present()?;
    Ok(())
}
