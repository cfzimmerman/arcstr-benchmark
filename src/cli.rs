use clap::{Parser, ValueEnum};
use std::{num::NonZeroUsize, path::PathBuf};

/// Spawns tokio tasks that clone strings. Used to compare the performance of
/// `String` v. `Arc<str>` on various string sizes and usage patterns.
#[derive(Parser)]
pub enum Cli {
    /// Run the experiment on a single configuration
    Single {
        /// How many tasks should be spawned
        task_ct: usize,
        /// How many times the string should be cloned
        clone_ct: usize,
        /// How long the string should be
        string_len: usize,
        /// Which variant of the experiment to run
        str_type: StrType,
    },
    CsvReport {
        /// How many tasks should be spawned per configuration
        task_ct: usize,
        /// How many trials to average over per configuration
        num_trials: NonZeroUsize,
        /// How many times each task should clone its string
        clone_ct: NonZeroUsize,
        /// A path to the csv file where output will be written
        csv_path: PathBuf,
    },
}

/// Which string type variant the experiment will use
#[derive(Clone, Copy, ValueEnum, serde::Serialize, Debug)]
pub enum StrType {
    OwnedString,
    ArcStr,
}

/// A result row the `CsvReport` writes as output
#[derive(serde::Serialize)]
pub struct CsvRow {
    pub task_ct: usize,
    pub num_trials: usize,
    pub clone_ct: usize,
    pub str_len: usize,
    pub str_type: StrType,
    pub time_sec: f64,
}
