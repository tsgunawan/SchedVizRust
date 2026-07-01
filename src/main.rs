#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod scheduler_core;

use eframe::egui;
use scheduler_core::{
    Cores, Migration, MulticoreAlgorithm, MulticoreConfig, MulticoreScheduleResult, Process,
    Quantum, ScheduleResult, run_selected, schedule_multicore,
};
use std::collections::HashMap;

const APP_AUTHOR: &str =
    "Prof. Ir. Ts. Dr. Teddy Surya Gunawan, International Islamic University Malaysia";

#[derive(Clone, Copy, PartialEq, Eq)]
enum SchedulingMode {
    Uniprocessor,
    Multicore,
}

enum CalcResults {
    Uni(HashMap<String, ScheduleResult>),
    Multi(MulticoreScheduleResult),
}

#[derive(Clone)]
struct ProcessRow {
    name: String,
    arrival: String,
    service: String,
}

struct SchedVizApp {
    processes: Vec<ProcessRow>,
    mode: SchedulingMode,
    algorithm_selection: HashMap<String, bool>,
    rr_quantum: u32,
    multicore_cores: Cores,
    multicore_migration: Migration,
    multicore_algorithm: MulticoreAlgorithm,
    results: Option<Result<CalcResults, String>>,
    dirty: bool,
    chart_metrics_scroll: f32,
    selected_example: Option<u32>,
}

impl Default for SchedVizApp {
    fn default() -> Self {
        let mut app = Self {
            processes: Vec::new(),
            mode: SchedulingMode::Uniprocessor,
            algorithm_selection: [
                ("FCFS".to_string(), true),
                ("RR".to_string(), true),
                ("SPN".to_string(), true),
                ("SRT".to_string(), true),
                ("HRRN".to_string(), true),
            ]
            .iter()
            .cloned()
            .collect(),
            rr_quantum: 1,
            multicore_cores: Cores::Two,
            multicore_migration: Migration::None,
            multicore_algorithm: MulticoreAlgorithm::Fcfs,
            results: None,
            dirty: true,
            chart_metrics_scroll: 0.0,
            selected_example: None,
        };
        app.load_example(0);
        app.calculate();
        app
    }
}

impl SchedVizApp {
    fn load_example(&mut self, n: u32) {
        let (rows, q): (&[(&str, &str, &str)], u32) = match n {
            // Stallings textbook example
            0 => (
                &[
                    ("A", "0", "3"),
                    ("B", "2", "6"),
                    ("C", "4", "4"),
                    ("D", "6", "5"),
                    ("E", "8", "2"),
                ],
                1,
            ),
            1 => (&[("A", "0", "5"), ("B", "2", "3"), ("C", "5", "7")], 1),
            2 => (&[("A", "0", "8"), ("B", "1", "4"), ("C", "6", "8")], 2),
            3 => (
                &[
                    ("A", "0", "4"),
                    ("B", "2", "6"),
                    ("C", "4", "2"),
                    ("D", "7", "3"),
                ],
                4,
            ),
            4 => (
                &[
                    ("A", "0", "6"),
                    ("B", "3", "2"),
                    ("C", "5", "8"),
                    ("D", "9", "4"),
                ],
                1,
            ),
            5 => (
                &[
                    ("A", "0", "9"),
                    ("B", "2", "5"),
                    ("C", "6", "7"),
                    ("D", "10", "4"),
                ],
                2,
            ),
            6 => (
                &[
                    ("A", "0", "3"),
                    ("B", "1", "7"),
                    ("C", "4", "2"),
                    ("D", "6", "6"),
                    ("E", "8", "2"),
                ],
                4,
            ),
            7 => (
                &[
                    ("A", "0", "5"),
                    ("B", "2", "9"),
                    ("C", "3", "3"),
                    ("D", "7", "6"),
                    ("E", "11", "2"),
                ],
                1,
            ),
            8 => (
                &[
                    ("A", "0", "10"),
                    ("B", "4", "4"),
                    ("C", "5", "8"),
                    ("D", "9", "5"),
                    ("E", "12", "3"),
                ],
                2,
            ),
            9 => (
                &[
                    ("A", "0", "4"),
                    ("B", "1", "6"),
                    ("C", "3", "3"),
                    ("D", "5", "7"),
                    ("E", "8", "2"),
                    ("F", "10", "3"),
                ],
                4,
            ),
            10 => (
                &[
                    ("A", "0", "7"),
                    ("B", "2", "5"),
                    ("C", "4", "8"),
                    ("D", "6", "3"),
                    ("E", "9", "4"),
                    ("F", "13", "3"),
                ],
                1,
            ),
            _ => return,
        };

        self.processes = rows
            .iter()
            .map(|(name, arrival, service)| ProcessRow {
                name: (*name).to_string(),
                arrival: (*arrival).to_string(),
                service: (*service).to_string(),
            })
            .collect();
        self.rr_quantum = q;
        self.dirty = true;
        self.selected_example = Some(n);
    }

    fn calculate(&mut self) {
        let mut parsed_processes = Vec::new();
        for (idx, row) in self.processes.iter().enumerate() {
            let name = row.name.trim();
            if name.is_empty() {
                self.results = Some(Err("Process name cannot be empty.".to_string()));
                return;
            }
            let arrival = match row.arrival.trim().parse::<f64>() {
                Ok(val) => val,
                Err(_) => {
                    self.results = Some(Err(format!(
                        "Invalid arrival time '{}' for process {}.",
                        row.arrival, name
                    )));
                    return;
                }
            };
            let service = match row.service.trim().parse::<f64>() {
                Ok(val) => val,
                Err(_) => {
                    self.results = Some(Err(format!(
                        "Invalid service time '{}' for process {}.",
                        row.service, name
                    )));
                    return;
                }
            };
            parsed_processes.push(Process {
                name: name.to_string(),
                arrival,
                service,
                order: idx,
            });
        }

        match self.mode {
            SchedulingMode::Uniprocessor => {
                let selected_algos: Vec<String> = self
                    .algorithm_selection
                    .iter()
                    .filter(|&(_, &selected)| selected)
                    .map(|(name, _)| name.clone())
                    .collect();

                if selected_algos.is_empty() {
                    self.results =
                        Some(Err("Select at least one scheduling algorithm.".to_string()));
                    return;
                }

                match run_selected(&parsed_processes, &selected_algos, self.rr_quantum) {
                    Ok(res) => {
                        self.results = Some(Ok(CalcResults::Uni(res)));
                        self.dirty = false;
                        self.chart_metrics_scroll = 0.0;
                    }
                    Err(err) => {
                        self.results = Some(Err(err));
                    }
                }
            }
            SchedulingMode::Multicore => {
                let Some(quantum) = Quantum::new(self.rr_quantum) else {
                    self.results = Some(Err(format!(
                        "Quantum must be between 1 and 6, got {}.",
                        self.rr_quantum
                    )));
                    return;
                };
                let config = MulticoreConfig {
                    cores: self.multicore_cores,
                    algorithm: self.multicore_algorithm,
                    quantum,
                    migration: self.multicore_migration,
                };
                match schedule_multicore(&parsed_processes, config) {
                    Ok(res) => {
                        self.results = Some(Ok(CalcResults::Multi(res)));
                        self.dirty = false;
                        self.chart_metrics_scroll = 0.0;
                    }
                    Err(err) => {
                        self.results = Some(Err(err));
                    }
                }
            }
        }
    }

    fn export_csv(&self) {
        match &self.results {
            Some(Ok(CalcResults::Uni(results))) => self.export_uni_csv(results),
            Some(Ok(CalcResults::Multi(result))) => self.export_multi_csv(result),
            _ => {
                rfd::MessageDialog::new()
                    .set_title("Export CSV")
                    .set_description("Calculate a valid schedule first.")
                    .set_level(rfd::MessageLevel::Warning)
                    .show();
            }
        }
    }

