use charming::{
    Chart,
    HtmlRenderer,
    ImageRenderer,
    component::{Axis, Title},
    element::AxisType,
    series::Line,
    theme::Theme,
};
use derive_more::Constructor;

#[derive(Default, Constructor)]
pub struct PlotBase {}

// use clap::{Arg, Command};
// use polars::prelude::*;

// use rta::read_from_file;

// const OUTPUT_DIR: &str = "charts";

// fn main() -> PolarsResult<()> {
//     let matches = Command::new("read_file")
//         .arg(
//             Arg::new("source")
//                 .short('s')
//                 .long("source")
//                 .value_name("SOURCE_FILE")
//                 .help("Sets the source file to use")
//                 .required(true),
//         )
//         .get_matches();

//     let path = matches
//         .get_one::<String>("source")
//         .expect("Unable to parse source path...");
//     println!("Reading data from file: {}", path);

//     let mut df = read_from_file(path)?;
//     println!("Data loaded. Number of rows: {}", df.height());
//     // println!("Columns: {:?}", df.get_column_names());

//     println!("Sorting data by ActionDay and UpdateTime...");
//     df.sort_in_place(
//         vec!["ActionDay", "UpdateTime"],
//         SortMultipleOptions {
//             // ascending order for both columns
//             descending: vec![false, false],
//             // ascending order for both columns
//             nulls_last: vec![false, false],
//             multithreaded: true,
//             maintain_order: false,
//             limit: None,
//         },
//     )?;

//     // Compose x-axis from "ActionDay" and "UpdateTime", y-axis from "LastPrice"
//     let action_day = df
//         .column("ActionDay")?
//         .str()
//         .expect("ActionDay is not Utf8 type");
//     let update_time = df
//         .column("UpdateTime")?
//         .str()
//         .expect("UpdateTime is not Utf8 type");
//     let last_price = df
//         .column("LastPrice")?
//         .f64()
//         .expect("LastPrice is not f64 type");

//     let n = df.height();

//     // Pick one every 120 records
//     let step = 120;
//     let mut x: Vec<String> = Vec::new();
//     let mut y: Vec<f64> = Vec::new();

//     println!("Sampling every {} records for plotting...", step);
//     for i in (0..n).step_by(step) {
//         let day = action_day.get(i).unwrap_or("");
//         let time = update_time.get(i).unwrap_or("");
//         x.push(format!("{}_{}", day, time));
//         y.push(last_price.get(i).unwrap_or(f64::NAN));
//     }
//     println!("Sampled {} points for the plot.", x.len());

//     println!("Generating charming plot...");
//     // Determine min and max for y-axis, ignoring NaN values
//     let (y_min, y_max) = y
//         .iter()
//         .filter_map(|v| if v.is_finite() { Some(*v) } else { None })
//         .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
//             (min.min(v), max.max(v))
//         });

//     // Add margin to y-axis range (e.g., 5% of the range)
//     let margin = ((y_max - y_min) * 0.05).max(1e-8); // avoid zero margin
//     let y_axis_min = y_min - margin;
//     let y_axis_max = y_max + margin;

//     // Dynamically adjust chart width based on number of x labels (minimum 1000, max 4000)
//     let base_width = 1000;
//     let width_per_label = 10;
//     let chart_width = (base_width + x.len() * width_per_label).min(4000);

//     let chart = Chart::new()
//         .title(Title::new().text(format!("LastPrice Timeseries (1 every {} records)", step)))
//         .x_axis(
//             Axis::new()
//                 .type_(AxisType::Category)
//                 .name("ActionDay_UpdateTime")
//                 .data(x.clone()),
//         )
//         .y_axis(
//             Axis::new()
//                 .name("LastPrice")
//                 .min(y_axis_min)
//                 .max(y_axis_max),
//         )
//         .series(Line::new().data(y.iter().cloned().collect::<Vec<f64>>()));

//     let file_stem = std::path::Path::new(path)
//         .file_stem()
//         .and_then(|s| s.to_str())
//         .unwrap_or("output");
//     let mut renderer = ImageRenderer::new(chart_width as u32, 800).theme(Theme::Vintage);
//     renderer
//         .save(&chart, format!("{}/{}.svg", OUTPUT_DIR, file_stem))
//         .expect("Failed to save charming plot");

//     Ok(())
// }
