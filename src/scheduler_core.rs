use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub arrival: f64,
    pub service: f64,
    pub order: usize,
}

impl Process {
    #[allow(dead_code)]
    pub fn new(name: &str, arrival: f64, service: f64, order: usize) -> Self {
        Self {
            name: name.to_string(),
            arrival,
            service,
            order,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionSlice {
    pub process: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessMetrics {
    pub finish_time: f64,
    pub turnaround_time: f64,
    pub normalized_turnaround_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResult {
    pub algorithm: String,
    pub slices: Vec<ExecutionSlice>,
    pub metrics: BTreeMap<String, ProcessMetrics>,
    pub average_normalized_turnaround_time: f64,
    pub total_completion_time: f64,
}

pub fn validate_processes(processes: &[Process]) -> Result<Vec<Process>, String> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for process in processes {
        let name = process.name.trim().to_string();
        if name.is_empty() {
            return Err("Process names must not be empty.".to_string());
        }
        if seen.contains(&name) {
            return Err(format!("Process names must be unique: {}", name));
        }
        if process.arrival < 0.0 {
            return Err(format!("Arrival time must be >= 0 for process {}.", name));
        }
        if process.service <= 0.0 {
            return Err(format!("Service time must be > 0 for process {}.", name));
        }
        seen.insert(name.clone());
        normalized.push(Process {
            name,
            arrival: process.arrival,
            service: process.service,
            order: process.order,
        });
    }

    if normalized.is_empty() {
        return Err("At least one process is required.".to_string());
    }
    Ok(normalized)
}

pub fn run_selected(
    processes: &[Process],
    algorithms: &[String],
    rr_quantum: u32,
) -> Result<HashMap<String, ScheduleResult>, String> {
    let checked = validate_processes(processes)?;
    let mut results = HashMap::new();

    for algo in algorithms {
        let key = algo.to_uppercase();
        if key == "FCFS" {
            results.insert(key.clone(), schedule_fcfs(&checked)?);
        } else if key == "RR" {
            let label = format!("RR q={}", rr_quantum);
            results.insert(label, schedule_rr(&checked, rr_quantum)?);
        } else if key == "SPN" {
            results.insert(key.clone(), schedule_spn(&checked)?);
        } else if key == "SRT" {
            results.insert(key.clone(), schedule_srt(&checked)?);
        } else if key == "HRRN" {
            results.insert(key.clone(), schedule_hrrn(&checked)?);
        } else {
            return Err(format!("Unknown scheduling algorithm: {}", algo));
        }
    }
    Ok(results)
}

pub fn schedule_fcfs(processes: &[Process]) -> Result<ScheduleResult, String> {
    let checked = validate_processes(processes)?;
    let mut ordered = checked.clone();

    ordered.sort_by(|a, b| {
        a.arrival
            .partial_cmp(&b.arrival)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.order.cmp(&b.order))
            .then(a.name.cmp(&b.name))
    });

    let mut time = 0.0f64;
    let mut slices = Vec::new();

    for process in &ordered {
        let start = time.max(process.arrival);
        let end = start + process.service;
        slices.push(ExecutionSlice {
            process: process.name.clone(),
            start,
            end,
        });
        time = end;
    }

    Ok(build_result("FCFS", &checked, slices))
}

pub fn schedule_rr(processes: &[Process], quantum: u32) -> Result<ScheduleResult, String> {
    if !(1..=6).contains(&quantum) {
        return Err("Round Robin quantum must be between 1 and 6.".to_string());
    }

    let checked = validate_processes(processes)?;
    let mut ordered = checked.clone();
    ordered.sort_by(|a, b| {
        a.arrival
            .partial_cmp(&b.arrival)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.order.cmp(&b.order))
            .then(a.name.cmp(&b.name))
    });

    let mut remaining: HashMap<String, f64> = checked
        .iter()
        .map(|p| (p.name.clone(), p.service))
        .collect();

    let mut time = 0.0f64;
    let mut index = 0;
    let mut ready: Vec<Process> = Vec::new();
    let mut slices: Vec<ExecutionSlice> = Vec::new();

    while remaining.values().any(|&val| val > 0.0) {
        if ready.is_empty() {
            if index < ordered.len() && time < ordered[index].arrival {
                time = ordered[index].arrival;
            }
            while index < ordered.len() && ordered[index].arrival <= time {
                ready.push(ordered[index].clone());
                index += 1;
            }
        }

        if ready.is_empty() {
            continue;
        }

        let process = ready.remove(0);
        let run_for = (quantum as f64).min(remaining[&process.name]);
        let start = time;
        time += run_for;
        *remaining.get_mut(&process.name).unwrap() -= run_for;
        append_slice(&mut slices, &process.name, start, time);

        while index < ordered.len() && ordered[index].arrival <= time {
            ready.push(ordered[index].clone());
            index += 1;
        }

        if remaining[&process.name] > 0.0 {
            ready.push(process);
        }
    }

    Ok(build_result(&format!("RR q={}", quantum), &checked, slices))
}

pub fn schedule_spn(processes: &[Process]) -> Result<ScheduleResult, String> {
    let checked = validate_processes(processes)?;
    let mut unscheduled: HashSet<String> = checked.iter().map(|p| p.name.clone()).collect();
    let by_name: HashMap<String, Process> = checked
        .iter()
        .map(|p| (p.name.clone(), p.clone()))
        .collect();
    let mut time = 0.0f64;
    let mut slices = Vec::new();

    while !unscheduled.is_empty() {
        let mut ready: Vec<&Process> = unscheduled
            .iter()
            .map(|name| &by_name[name])
            .filter(|p| p.arrival <= time)
            .collect();

        if ready.is_empty() {
            time = unscheduled
                .iter()
                .map(|name| by_name[name].arrival)
                .fold(f64::INFINITY, f64::min);
            ready = unscheduled
                .iter()
                .map(|name| &by_name[name])
                .filter(|p| p.arrival <= time)
                .collect();
        }

        // Sort by: (service, arrival, order, name)
        ready.sort_by(|a, b| {
            a.service
                .partial_cmp(&b.service)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.arrival
                        .partial_cmp(&b.arrival)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(a.order.cmp(&b.order))
                .then(a.name.cmp(&b.name))
        });

        let process = ready[0];
        let start = time;
        let end = start + process.service;
        slices.push(ExecutionSlice {
            process: process.name.clone(),
            start,
            end,
        });
        time = end;
        unscheduled.remove(&process.name);
    }

    Ok(build_result("SPN", &checked, slices))
}

pub fn schedule_srt(processes: &[Process]) -> Result<ScheduleResult, String> {
    let checked = validate_processes(processes)?;
    let by_name: HashMap<String, Process> = checked
        .iter()
        .map(|p| (p.name.clone(), p.clone()))
        .collect();
    let mut remaining: HashMap<String, f64> = checked
        .iter()
        .map(|p| (p.name.clone(), p.service))
        .collect();
    let mut time = 0.0f64;
    let mut current: Option<String> = None;
    let mut slices: Vec<ExecutionSlice> = Vec::new();

    while remaining.values().any(|&val| val > 0.0) {
        let mut ready: Vec<&Process> = checked
            .iter()
            .filter(|p| p.arrival <= time && remaining[&p.name] > 0.0)
            .collect();

        if ready.is_empty() {
            time = checked
                .iter()
                .filter(|p| remaining[&p.name] > 0.0)
                .map(|p| p.arrival)
                .fold(f64::INFINITY, f64::min);
            current = None;
            continue;
        }

        // Find candidate by min: (remaining[p.name], p.arrival, p.order, p.name)
        ready.sort_by(|a, b| {
            remaining[&a.name]
                .partial_cmp(&remaining[&b.name])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.arrival
                        .partial_cmp(&b.arrival)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(a.order.cmp(&b.order))
                .then(a.name.cmp(&b.name))
        });

        let candidate = ready[0];
        let chosen = if let Some(ref cur_name) = current {
            if remaining[cur_name] > 0.0
                && by_name[cur_name].arrival <= time
                && (remaining[cur_name] - remaining[&candidate.name]).abs() < 1e-9
            {
                &by_name[cur_name]
            } else {
                candidate
            }
        } else {
            candidate
        };

        current = Some(chosen.name.clone());

        let future_arrivals: Vec<f64> = checked
            .iter()
            .filter(|p| p.arrival > time && remaining[&p.name] > 0.0)
            .map(|p| p.arrival)
            .collect();

        let next_arrival = if future_arrivals.is_empty() {
            None
        } else {
            Some(future_arrivals.into_iter().fold(f64::INFINITY, f64::min))
        };

        let completion_time = time + remaining[&chosen.name];
        let end = match next_arrival {
            None => completion_time,
            Some(arr) => completion_time.min(arr),
        };

        append_slice(&mut slices, &chosen.name, time, end);
        *remaining.get_mut(&chosen.name).unwrap() -= end - time;
        time = end;

        if remaining[&chosen.name] <= 1e-9 {
            current = None;
        }
    }

    Ok(build_result("SRT", &checked, slices))
}

pub fn schedule_hrrn(processes: &[Process]) -> Result<ScheduleResult, String> {
    let checked = validate_processes(processes)?;
    let mut unscheduled: HashSet<String> = checked.iter().map(|p| p.name.clone()).collect();
    let by_name: HashMap<String, Process> = checked
        .iter()
        .map(|p| (p.name.clone(), p.clone()))
        .collect();
    let mut time = 0.0f64;
    let mut slices = Vec::new();

    while !unscheduled.is_empty() {
        let mut ready: Vec<&Process> = unscheduled
            .iter()
            .map(|name| &by_name[name])
            .filter(|p| p.arrival <= time)
            .collect();

        if ready.is_empty() {
            time = unscheduled
                .iter()
                .map(|name| by_name[name].arrival)
                .fold(f64::INFINITY, f64::min);
            ready = unscheduled
                .iter()
                .map(|name| &by_name[name])
                .filter(|p| p.arrival <= time)
                .collect();
        }

        // Sort by HRRN priority:
        // priority = (-response_ratio, arrival, order, name)
        // response_ratio = (wait + service) / service
        ready.sort_by(|a, b| {
            let rr_a = (time - a.arrival + a.service) / a.service;
            let rr_b = (time - b.arrival + b.service) / b.service;

            rr_b.partial_cmp(&rr_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.arrival
                        .partial_cmp(&b.arrival)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
                .then(a.order.cmp(&b.order))
                .then(a.name.cmp(&b.name))
        });

        let process = ready[0];
        let start = time;
        let end = start + process.service;
        slices.push(ExecutionSlice {
            process: process.name.clone(),
            start,
            end,
        });
        time = end;
        unscheduled.remove(&process.name);
    }

    Ok(build_result("HRRN", &checked, slices))
}

fn append_slice(slices: &mut Vec<ExecutionSlice>, process_name: &str, start: f64, end: f64) {
    if (start - end).abs() < 1e-9 {
        return;
    }
    if let Some(last) = slices.last_mut() {
        if last.process == process_name && (last.end - start).abs() < 1e-9 {
            last.end = end;
            return;
        }
    }
    slices.push(ExecutionSlice {
        process: process_name.to_string(),
        start,
        end,
    });
}

fn build_result(
    algorithm: &str,
    processes: &[Process],
    slices: Vec<ExecutionSlice>,
) -> ScheduleResult {
    let mut finish_times = HashMap::new();
    for s in &slices {
        finish_times.insert(s.process.clone(), s.end);
    }

    let mut metrics = BTreeMap::new();
    let mut ordered = processes.to_vec();
    ordered.sort_by(|a, b| a.order.cmp(&b.order).then(a.name.cmp(&b.name)));

    for process in &ordered {
        let finish = *finish_times
            .get(&process.name)
            .unwrap_or(&(process.arrival + process.service));
        let turnaround = finish - process.arrival;
        let normalized_turnaround = turnaround / process.service;
        metrics.insert(
            process.name.clone(),
            ProcessMetrics {
                finish_time: finish,
                turnaround_time: turnaround,
                normalized_turnaround_time: normalized_turnaround,
            },
        );
    }

    let average = if metrics.is_empty() {
        0.0
    } else {
        metrics
            .values()
            .map(|m| m.normalized_turnaround_time)
            .sum::<f64>()
            / (metrics.len() as f64)
    };

    let total = slices.iter().map(|s| s.end).fold(0.0, f64::max);

    ScheduleResult {
        algorithm: algorithm.to_string(),
        slices,
        metrics,
        average_normalized_turnaround_time: average,
        total_completion_time: total,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MulticoreAlgorithm {
    Fcfs,
    Rr,
    Srt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cores {
    Two,
    Four,
}

impl Cores {
    pub fn count(self) -> u8 {
        match self {
            Cores::Two => 2,
            Cores::Four => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Migration {
    None,
    OnIdle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantum(u32);

impl Quantum {
    pub fn new(q: u32) -> Option<Self> {
        (1..=6).contains(&q).then_some(Self(q))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationEvent {
    pub process: String,
    pub from: u8,
    pub to: u8,
    pub at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MulticoreScheduleResult {
    pub algorithm: String,
    pub cores: u8,
    pub slices_per_core: Vec<Vec<ExecutionSlice>>,
    pub migrations: Vec<MigrationEvent>,
    pub metrics: BTreeMap<String, ProcessMetrics>,
    pub average_normalized_turnaround_time: f64,
    pub total_completion_time: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MulticoreConfig {
    pub cores: Cores,
    pub algorithm: MulticoreAlgorithm,
    pub quantum: Quantum,
    pub migration: Migration,
}

impl MulticoreConfig {
    pub fn migration_active(&self) -> bool {
        matches!(self.migration, Migration::OnIdle)
    }
}

pub fn schedule_multicore(
    processes: &[Process],
    config: MulticoreConfig,
) -> Result<MulticoreScheduleResult, String> {
    let checked = validate_processes(processes)?;
    let num_cores = config.cores.count() as usize;

    let mut sorted = checked.clone();
    sorted.sort_by(|a, b| {
        a.arrival
            .partial_cmp(&b.arrival)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.order.cmp(&b.order))
            .then(a.name.cmp(&b.name))
    });

    let mut remaining: HashMap<String, f64> = checked
        .iter()
        .map(|p| (p.name.clone(), p.service))
        .collect();
    let mut queues: Vec<Vec<String>> = vec![Vec::new(); num_cores];
    let mut quantum_left: Vec<f64> = vec![0.0; num_cores];
    let mut current_head: Vec<Option<String>> = vec![None; num_cores];
    let mut slices_per_core: Vec<Vec<ExecutionSlice>> = vec![Vec::new(); num_cores];
    let mut migrations: Vec<MigrationEvent> = Vec::new();

    let mut arrival_idx = 0usize;
    let mut time = 0.0f64;
    let mut next_core_assignment = 0usize;
    let q_full = config.quantum.get() as f64;

    loop {
        while arrival_idx < sorted.len() && sorted[arrival_idx].arrival <= time + 1e-9 {
            let p = &sorted[arrival_idx];
            let target = next_core_assignment % num_cores;
            queues[target].push(p.name.clone());
            next_core_assignment += 1;
            arrival_idx += 1;
        }

        if remaining.values().all(|&r| r <= 1e-9) {
            break;
        }

        if config.migration_active() {
            loop {
                let idle: Vec<usize> = (0..num_cores).filter(|&c| queues[c].is_empty()).collect();
                if idle.is_empty() {
                    break;
                }
                let donor = (0..num_cores)
                    .filter(|c| queues[*c].len() >= 2)
                    .max_by_key(|c| queues[*c].len());
                let Some(donor_idx) = donor else { break };
                let stolen = queues[donor_idx].pop().unwrap();
                let recipient = idle[0];
                migrations.push(MigrationEvent {
                    process: stolen.clone(),
                    from: donor_idx as u8,
                    to: recipient as u8,
                    at: time,
                });
                queues[recipient].push(stolen);
            }
        }

        let mut chosen: Vec<Option<String>> = vec![None; num_cores];
        for c in 0..num_cores {
            if queues[c].is_empty() {
                continue;
            }
            match config.algorithm {
                MulticoreAlgorithm::Fcfs => {
                    chosen[c] = Some(queues[c][0].clone());
                }
                MulticoreAlgorithm::Srt => {
                    let best_idx = (0..queues[c].len())
                        .min_by(|&a, &b| {
                            remaining[&queues[c][a]]
                                .partial_cmp(&remaining[&queues[c][b]])
                                .unwrap_or(std::cmp::Ordering::Equal)
                                .then(queues[c][a].cmp(&queues[c][b]))
                        })
                        .unwrap();
                    if best_idx != 0 {
                        let name = queues[c].remove(best_idx);
                        queues[c].insert(0, name);
                    }
                    chosen[c] = Some(queues[c][0].clone());
                }
                MulticoreAlgorithm::Rr => {
                    let head_name = queues[c][0].clone();
                    if current_head[c].as_ref() != Some(&head_name) {
                        quantum_left[c] = q_full;
                        current_head[c] = Some(head_name.clone());
                    }
                    chosen[c] = Some(head_name);
                }
            }
        }

        if chosen.iter().all(Option::is_none) {
            if arrival_idx < sorted.len() {
                time = sorted[arrival_idx].arrival;
                continue;
            } else {
                break;
            }
        }

        let mut next_event = f64::INFINITY;
        if arrival_idx < sorted.len() {
            next_event = next_event.min(sorted[arrival_idx].arrival);
        }
        for c in 0..num_cores {
            if let Some(name) = &chosen[c] {
                let rem = remaining[name];
                next_event = next_event.min(time + rem);
                if matches!(config.algorithm, MulticoreAlgorithm::Rr) {
                    next_event = next_event.min(time + quantum_left[c]);
                }
            }
        }

        let step = next_event - time;
        if step <= 1e-9 {
            time = next_event;
            continue;
        }

        for c in 0..num_cores {
            if let Some(name) = &chosen[c] {
                append_slice(&mut slices_per_core[c], name, time, next_event);
                *remaining.get_mut(name).unwrap() -= step;
                if matches!(config.algorithm, MulticoreAlgorithm::Rr) {
                    quantum_left[c] -= step;
                }
            }
        }
        time = next_event;

        for c in 0..num_cores {
            if let Some(name) = &chosen[c] {
                if remaining[name] <= 1e-9 {
                    queues[c].retain(|n| n != name);
                    current_head[c] = None;
                } else if matches!(config.algorithm, MulticoreAlgorithm::Rr)
                    && quantum_left[c] <= 1e-9
                {
                    if let Some(pos) = queues[c].iter().position(|n| n == name) {
                        let item = queues[c].remove(pos);
                        queues[c].push(item);
                    }
                    current_head[c] = None;
                }
            }
        }
    }

    let mut finish_times: HashMap<String, f64> = HashMap::new();
    for slices in &slices_per_core {
        for s in slices {
            let entry = finish_times.entry(s.process.clone()).or_insert(s.end);
            if s.end > *entry {
                *entry = s.end;
            }
        }
    }

    let mut metrics = BTreeMap::new();
    let mut ordered = checked.clone();
    ordered.sort_by(|a, b| a.order.cmp(&b.order).then(a.name.cmp(&b.name)));
    for p in &ordered {
        let finish = *finish_times
            .get(&p.name)
            .unwrap_or(&(p.arrival + p.service));
        let turnaround = finish - p.arrival;
        let normalized_turnaround = turnaround / p.service;
        metrics.insert(
            p.name.clone(),
            ProcessMetrics {
                finish_time: finish,
                turnaround_time: turnaround,
                normalized_turnaround_time: normalized_turnaround,
            },
        );
    }

    let average = if metrics.is_empty() {
        0.0
    } else {
        metrics
            .values()
            .map(|m| m.normalized_turnaround_time)
            .sum::<f64>()
            / metrics.len() as f64
    };
    let total = slices_per_core
        .iter()
        .flat_map(|s| s.iter().map(|x| x.end))
        .fold(0.0, f64::max);

    let algo_label = match config.algorithm {
        MulticoreAlgorithm::Fcfs => format!("FCFS ({}c)", num_cores),
        MulticoreAlgorithm::Rr => format!("RR q={} ({}c)", config.quantum.get(), num_cores),
        MulticoreAlgorithm::Srt => format!("SRT ({}c)", num_cores),
    };

    Ok(MulticoreScheduleResult {
        algorithm: algo_label,
        cores: num_cores as u8,
        slices_per_core,
        migrations,
        metrics,
        average_normalized_turnaround_time: average,
        total_completion_time: total,
    })
}