    fn export_uni_csv(&self, results: &HashMap<String, ScheduleResult>) {
        let file_path = rfd::FileDialog::new()
            .set_title("Export CSV")
            .set_file_name("schedule_unicore_metrics.csv")
            .add_filter("CSV File", &["csv"])
            .save_file();

        let Some(path) = file_path else {
            return;
        };

        let mut writer = match csv::Writer::from_path(&path) {
            Ok(w) => w,
            Err(e) => {
                Self::csv_error_dialog(&e);
                return;
            }
        };

        let _ = writer.write_record(&[
            "Algorithm",
            "Process",
            "Arrival Time",
            "Service Time",
            "Finish Time",
            "Turnaround Time",
            "Normalized Turnaround Time",
            "Average Normalized Turnaround Time",
        ]);

        let process_lookup: HashMap<String, &ProcessRow> =
            self.processes.iter().map(|p| (p.name.clone(), p)).collect();

        let mut sorted_keys: Vec<&String> = results.keys().collect();
        sorted_keys.sort_by_key(|key| algo_priority(key));

        for algo in sorted_keys {
            let result = &results[algo];
            for (p_name, metric) in &result.metrics {
                let proc = process_lookup.get(p_name);
                let arrival_str = proc.map(|p| p.arrival.as_str()).unwrap_or("0");
                let service_str = proc.map(|p| p.service.as_str()).unwrap_or("1");
                let _ = writer.write_record(&[
                    algo,
                    p_name,
                    arrival_str,
                    service_str,
                    &format_float(metric.finish_time),
                    &format_float(metric.turnaround_time),
                    &format!("{:.6}", metric.normalized_turnaround_time),
                    &format!("{:.6}", result.average_normalized_turnaround_time),
                ]);
            }
        }

        Self::finish_csv(writer);
    }

    fn export_multi_csv(&self, result: &MulticoreScheduleResult) {
        let file_path = rfd::FileDialog::new()
            .set_title("Export CSV")
            .set_file_name("schedule_multicore_metrics.csv")
            .add_filter("CSV File", &["csv"])
            .save_file();

        let Some(path) = file_path else {
            return;
        };

        // Flexible so the per-process metrics block and the migration-events
        // block can have different column counts in the same file.
        let mut writer = match csv::WriterBuilder::new().flexible(true).from_path(&path) {
            Ok(w) => w,
            Err(e) => {
                Self::csv_error_dialog(&e);
                return;
            }
        };

        for record in multicore_csv_records(result, &self.processes) {
            let _ = writer.write_record(&record);
        }

        Self::finish_csv(writer);
    }

    fn csv_error_dialog(err: &csv::Error) {
        rfd::MessageDialog::new()
            .set_title("Error")
            .set_description(&format!("Failed to write CSV: {}", err))
            .set_level(rfd::MessageLevel::Error)
            .show();
    }

    fn finish_csv<W: std::io::Write>(mut writer: csv::Writer<W>) {
        let _ = writer.flush();
        rfd::MessageDialog::new()
            .set_title("Success")
            .set_description("Metrics successfully exported to CSV.")
            .set_level(rfd::MessageLevel::Info)
            .show();
    }

    fn export_png(&self) {
        match &self.results {
            Some(Ok(CalcResults::Uni(results))) => self.export_uni_png(results),
            Some(Ok(CalcResults::Multi(result))) => self.export_multi_png(result),
            _ => {
                rfd::MessageDialog::new()
                    .set_title("Export PNG")
                    .set_description("Calculate a valid schedule first.")
                    .set_level(rfd::MessageLevel::Warning)
                    .show();
            }
        }
    }

    fn export_uni_png(&self, results: &HashMap<String, ScheduleResult>) {
        let file_path = rfd::FileDialog::new()
            .set_title("Export PNG")
            .set_file_name("schedule_unicore.png")
            .add_filter("PNG Image", &["png"])
            .save_file();

        let Some(path) = file_path else {
            return;
        };

        let num_processes = self.processes.len();
        let num_algos = results.len();
        if num_processes == 0 || num_algos == 0 {
            return;
        }

        let max_time = results
            .values()
            .map(|r| r.total_completion_time)
            .fold(0.0f64, f64::max)
            .max(1.0);

        // Scale factor for 300 DPI high resolution
        let sf: u32 = 3;

        let lane_h = 30 * sf;
        let lane_g = 5 * sf;
        let algo_g = 30 * sf;
        let process_area_h = num_processes as u32 * (lane_h + lane_g);
        let algo_block_h = process_area_h + algo_g;
        let img_w = 1200 * sf;
        let img_h = num_algos as u32 * algo_block_h + 80 * sf; // top and bottom margins

        let mut img = image::RgbImage::from_pixel(img_w, img_h, image::Rgb([255, 255, 255]));

        let left_margin = 160 * sf;
        let right_margin = 40 * sf;
        let plot_w = img_w - left_margin - right_margin;

        let scale_x = |t: f64| -> u32 { left_margin + ((t / max_time) * plot_w as f64) as u32 };

        let font = load_system_font();

        let step = if max_time <= 20.0 {
            1.0
        } else if max_time <= 50.0 {
            2.0
        } else {
            5.0
        };
        let mut t = 0.0f64;
        let grid_color = image::Rgb([220, 220, 220]);
        let axis_y_start = 40 * sf;
        let axis_y_end = img_h - 40 * sf;

        while t <= max_time {
            let x = scale_x(t);
            draw_dashed_line_v(
                &mut img,
                x,
                axis_y_start,
                axis_y_end,
                grid_color,
                4 * sf,
                4 * sf,
                sf,
            );

            if let Some(ref f) = font {
                let lbl = format_float(t);
                draw_text(
                    &mut img,
                    f,
                    &lbl,
                    x as i32 - (10 * sf) as i32,
                    axis_y_end as i32 + (10 * sf) as i32,
                    14.0 * sf as f32,
                    image::Rgb([50, 50, 50]),
                );
            }
            t += step;
        }

        let mut sorted_keys: Vec<&String> = results.keys().collect();
        sorted_keys.sort_by_key(|key| algo_priority(key));

        let process_order: Vec<String> = self.processes.iter().map(|p| p.name.clone()).collect();

        for (algo_idx, algo_name) in sorted_keys.iter().enumerate() {
            let result = &results[*algo_name];
            let algo_y_start = axis_y_start + algo_idx as u32 * algo_block_h;

            if let Some(ref f) = font {
                draw_text(
                    &mut img,
                    f,
                    algo_name,
                    15 * sf as i32,
                    (algo_y_start + process_area_h / 2) as i32 - 7 * sf as i32,
                    16.0 * sf as f32,
                    image::Rgb([0, 0, 0]),
                );
            }

            if algo_idx > 0 {
                let sep_y = algo_y_start - algo_g / 2;
                draw_line_h(
                    &mut img,
                    10 * sf,
                    img_w - 10 * sf,
                    sep_y,
                    sf,
                    image::Rgb([200, 200, 200]),
                );
            }

            for (p_idx, p_name) in process_order.iter().enumerate() {
                let lane_y = algo_y_start + p_idx as u32 * (lane_h + lane_g);
                let mid_y = lane_y + lane_h / 2;

                if let Some(ref f) = font {
                    draw_text(
                        &mut img,
                        f,
                        p_name,
                        (left_margin as i32) - 30 * sf as i32,
                        mid_y as i32 - 7 * sf as i32,
                        14.0 * sf as f32,
                        image::Rgb([50, 50, 50]),
                    );
                }

                draw_line_h(
                    &mut img,
                    left_margin,
                    img_w - right_margin,
                    mid_y,
                    sf,
                    image::Rgb([240, 240, 240]),
                );
            }

            for slice in &result.slices {
                if let Some(p_idx) = process_order.iter().position(|name| *name == slice.process) {
                    let lane_y = algo_y_start + p_idx as u32 * (lane_h + lane_g);
                    let x0 = scale_x(slice.start);
                    let x1 = scale_x(slice.end);
                    let block_w = x1 - x0;
                    let color = process_color_rgb(&slice.process);

                    draw_rect(&mut img, x0, lane_y, x1, lane_y + lane_h, color);
                    draw_rect_outline(
                        &mut img,
                        x0,
                        lane_y,
                        x1,
                        lane_y + lane_h,
                        sf,
                        image::Rgb([0, 0, 0]),
                    );

                    if block_w >= 14 * sf {
                        if let Some(ref f) = font {
                            let text_x = x0 + block_w / 2 - 5 * sf;
                            let text_y = lane_y + lane_h / 2 - 7 * sf;
                            draw_text(
                                &mut img,
                                f,
                                &slice.process,
                                text_x as i32,
                                text_y as i32,
                                14.0 * sf as f32,
                                image::Rgb([255, 255, 255]),
                            );
                        }
                    }
                }
            }
        }

        let arrival_color = image::Rgb([200, 0, 0]);
        let half_thick = sf;
        for row in &self.processes {
            let Ok(arrival) = row.arrival.trim().parse::<f64>() else {
                continue;
            };
            if arrival < 0.0 || arrival > max_time {
                continue;
            }
            let Some(p_idx) = process_order.iter().position(|name| *name == row.name) else {
                continue;
            };
            let x = scale_x(arrival);
            let x0 = x.saturating_sub(half_thick);
            let x1 = x + half_thick;
            for algo_idx in 0..sorted_keys.len() {
                let algo_y_start = axis_y_start + algo_idx as u32 * algo_block_h;
                let lane_y = algo_y_start + p_idx as u32 * (lane_h + lane_g);
                draw_rect(&mut img, x0, lane_y, x1, lane_y + lane_h, arrival_color);
            }
        }

        if let Some(ref f) = font {
            draw_text(
                &mut img,
                f,
                "CPU Scheduling Timeline",
                left_margin as i32,
                25 * sf as i32,
                20.0 * sf as f32,
                image::Rgb([0, 0, 0]),
            );
        }

        match save_png_with_300_dpi(&path, &img) {
            Ok(_) => {
                rfd::MessageDialog::new()
                    .set_title("Success")
                    .set_description("Timeline image successfully exported as PNG at 300 DPI.")
                    .set_level(rfd::MessageLevel::Info)
                    .show();
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(&format!("Failed to save image: {}", e))
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            }
        }
    }

