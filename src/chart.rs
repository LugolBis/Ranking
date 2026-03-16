use csv::ReaderBuilder;
use plotters::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Record {
    alpha: f64,
    #[serde(rename = "percent_of_edges_removed")]
    edge_removed: f64,
    #[serde(rename = "stationnary_distrib_converge_time")]
    time: f64,
}

pub fn generate(data_path: &PathBuf, chart_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    // Read the CSV file
    let mut data_per_edge: HashMap<u64, Vec<(f64, f64)>> = HashMap::new();

    let file = File::open(data_path)?;
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(file);

    for result in reader.deserialize() {
        let record: Record = result?;
        data_per_edge
            .entry((record.edge_removed * 1000f64).floor() as u64)
            .or_insert_with(Vec::new)
            .push((record.alpha, record.time));
    }

    if data_per_edge.is_empty() {
        return Err("No data found.".into());
    }

    // Defined axis bounds
    let mut all_alpha = Vec::new();
    let mut all_times = Vec::new();
    for points in data_per_edge.values() {
        for (a, t) in points {
            all_alpha.push(*a);
            all_times.push(*t);
        }
    }
    let alpha_min = all_alpha.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let alpha_max = all_alpha.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let time_min = all_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let time_max = all_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let alpha_range = alpha_max - alpha_min;
    let time_range = time_max - time_min;
    let alpha_min = alpha_min - 0.02 * alpha_range;
    let alpha_max = alpha_max + 0.02 * alpha_range;
    let time_min = time_min - 0.02 * time_range;
    let time_max = time_max + 0.02 * time_range;

    // Draw creation
    let root = BitMapBackend::new(chart_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Stationary distribution converging time according to alpha",
            ("sans-serif", 25),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(alpha_min..alpha_max, time_min..time_max)?;

    chart
        .configure_mesh()
        .x_desc("Alpha")
        .y_desc("Converging time")
        .draw()?;

    // Color palette
    let palette = &Palette99::pick;

    // Sort keys
    let mut edges: Vec<_> = data_per_edge.keys().cloned().collect();
    edges.sort();

    for (idx, edge) in edges.iter().enumerate() {
        let mut points = data_per_edge[edge].clone();
        // Sort alpha
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let color = palette(idx);
        let shape = color.filled();

        chart
            .draw_series(LineSeries::new(points.iter().cloned(), shape.clone()))?
            .label(format!("Edge removed = {:}%", (*edge as f64) / 10f64))
            .legend(move |(x, y)| Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], shape));

        chart.draw_series(PointSeries::of_element(
            points.iter().cloned(),
            3,
            color.filled(),
            &|coord, size, style| {
                EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
            },
        ))?;
    }

    // Print legend
    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    println!(
        "Successfully generate the chart at '{}'",
        chart_path.display()
    );
    Ok(())
}