    fn export_multi_png(&self, result: &MulticoreScheduleResult) {
        let file_path = rfd::FileDialog::new()
            .set_title("Export PNG")
            .set_file_name("schedule_multicore.png")
            .add_filter("PNG Image", &["png"])
            .save_file();

        let Some(path) = file_path else {
            return;
        };

        let mut process_order: Vec<String> =
            self.processes.iter().map(|p| p.name.clone()).collect();
        process_order.sort();
        let num_processes = process_order.len();
        let num_cores = result.cores as usize;
        if num_processes == 0 || num_cores == 0 {
            return;
        }

        let max_time = result.total_completion_time.max(1.0);

        let sf: u32 = 3;
        let lane_h = 30 * sf;
        let lane_g = 5 * sf;
        let core_g = 30 * sf;
        let process_area_h = num_processes as u32 * (lane_h + lane_g);
        let core_block_h = process_area_h + core_g;
        let img_w = 1200 * sf;
        let img_h = num_cores as u32 * core_block_h + 80 * sf;

        let mut img = image::RgbImage::from_pixel(img_w, img_h, image::Rgb([255, 255, 255]));

        let left_margin = 160 * sf;
        let right_margin = 40 * sf;
        let plot_w = img_w - left_margin - right_margin;

        let scale_x = |t: f64| -> u32 { left_margin + ((t / max_time) * plot_w as f64) as u32 };

        let font = load_system_font();

        let step = if max_time <= 20.0 {
            1.0
        } else if max_time <= 50.0 {
            2.0
        } else {
            5.0
        };
        let grid_color = image::Rgb([220, 220, 220]);
        let axis_y_start = 40 * sf;
        let axis_y_end = img_h - 40 * sf;

        let mut t = 0.0f64;
        while t <= max_time {
            let x = scale_x(t);
            draw_dashed_line_v(
                &mut img,
                x,
                axis_y_start,
                axis_y_end,
                grid_color,
                4 * sf,
                4 * sf,
                sf,
            );

            if let Some(ref f) = font {
                let lbl = format_float(t);
                draw_text(
                    &mut img,
                    f,
                    &lbl,
                    x as i32 - (10 * sf) as i32,
                    axis_y_end as i32 + (10 * sf) as i32,
                    14.0 * sf as f32,
                    image::Rgb([50, 50, 50]),
                );
            }
            t += step;
        }

        for core_idx in 0..num_cores {
            let core_y_start = axis_y_start + core_idx as u32 * core_block_h;

            if let Some(ref f) = font {
                draw_text(
                    &mut img,
                    f,
                    &format!("Core {}", core_idx),
                    15 * sf as i32,
                    (core_y_start + process_area_h / 2) as i32 - 7 * sf as i32,
                    16.0 * sf as f32,
                    image::Rgb([0, 0, 0]),
                );
            }

            if core_idx > 0 {
                let sep_y = core_y_start - core_g / 2;
                draw_line_h(
                    &mut img,
                    10 * sf,
                    img_w - 10 * sf,
                    sep_y,
                    sf,
                    image::Rgb([200, 200, 200]),
                );
            }

            for (p_idx, p_name) in process_order.iter().enumerate() {
                let lane_y = core_y_start + p_idx as u32 * (lane_h + lane_g);
                let mid_y = lane_y + lane_h / 2;

                if let Some(ref f) = font {
                    draw_text(
                        &mut img,
                        f,
                        p_name,
                        (left_margin as i32) - 30 * sf as i32,
                        mid_y as i32 - 7 * sf as i32,
                        14.0 * sf as f32,
                        image::Rgb([50, 50, 50]),
                    );
                }

                draw_line_h(
                    &mut img,
                    left_margin,
                    img_w - right_margin,
                    mid_y,
                    sf,
                    image::Rgb([240, 240, 240]),
                );
            }

            for slice in &result.slices_per_core[core_idx] {
                let Some(p_idx) = process_order.iter().position(|name| *name == slice.process)
                else {
                    continue;
                };
                let lane_y = core_y_start + p_idx as u32 * (lane_h + lane_g);
                let x0 = scale_x(slice.start);
                let x1 = scale_x(slice.end);
                let block_w = x1 - x0;
                let color = process_color_rgb(&slice.process);

                draw_rect(&mut img, x0, lane_y, x1, lane_y + lane_h, color);
                draw_rect_outline(
                    &mut img,
                    x0,
                    lane_y,
                    x1,
                    lane_y + lane_h,
                    sf,
                    image::Rgb([0, 0, 0]),
                );

                if block_w >= 14 * sf {
                    if let Some(ref f) = font {
                        let text_x = x0 + block_w / 2 - 5 * sf;
                        let text_y = lane_y + lane_h / 2 - 7 * sf;
                        draw_text(
                            &mut img,
                            f,
                            &slice.process,
                            text_x as i32,
                            text_y as i32,
                            14.0 * sf as f32,
                            image::Rgb([255, 255, 255]),
                        );
                    }
                }
            }
        }

        let migration_color = image::Rgb([220, 80, 80]);
        for ev in &result.migrations {
            let x = scale_x(ev.at);
            draw_dashed_line_v(
                &mut img,
                x,
                axis_y_start,
                axis_y_end,
                migration_color,
                3 * sf,
                3 * sf,
                sf,
            );
            if let Some(ref f) = font {
                draw_text(
                    &mut img,
                    f,
                    &format!("{}: {}→{}", ev.process, ev.from, ev.to),
                    x as i32 + (3 * sf) as i32,
                    axis_y_start as i32 + (15 * sf) as i32,
                    12.0 * sf as f32,
                    migration_color,
                );
            }
        }

        let arrival_color = image::Rgb([200, 0, 0]);
        let half_thick = sf;
        for row in &self.processes {
            let Ok(arrival) = row.arrival.trim().parse::<f64>() else {
                continue;
            };
            if arrival < 0.0 || arrival > max_time {
                continue;
            }
            let Some(p_idx) = process_order.iter().position(|name| *name == row.name) else {
                continue;
            };
            let x = scale_x(arrival);
            let x0 = x.saturating_sub(half_thick);
            let x1 = x + half_thick;
            for core_idx in 0..num_cores {
                let core_y_start = axis_y_start + core_idx as u32 * core_block_h;
                let lane_y = core_y_start + p_idx as u32 * (lane_h + lane_g);
                draw_rect(&mut img, x0, lane_y, x1, lane_y + lane_h, arrival_color);
            }
        }

        if let Some(ref f) = font {
            let title = format!(
                "Multicore Scheduling Timeline ({} — {} cores)",
                result.algorithm, result.cores
            );
            draw_text(
                &mut img,
                f,
                &title,
                left_margin as i32,
                25 * sf as i32,
                20.0 * sf as f32,
                image::Rgb([0, 0, 0]),
            );
        }

        match save_png_with_300_dpi(&path, &img) {
            Ok(_) => {
                rfd::MessageDialog::new()
                    .set_title("Success")
                    .set_description("Timeline image successfully exported as PNG at 300 DPI.")
                    .set_level(rfd::MessageLevel::Info)
                    .show();
            }
            Err(e) => {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(&format!("Failed to save image: {}", e))
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            }
        }
    }
}

impl eframe::App for SchedVizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.window_rounding = 8.0.into();
        style.visuals.widgets.noninteractive.rounding = 4.0.into();
        style.visuals.widgets.inactive.rounding = 4.0.into();
        style.visuals.widgets.hovered.rounding = 4.0.into();
        style.visuals.widgets.active.rounding = 4.0.into();
        ctx.set_style(style);

        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.heading("CPU Scheduling Algorithms Simulator & Visualizer");
                ui.label(
                    egui::RichText::new(
                        "A native tool to simulate uniprocessor and multicore short-term scheduling policies",
                    )
                    .weak()
                    .size(11.0),
                );
                ui.label(
                    egui::RichText::new(format!("Author: {}", APP_AUTHOR))
                        .weak()
                        .size(11.0),
                );
                ui.add_space(8.0);
            });
        });

        egui::SidePanel::left("input_panel")
            .resizable(true)
            .default_width(290.0)
            .min_width(260.0)
            .max_width(450.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("Processes");
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .id_salt("input_scroll")
                    .max_height(250.0)
                    .show(ui, |ui| {
                        egui::Grid::new("process_grid")
                            .num_columns(4)
                            .striped(true)
                            .spacing([8.0, 6.0])
                            .show(ui, |ui| {
                                ui.label("Name");
                                ui.label("Arrival Time");
                                ui.label("Service Time");
                                ui.label("Action");
                                ui.end_row();

                                let mut to_delete = None;
                                for (idx, row) in self.processes.iter_mut().enumerate() {
                                    ui.text_edit_singleline(&mut row.name);
                                    if ui.text_edit_singleline(&mut row.arrival).changed() {
                                        self.dirty = true;
                                    }
                                    if ui.text_edit_singleline(&mut row.service).changed() {
                                        self.dirty = true;
                                    }
                                    if ui.button("❌").clicked() {
                                        to_delete = Some(idx);
                                        self.dirty = true;
                                    }
                                    ui.end_row();
                                }

                                if let Some(idx) = to_delete {
                                    self.processes.remove(idx);
                                }
                            });
                    });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("➕ Add Row").clicked() {
                        let next_letter = if self.processes.is_empty() {
                            'A'
                        } else {
                            let last_char = self
                                .processes
                                .last()
                                .unwrap()
                                .name
                                .chars()
                                .next()
                                .unwrap_or('A');
                            if last_char.is_ascii_alphabetic() && last_char != 'Z' {
                                ((last_char as u8) + 1) as char
                            } else {
                                'P'
                            }
                        };
                        self.processes.push(ProcessRow {
                            name: next_letter.to_string(),
                            arrival: "0".to_string(),
                            service: "1".to_string(),
                        });
                        self.dirty = true;
                    }
                });

                ui.label("Load Example:");
                ui.horizontal_wrapped(|ui| {
                    for n in 0..=10u32 {
                        let text = if self.selected_example == Some(n) {
                            if self.dirty {
                                egui::RichText::new(format!("{}", n))
                                    .color(egui::Color32::from_rgb(180, 120, 0))
                                    .strong()
                            } else {
                                egui::RichText::new(format!("{}", n))
                                    .color(egui::Color32::from_rgb(30, 150, 50))
                                    .strong()
                            }
                        } else {
                            egui::RichText::new(format!("{}", n))
                        };
                        if ui.add(egui::Button::new(text).small()).clicked() {
                            self.load_example(n);
                        }
                    }
                });

                ui.separator();
                ui.heading("Mode");
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut self.mode, SchedulingMode::Uniprocessor, "Uniprocessor")
                        .changed()
                    {
                        self.dirty = true;
                    }
                    if ui
                        .radio_value(&mut self.mode, SchedulingMode::Multicore, "Multicore")
                        .changed()
                    {
                        self.dirty = true;
                    }
                });

                ui.separator();
                ui.heading("Algorithms");

                let show_quantum = match self.mode {
                    SchedulingMode::Uniprocessor => {
                        for algo_name in &["FCFS", "RR", "SPN", "SRT", "HRRN"] {
                            let mut is_checked =
                                *self.algorithm_selection.get(*algo_name).unwrap_or(&false);
                            if ui.checkbox(&mut is_checked, *algo_name).changed() {
                                self.algorithm_selection
                                    .insert(algo_name.to_string(), is_checked);
                                self.dirty = true;
                            }
                        }
                        *self.algorithm_selection.get("RR").unwrap_or(&false)
                    }
                    SchedulingMode::Multicore => {
                        let mut changed = false;
                        changed |= ui
                            .radio_value(
                                &mut self.multicore_algorithm,
                                MulticoreAlgorithm::Fcfs,
                                "FCFS",
                            )
                            .changed();
                        changed |= ui
                            .radio_value(
                                &mut self.multicore_algorithm,
                                MulticoreAlgorithm::Rr,
                                "RR",
                            )
                            .changed();
                        changed |= ui
                            .radio_value(
                                &mut self.multicore_algorithm,
                                MulticoreAlgorithm::Srt,
                                "SRT",
                            )
                            .changed();
                        if changed {
                            self.dirty = true;
                        }
                        matches!(self.multicore_algorithm, MulticoreAlgorithm::Rr)
                    }
                };

                if show_quantum {
                    ui.add_space(8.0);
                    ui.label("Round Robin Quantum (q):");
                    ui.horizontal(|ui| {
                        for q_val in 1u32..=6 {
                            if ui
                                .radio_value(&mut self.rr_quantum, q_val, format!("q = {}", q_val))
                                .changed()
                            {
                                self.dirty = true;
                            }
                        }
                    });
                }

                if matches!(self.mode, SchedulingMode::Multicore) {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.heading("Multicore Options");
                    ui.label("Cores:");
                    ui.horizontal(|ui| {
                        if ui
                            .radio_value(&mut self.multicore_cores, Cores::Two, "2")
                            .changed()
                        {
                            self.dirty = true;
                        }
                        if ui
                            .radio_value(&mut self.multicore_cores, Cores::Four, "4")
                            .changed()
                        {
                            self.dirty = true;
                        }
                    });
                    ui.add_space(4.0);
                    ui.label("Migration:");
                    ui.horizontal(|ui| {
                        if ui
                            .radio_value(&mut self.multicore_migration, Migration::None, "None")
                            .changed()
                        {
                            self.dirty = true;
                        }
                        if ui
                            .radio_value(
                                &mut self.multicore_migration,
                                Migration::OnIdle,
                                "On Idle",
                            )
                            .changed()
                        {
                            self.dirty = true;
                        }
                    });
                }

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    // Calculation button (Orange highlighting if dirty)
                    let btn_text = egui::RichText::new("Calculate").strong();
                    let btn = if self.dirty {
                        ui.button(
                            egui::RichText::new("Calculate ⚠️")
                                .color(egui::Color32::from_rgb(255, 165, 0))
                                .strong(),
                        )
                    } else {
                        ui.button(btn_text)
                    };

                    if btn.clicked() {
                        self.calculate();
                    }

                    if ui.button("💾 Export PNG").clicked() {
                        self.export_png();
                    }

                    if ui.button("📄 Export CSV").clicked() {
                        self.export_csv();
                    }
                });
                ui.add_space(10.0);

                if let Some(Err(error)) = &self.results {
                    ui.add_space(6.0);
                    ui.colored_label(egui::Color32::LIGHT_RED, format!("Error: {}", error));
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| match &self.results {
            None => {
                ui.centered_and_justified(|ui| {
                    ui.label("No data loaded. Please add processes and press Calculate.");
                });
            }
            Some(Err(_)) => {
                ui.centered_and_justified(|ui| {
                    ui.label("Calculation error. Check left panel instructions.");
                });
            }
            Some(Ok(CalcResults::Uni(results))) => {
                ui.group(|ui| {
                    ui.subheading("Timeline Chart");
                    ui.add_space(4.0);
                    draw_egui_timeline(
                        ui,
                        results,
                        &self.processes,
                        &mut self.chart_metrics_scroll,
                    );
                });
                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.subheading("Metrics Table");
                    ui.add_space(4.0);
                    draw_egui_metrics_table(
                        ui,
                        results,
                        &self.processes,
                        &mut self.chart_metrics_scroll,
                    );
                });
            }
            Some(Ok(CalcResults::Multi(result))) => {
                ui.group(|ui| {
                    ui.subheading(format!("Timeline Chart — {}", result.algorithm));
                    ui.add_space(4.0);
                    draw_egui_multicore_timeline(
                        ui,
                        result,
                        &self.processes,
                        &mut self.chart_metrics_scroll,
                    );
                });
                ui.add_space(6.0);
                ui.group(|ui| {
                    ui.subheading("Metrics Table");
                    ui.add_space(4.0);
                    draw_egui_multicore_metrics_table(
                        ui,
                        result,
                        &self.processes,
                        &mut self.chart_metrics_scroll,
                    );
                });
            }
        });
    }
}

// GUI Drawing helper functions
trait Subheading {
    fn subheading(&mut self, text: impl Into<String>);
}
impl Subheading for egui::Ui {
    fn subheading(&mut self, text: impl Into<String>) {
        self.label(egui::RichText::new(text).size(16.0).strong());
    }
}

fn draw_egui_timeline(
    ui: &mut egui::Ui,
    results: &HashMap<String, ScheduleResult>,
    processes: &[ProcessRow],
    scroll_offset: &mut f32,
) {
    let mut process_order: Vec<String> = processes.iter().map(|p| p.name.clone()).collect();
    process_order.sort();
    let num_processes = process_order.len();
    if num_processes == 0 {
        ui.label("No processes found.");
        return;
    }

    let max_time = results
        .values()
        .map(|r| r.total_completion_time)
        .fold(0.0f64, f64::max)
        .max(1.0);

    let lane_height = 24.0;
    let lane_gap = 4.0;
    let algo_gap = 24.0;
    let process_area_height = num_processes as f32 * (lane_height + lane_gap);
    let algo_block_height = process_area_height + algo_gap;
    let total_height = results.len() as f32 * algo_block_height + 40.0;

    let out = egui::ScrollArea::vertical()
        .id_salt("gui_timeline_scroll")
        .max_height(280.0)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .vertical_scroll_offset(*scroll_offset)
        .show(ui, |ui| {
            let desired_size = egui::vec2(ui.available_width().max(400.0), total_height);
            let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
            let rect = response.rect;

            painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);

            let left_margin = 95.0;
            let right_margin = 10.0;
            let plot_width = rect.width() - left_margin - right_margin;
            let plot_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x + left_margin, rect.min.y + 10.0),
                egui::pos2(rect.max.x - right_margin, rect.max.y - 30.0),
            );

            let scale_x =
                |t: f64| -> f32 { plot_rect.min.x + (t as f32 / max_time as f32) * plot_width };

            let grid_color = if ui.visuals().dark_mode {
                egui::Color32::from_gray(60)
            } else {
                egui::Color32::from_gray(215)
            };

            let step = if max_time <= 20.0 {
                1.0
            } else if max_time <= 50.0 {
                2.0
            } else {
                5.0
            };
            let mut t = 0.0f64;
            while t <= max_time {
                let x = scale_x(t);

                painter.line_segment(
                    [
                        egui::pos2(x, plot_rect.min.y),
                        egui::pos2(x, plot_rect.max.y),
                    ],
                    egui::Stroke::new(0.5, grid_color),
                );

                painter.text(
                    egui::pos2(x, plot_rect.max.y + 12.0),
                    egui::Align2::CENTER_CENTER,
                    format_float(t),
                    egui::FontId::proportional(13.0),
                    ui.visuals().text_color(),
                );

                t += step;
            }

            let mut sorted_keys: Vec<&String> = results.keys().collect();
            sorted_keys.sort_by_key(|key| algo_priority(key));

            for (algo_idx, algo_name) in sorted_keys.iter().enumerate() {
                let result = &results[*algo_name];
                let algo_y_start = plot_rect.min.y + algo_idx as f32 * algo_block_height;

                painter.text(
                    egui::pos2(
                        rect.min.x + 12.0,
                        algo_y_start + process_area_height / 2.0 - 4.0,
                    ),
                    egui::Align2::LEFT_CENTER,
                    algo_name,
                    egui::FontId::proportional(15.0),
                    ui.visuals().text_color(),
                );

                if algo_idx > 0 {
                    let sep_color = if ui.visuals().dark_mode {
                        egui::Color32::from_gray(60)
                    } else {
                        egui::Color32::from_gray(190)
                    };
                    painter.line_segment(
                        [
                            egui::pos2(rect.min.x, algo_y_start - algo_gap / 2.0),
                            egui::pos2(rect.max.x, algo_y_start - algo_gap / 2.0),
                        ],
                        egui::Stroke::new(1.0, sep_color),
                    );
                }

                for (p_idx, p_name) in process_order.iter().enumerate() {
                    let lane_y = algo_y_start + p_idx as f32 * (lane_height + lane_gap);
                    let mid_y = lane_y + lane_height / 2.0;

                    painter.text(
                        egui::pos2(rect.min.x + 80.0, mid_y),
                        egui::Align2::RIGHT_CENTER,
                        p_name,
                        egui::FontId::proportional(14.0),
                        ui.visuals().text_color(),
                    );

                    painter.line_segment(
                        [
                            egui::pos2(plot_rect.min.x, mid_y),
                            egui::pos2(plot_rect.max.x, mid_y),
                        ],
                        egui::Stroke::new(0.5, grid_color),
                    );
                }

                for slice in &result.slices {
                    if let Some(p_idx) =
                        process_order.iter().position(|name| *name == slice.process)
                    {
                        let lane_y = algo_y_start + p_idx as f32 * (lane_height + lane_gap);
                        let x0 = scale_x(slice.start);
                        let x1 = scale_x(slice.end);
                        let block_rect = egui::Rect::from_min_max(
                            egui::pos2(x0, lane_y),
                            egui::pos2(x1, lane_y + lane_height),
                        );

                        let color = process_color(&slice.process);
                        painter.rect_filled(block_rect, 2.0, color);
                        painter.rect_stroke(
                            block_rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::BLACK),
                        );

                        if (x1 - x0) >= 14.0 {
                            painter.text(
                                block_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &slice.process,
                                egui::FontId::proportional(13.0),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }
            }

            let arrival_color = egui::Color32::from_rgb(200, 0, 0);
            for row in processes {
                let Ok(arrival) = row.arrival.trim().parse::<f64>() else {
                    continue;
                };
                if arrival < 0.0 || arrival > max_time {
                    continue;
                }
                let Some(p_idx) = process_order.iter().position(|name| *name == row.name) else {
                    continue;
                };
                let x = scale_x(arrival);
                for algo_idx in 0..sorted_keys.len() {
                    let algo_y_start = plot_rect.min.y + algo_idx as f32 * algo_block_height;
                    let lane_y = algo_y_start + p_idx as f32 * (lane_height + lane_gap);
                    painter.line_segment(
                        [egui::pos2(x, lane_y), egui::pos2(x, lane_y + lane_height)],
                        egui::Stroke::new(2.0, arrival_color),
                    );
                }
            }
        });
    let max_scroll = (out.content_size.y - out.inner_rect.height()).max(0.0);
    let expected = (*scroll_offset).clamp(0.0, max_scroll);
    let actual = out.state.offset.y;
    if (actual - expected).abs() > 0.5 {
        *scroll_offset = actual;
    }
}

fn draw_egui_metrics_table(
    ui: &mut egui::Ui,
    results: &HashMap<String, ScheduleResult>,
    processes: &[ProcessRow],
    scroll_offset: &mut f32,
) {
    let mut sorted_keys: Vec<&String> = results.keys().collect();
    sorted_keys.sort_by_key(|key| algo_priority(key));

    let mut process_order: Vec<String> = processes.iter().map(|p| p.name.clone()).collect();
    process_order.sort();

    let out = egui::ScrollArea::vertical()
        .id_salt("metrics_scroll")
        .max_height(250.0)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .vertical_scroll_offset(*scroll_offset)
        .show(ui, |ui| {
            egui::Grid::new("metrics_grid")
                .num_columns(6)
                .striped(true)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Algorithm").strong());
                    ui.label(egui::RichText::new("Process").strong());
                    ui.label(egui::RichText::new("Finish Time").strong());
                    ui.label(egui::RichText::new("Turnaround").strong());
                    ui.label(egui::RichText::new("Normalized TAT").strong());
                    ui.label(egui::RichText::new("Avg Normalized TAT").strong());
                    ui.end_row();

                    for algo in sorted_keys {
                        let result = &results[algo];
                        let mut first_row = true;

                        for (p_name, metric) in &result.metrics {
                            ui.label(if first_row { algo.as_str() } else { "" });
                            ui.label(p_name);
                            ui.label(format_float(metric.finish_time));
                            ui.label(format_float(metric.turnaround_time));
                            ui.label(format!("{:.2}", metric.normalized_turnaround_time));
                            ui.label(if first_row {
                                format!("{:.2}", result.average_normalized_turnaround_time)
                            } else {
                                "".to_string()
                            });
                            ui.end_row();
                            first_row = false;
                        }
                    }
                });
        });
    let max_scroll = (out.content_size.y - out.inner_rect.height()).max(0.0);
    let expected = (*scroll_offset).clamp(0.0, max_scroll);
    let actual = out.state.offset.y;
    if (actual - expected).abs() > 0.5 {
        *scroll_offset = actual;
    }
}

fn draw_egui_multicore_timeline(
    ui: &mut egui::Ui,
    result: &MulticoreScheduleResult,
    processes: &[ProcessRow],
    scroll_offset: &mut f32,
) {
    let mut process_order: Vec<String> = processes.iter().map(|p| p.name.clone()).collect();
    process_order.sort();
    let num_processes = process_order.len();
    if num_processes == 0 {
        ui.label("No processes found.");
        return;
    }

    let num_cores = result.cores as usize;
    let max_time = result.total_completion_time.max(1.0);

    let lane_height = 24.0;
    let lane_gap = 4.0;
    let core_gap = 24.0;
    let process_area_height = num_processes as f32 * (lane_height + lane_gap);
    let core_block_height = process_area_height + core_gap;
    let total_height = num_cores as f32 * core_block_height + 40.0;

    let out = egui::ScrollArea::vertical()
        .id_salt("gui_multicore_timeline_scroll")
        .max_height(380.0)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .vertical_scroll_offset(*scroll_offset)
        .show(ui, |ui| {
            let desired_size = egui::vec2(ui.available_width().max(400.0), total_height);
            let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
            let rect = response.rect;

            painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);

            let left_margin = 95.0;
            let right_margin = 10.0;
            let plot_width = rect.width() - left_margin - right_margin;
            let plot_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x + left_margin, rect.min.y + 10.0),
                egui::pos2(rect.max.x - right_margin, rect.max.y - 30.0),
            );

            let scale_x =
                |t: f64| -> f32 { plot_rect.min.x + (t as f32 / max_time as f32) * plot_width };

            let grid_color = if ui.visuals().dark_mode {
                egui::Color32::from_gray(60)
            } else {
                egui::Color32::from_gray(215)
            };

            let step = if max_time <= 20.0 {
                1.0
            } else if max_time <= 50.0 {
                2.0
            } else {
                5.0
            };
            let mut t = 0.0f64;
            while t <= max_time {
                let x = scale_x(t);
                painter.line_segment(
                    [
                        egui::pos2(x, plot_rect.min.y),
                        egui::pos2(x, plot_rect.max.y),
                    ],
                    egui::Stroke::new(0.5, grid_color),
                );
                painter.text(
                    egui::pos2(x, plot_rect.max.y + 12.0),
                    egui::Align2::CENTER_CENTER,
                    format_float(t),
                    egui::FontId::proportional(13.0),
                    ui.visuals().text_color(),
                );
                t += step;
            }

            for core_idx in 0..num_cores {
                let core_y_start = plot_rect.min.y + core_idx as f32 * core_block_height;

                painter.text(
                    egui::pos2(
                        rect.min.x + 12.0,
                        core_y_start + process_area_height / 2.0 - 4.0,
                    ),
                    egui::Align2::LEFT_CENTER,
                    format!("Core {}", core_idx),
                    egui::FontId::proportional(15.0),
                    ui.visuals().text_color(),
                );

                if core_idx > 0 {
                    let sep_color = if ui.visuals().dark_mode {
                        egui::Color32::from_gray(60)
                    } else {
                        egui::Color32::from_gray(190)
                    };
                    painter.line_segment(
                        [
                            egui::pos2(rect.min.x, core_y_start - core_gap / 2.0),
                            egui::pos2(rect.max.x, core_y_start - core_gap / 2.0),
                        ],
                        egui::Stroke::new(1.0, sep_color),
                    );
                }

                for (p_idx, p_name) in process_order.iter().enumerate() {
                    let lane_y = core_y_start + p_idx as f32 * (lane_height + lane_gap);
                    let mid_y = lane_y + lane_height / 2.0;

                    painter.text(
                        egui::pos2(rect.min.x + 80.0, mid_y),
                        egui::Align2::RIGHT_CENTER,
                        p_name,
                        egui::FontId::proportional(14.0),
                        ui.visuals().text_color(),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(plot_rect.min.x, mid_y),
                            egui::pos2(plot_rect.max.x, mid_y),
                        ],
                        egui::Stroke::new(0.5, grid_color),
                    );
                }

                for slice in &result.slices_per_core[core_idx] {
                    let Some(p_idx) = process_order.iter().position(|name| *name == slice.process)
                    else {
                        continue;
                    };
                    let lane_y = core_y_start + p_idx as f32 * (lane_height + lane_gap);
                    let x0 = scale_x(slice.start);
                    let x1 = scale_x(slice.end);
                    let block_rect = egui::Rect::from_min_max(
                        egui::pos2(x0, lane_y),
                        egui::pos2(x1, lane_y + lane_height),
                    );
                    let color = process_color(&slice.process);
                    painter.rect_filled(block_rect, 2.0, color);
                    painter.rect_stroke(
                        block_rect,
                        2.0,
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    );
                    if (x1 - x0) >= 14.0 {
                        painter.text(
                            block_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &slice.process,
                            egui::FontId::proportional(13.0),
                            egui::Color32::WHITE,
                        );
                    }
                }
            }

            let migration_color = egui::Color32::from_rgb(220, 80, 80);
            for ev in &result.migrations {
                let x = scale_x(ev.at);
                painter.line_segment(
                    [
                        egui::pos2(x, plot_rect.min.y),
                        egui::pos2(x, plot_rect.max.y),
                    ],
                    egui::Stroke::new(1.0, migration_color),
                );
                painter.text(
                    egui::pos2(x + 2.0, plot_rect.min.y + 14.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}: {}→{}", ev.process, ev.from, ev.to),
                    egui::FontId::proportional(11.0),
                    migration_color,
                );
            }

            let arrival_color = egui::Color32::from_rgb(200, 0, 0);
            for row in processes {
                let Ok(arrival) = row.arrival.trim().parse::<f64>() else {
                    continue;
                };
                if arrival < 0.0 || arrival > max_time {
                    continue;
                }
                let Some(p_idx) = process_order.iter().position(|name| *name == row.name) else {
                    continue;
                };
                let x = scale_x(arrival);
                for core_idx in 0..num_cores {
                    let core_y_start = plot_rect.min.y + core_idx as f32 * core_block_height;
                    let lane_y = core_y_start + p_idx as f32 * (lane_height + lane_gap);
                    painter.line_segment(
                        [egui::pos2(x, lane_y), egui::pos2(x, lane_y + lane_height)],
                        egui::Stroke::new(2.0, arrival_color),
                    );
                }
            }
        });
    let max_scroll = (out.content_size.y - out.inner_rect.height()).max(0.0);
    let expected = (*scroll_offset).clamp(0.0, max_scroll);
    let actual = out.state.offset.y;
    if (actual - expected).abs() > 0.5 {
        *scroll_offset = actual;
    }
}

fn draw_egui_multicore_metrics_table(
    ui: &mut egui::Ui,
    result: &MulticoreScheduleResult,
    processes: &[ProcessRow],
    scroll_offset: &mut f32,
) {
    let _ = processes;
    let out = egui::ScrollArea::vertical()
        .id_salt("multicore_metrics_scroll")
        .max_height(250.0)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .vertical_scroll_offset(*scroll_offset)
        .show(ui, |ui| {
            egui::Grid::new("multicore_metrics_grid")
                .num_columns(5)
                .striped(true)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Process").strong());
                    ui.label(egui::RichText::new("Finish Time").strong());
                    ui.label(egui::RichText::new("Turnaround").strong());
                    ui.label(egui::RichText::new("Normalized TAT").strong());
                    ui.label(egui::RichText::new("Avg Normalized TAT").strong());
                    ui.end_row();

                    let mut first = true;
                    for (p_name, metric) in &result.metrics {
                        ui.label(p_name);
                        ui.label(format_float(metric.finish_time));
                        ui.label(format_float(metric.turnaround_time));
                        ui.label(format!("{:.2}", metric.normalized_turnaround_time));
                        ui.label(if first {
                            format!("{:.2}", result.average_normalized_turnaround_time)
                        } else {
                            String::new()
                        });
                        ui.end_row();
                        first = false;
                    }
                });

            if !result.migrations.is_empty() {
                ui.add_space(8.0);
                ui.subheading("Migration Events");
                egui::Grid::new("multicore_migrations_grid")
                    .num_columns(4)
                    .striped(true)
                    .spacing([14.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Time").strong());
                        ui.label(egui::RichText::new("Process").strong());
                        ui.label(egui::RichText::new("From Core").strong());
                        ui.label(egui::RichText::new("To Core").strong());
                        ui.end_row();
                        for ev in &result.migrations {
                            ui.label(format_float(ev.at));
                            ui.label(&ev.process);
                            ui.label(format!("{}", ev.from));
                            ui.label(format!("{}", ev.to));
                            ui.end_row();
                        }
                    });
            }
        });
    let max_scroll = (out.content_size.y - out.inner_rect.height()).max(0.0);
    let expected = (*scroll_offset).clamp(0.0, max_scroll);
    let actual = out.state.offset.y;
    if (actual - expected).abs() > 0.5 {
        *scroll_offset = actual;
    }
}

fn algo_priority(name: &str) -> usize {
    if name.starts_with("FCFS") {
        0
    } else if name.starts_with("RR") {
        1
    } else if name.starts_with("SPN") {
        2
    } else if name.starts_with("SRT") {
        3
    } else if name.starts_with("HRRN") {
        4
    } else {
        5
    }
}

fn format_float(val: f64) -> String {
    if val.fract() == 0.0 {
        format!("{}", val as i64)
    } else {
        format!("{:.2}", val)
    }
}

/// Build the CSV records for a multicore schedule: a per-process metrics block
/// (with a `Cores` column listing which core[s] each process ran on) followed by
/// a migration-events block that mirrors the on-screen "Migration Events" table.
/// Kept as a pure function so it is unit-testable without the file dialog.
fn multicore_csv_records(
    result: &MulticoreScheduleResult,
    processes: &[ProcessRow],
) -> Vec<Vec<String>> {
    let s = |v: &str| v.to_string();
    let mut records: Vec<Vec<String>> = Vec::new();

    records.push(vec![
        s("Algorithm"),
        s("Process"),
        s("Arrival Time"),
        s("Service Time"),
        s("Cores"),
        s("Finish Time"),
        s("Turnaround Time"),
        s("Normalized Turnaround Time"),
        s("Average Normalized Turnaround Time"),
    ]);

    let process_lookup: HashMap<&str, &ProcessRow> =
        processes.iter().map(|p| (p.name.as_str(), p)).collect();

    for (p_name, metric) in &result.metrics {
        let proc = process_lookup.get(p_name.as_str());
        let arrival = proc.map(|p| p.arrival.clone()).unwrap_or_else(|| s("0"));
        let service = proc.map(|p| p.service.clone()).unwrap_or_else(|| s("1"));

        // Cores this process executed on (more than one if it migrated).
        let cores_str = result
            .slices_per_core
            .iter()
            .enumerate()
            .filter(|(_, slices)| slices.iter().any(|slice| slice.process == *p_name))
            .map(|(core, _)| core.to_string())
            .collect::<Vec<_>>()
            .join(";");

        records.push(vec![
            result.algorithm.clone(),
            p_name.clone(),
            arrival,
            service,
            cores_str,
            format_float(metric.finish_time),
            format_float(metric.turnaround_time),
            format!("{:.6}", metric.normalized_turnaround_time),
            format!("{:.6}", result.average_normalized_turnaround_time),
        ]);
    }

    if !result.migrations.is_empty() {
        records.push(vec![String::new()]);
        records.push(vec![s("Migration Events")]);
        records.push(vec![s("Time"), s("Process"), s("From Core"), s("To Core")]);
        for ev in &result.migrations {
            records.push(vec![
                format_float(ev.at),
                ev.process.clone(),
                ev.from.to_string(),
                ev.to.to_string(),
            ]);
        }
    }

    records
}

fn process_color(name: &str) -> egui::Color32 {
    let r = process_color_rgb(name);
    egui::Color32::from_rgb(r[0], r[1], r[2])
}

fn process_color_rgb(name: &str) -> image::Rgb<u8> {
    match name {
        "A" => image::Rgb([76, 120, 168]),
        "B" => image::Rgb([245, 133, 24]),
        "C" => image::Rgb([84, 162, 75]),
        "D" => image::Rgb([228, 87, 86]),
        "E" => image::Rgb([114, 183, 178]),
        "F" => image::Rgb([178, 121, 162]),
        "G" => image::Rgb([255, 157, 166]),
        "H" => image::Rgb([157, 117, 93]),
        "I" => image::Rgb([186, 176, 172]),
        _ => image::Rgb([107, 114, 128]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, arrival: &str, service: &str) -> ProcessRow {
        ProcessRow {
            name: name.to_string(),
            arrival: arrival.to_string(),
            service: service.to_string(),
        }
    }

    fn proc(name: &str, arrival: f64, service: f64, order: usize) -> Process {
        Process {
            name: name.to_string(),
            arrival,
            service,
            order,
        }
    }

    // Example 8 on 2 cores with on-idle migration produces two work-steals, so
    // the CSV must expose a migrated process on multiple cores and a
    // migration-events block that matches the migration log.
    #[test]
    fn multicore_csv_includes_cores_and_migration_block() {
        let rows = vec![
            row("A", "0", "10"),
            row("B", "4", "4"),
            row("C", "5", "8"),
            row("D", "9", "5"),
            row("E", "12", "3"),
        ];
        let procs: Vec<Process> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| {
                proc(
                    &r.name,
                    r.arrival.parse().unwrap(),
                    r.service.parse().unwrap(),
                    i,
                )
            })
            .collect();
        let config = MulticoreConfig {
            cores: Cores::Two,
            algorithm: MulticoreAlgorithm::Fcfs,
            quantum: Quantum::new(2).unwrap(),
            migration: Migration::OnIdle,
        };
        let result = schedule_multicore(&procs, config).unwrap();
        assert!(
            !result.migrations.is_empty(),
            "expected migrations for setup"
        );

        let records = multicore_csv_records(&result, &rows);

        // Header carries the multicore-specific Cores column.
        assert_eq!(records[0][0], "Algorithm");
        assert_eq!(records[0][4], "Cores");
        assert_eq!(records[0].len(), 9);

        // One metrics row per process, arrival/service resolved from ProcessRow.
        let metric_rows: Vec<&Vec<String>> =
            records.iter().skip(1).take(result.metrics.len()).collect();
        assert_eq!(metric_rows.len(), 5);
        for r in &metric_rows {
            assert_eq!(r.len(), 9);
            assert!(!r[4].is_empty(), "Cores column must not be empty");
        }

        // The Cores column reflects where each process actually executed: a
        // migrated process runs on (at least) its destination core.
        for ev in &result.migrations {
            let cores = &metric_rows
                .iter()
                .find(|r| r[1] == ev.process)
                .expect("migrated process has a metrics row")[4];
            assert!(
                cores.split(';').any(|c| c == ev.to.to_string()),
                "process {} should execute on destination core {} (got {})",
                ev.process,
                ev.to,
                cores
            );
        }

        // Migration-events block header + one row per logged migration.
        let header_idx = records
            .iter()
            .position(|r| r == &vec!["Migration Events".to_string()])
            .expect("migration block header present");
        assert_eq!(
            records[header_idx + 1],
            vec!["Time", "Process", "From Core", "To Core"]
        );
        let event_rows = &records[header_idx + 2..];
        assert_eq!(event_rows.len(), result.migrations.len());
        for (rec, ev) in event_rows.iter().zip(&result.migrations) {
            assert_eq!(rec[1], ev.process);
            assert_eq!(rec[2], ev.from.to_string());
            assert_eq!(rec[3], ev.to.to_string());
        }
    }
}

fn draw_rect(img: &mut image::RgbImage, x0: u32, y0: u32, x1: u32, y1: u32, color: image::Rgb<u8>) {
    let x_min = x0.min(img.width());
    let x_max = x1.min(img.width());
    let y_min = y0.min(img.height());
    let y_max = y1.min(img.height());
    for y in y_min..y_max {
        for x in x_min..x_max {
            img.put_pixel(x, y, color);
        }
    }
}

fn draw_rect_outline(
    img: &mut image::RgbImage,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    thickness: u32,
    color: image::Rgb<u8>,
) {
    draw_line_h(img, x0, x1, y0, thickness, color);
    if y1 >= thickness {
        draw_line_h(img, x0, x1, y1 - thickness, thickness, color);
    }
    draw_line_v(img, x0, y0, y1, thickness, color);
    if x1 >= thickness {
        draw_line_v(img, x1 - thickness, y0, y1, thickness, color);
    }
}

fn draw_line_h(
    img: &mut image::RgbImage,
    x0: u32,
    x1: u32,
    y: u32,
    thickness: u32,
    color: image::Rgb<u8>,
) {
    let x_min = x0.min(img.width());
    let x_max = x1.min(img.width());
    for t_idx in 0..thickness {
        let cur_y = y + t_idx;
        if cur_y < img.height() {
            for x in x_min..x_max {
                img.put_pixel(x, cur_y, color);
            }
        }
    }
}

fn draw_line_v(
    img: &mut image::RgbImage,
    x: u32,
    y0: u32,
    y1: u32,
    thickness: u32,
    color: image::Rgb<u8>,
) {
    let y_min = y0.min(img.height());
    let y_max = y1.min(img.height());
    for t_idx in 0..thickness {
        let cur_x = x + t_idx;
        if cur_x < img.width() {
            for y in y_min..y_max {
                img.put_pixel(cur_x, y, color);
            }
        }
    }
}

fn draw_dashed_line_v(
    img: &mut image::RgbImage,
    x: u32,
    y0: u32,
    y1: u32,
    color: image::Rgb<u8>,
    dash_len: u32,
    gap_len: u32,
    thickness: u32,
) {
    let y_min = y0.min(img.height());
    let y_max = y1.min(img.height());
    for t_idx in 0..thickness {
        let cur_x = x + t_idx;
        if cur_x < img.width() {
            let mut drawing = true;
            let mut count = 0;
            for y in y_min..y_max {
                if drawing {
                    img.put_pixel(cur_x, y, color);
                }
                count += 1;
                if drawing && count >= dash_len {
                    drawing = false;
                    count = 0;
                } else if !drawing && count >= gap_len {
                    drawing = true;
                    count = 0;
                }
            }
        }
    }
}

fn save_png_with_300_dpi(
    path: &std::path::Path,
    img: &image::RgbImage,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(path)?;
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, img.width(), img.height());
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    // Set 300 DPI: 300 dpi = 11811 pixels per meter
    encoder.set_pixel_dims(Some(png::PixelDimensions {
        xppu: 11811,
        yppu: 11811,
        unit: png::Unit::Meter,
    }));

    let mut writer = encoder.write_header()?;
    writer.write_image_data(img.as_raw())?;
    Ok(())
}

fn load_system_font() -> Option<ab_glyph::FontArc> {
    let paths = [
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\tahoma.ttf",
        "C:\\Windows\\Fonts\\cour.ttf",
    ];
    for path in &paths {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(font) = ab_glyph::FontArc::try_from_vec(bytes) {
                return Some(font);
            }
        }
    }
    None
}

fn draw_text(
    img: &mut image::RgbImage,
    font: &ab_glyph::FontArc,
    text: &str,
    x: i32,
    y: i32,
    scale: f32,
    color: image::Rgb<u8>,
) {
    use ab_glyph::{Font, PxScale, ScaleFont};

    let px_scale = PxScale::from(scale);
    let scaled_font = font.as_scaled(px_scale);
    let mut layout_x = x as f32;

    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let glyph = glyph_id.with_scale_and_position(px_scale, ab_glyph::point(layout_x, y as f32));
        layout_x += scaled_font.h_advance(glyph_id);

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|px, py, v| {
                let px = (bounds.min.x as i32 + px as i32) as u32;
                let py = (bounds.min.y as i32 + py as i32) as u32;
                if px < img.width() && py < img.height() {
                    let bg = img.get_pixel(px, py);
                    let r = ((color[0] as f32 * v) + (bg[0] as f32 * (1.0 - v))) as u8;
                    let g = ((color[1] as f32 * v) + (bg[1] as f32 * (1.0 - v))) as u8;
                    let b = ((color[2] as f32 * v) + (bg[2] as f32 * (1.0 - v))) as u8;
                    img.put_pixel(px, py, image::Rgb([r, g, b]));
                }
            });
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Scheduling Visualization using Rust")
            .with_inner_size([1200.0, 720.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Scheduling Visualization using Rust",
        options,
        Box::new(|_cc| Ok(Box::new(SchedVizApp::default()))),
    )
}
