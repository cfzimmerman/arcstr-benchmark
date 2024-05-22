use arcstr_benchmark::cli::{Cli, CsvRow, StrType};
use clap::Parser;
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use std::fs::File;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::{fmt::Debug, sync::Arc, time::Instant};
use tokio::task::JoinHandle;

/// Takes a threadsafe string and clones it `clone_ct` times.
async fn task<S>(string: S, clone_ct: usize) -> ()
where
    S: AsRef<str> + Clone + Eq + Debug + Send + Sync,
{
    for _ in 0..clone_ct {
        assert_eq!(string.clone(), string);
    }
}

/// Runs the `Cli::Single` experiment variant.
fn run_single(
    rng: &mut ThreadRng,
    clone_ct: usize,
    string_len: usize,
    str_type: StrType,
) -> JoinHandle<()> {
    let string: String = Alphanumeric.sample_string(rng, string_len);
    let task = match str_type {
        StrType::ArcStr => {
            // The additional `into` puts Arc<str> at a bit of a disadvantage, but it's consistent
            // with what most I/O programs will need anyway.
            let arc_str: Arc<str> = string.into();
            tokio::spawn(black_box(task(arc_str, clone_ct)))
        }
        StrType::OwnedString => tokio::spawn(black_box(task(string, clone_ct))),
    };
    task
}

/// Runs a series of preconfigured experiments, writing results to the given CSV file
async fn csv_report(
    rng: &mut ThreadRng,
    mut csv: csv::Writer<File>,
    task_ct: usize,
    average_over: NonZeroUsize,
    clone_ct: usize,
) -> anyhow::Result<()> {
    // How long each string should be
    const STR_LENS: [usize; 5] = [8, 16, 32, 64, 128];
    // Which string types to test
    const STR_TYPES: [StrType; 2] = [StrType::OwnedString, StrType::ArcStr];

    let num_trials: usize = average_over.into();
    let mut tasks = Vec::with_capacity(task_ct);
    for str_len in STR_LENS.into_iter() {
        for str_type in STR_TYPES {
            println!(
                "tasks: {task_ct}, string len: {str_len}, clone ct: {clone_ct}, type: {str_type:?}"
            );
            let start = Instant::now();
            for _ in 0..num_trials {
                assert_eq!(tasks.len(), 0);
                for _ in 0..task_ct {
                    tasks.push(run_single(rng, clone_ct, str_len, str_type));
                }
                for task in tasks.drain(..) {
                    task.await?;
                }
            }
            csv.serialize(CsvRow {
                task_ct,
                num_trials,
                clone_ct,
                str_len,
                str_type,
                time_sec: start.elapsed().as_secs_f64() / num_trials as f64,
            })?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut rng = thread_rng();

    match args {
        Cli::Single {
            task_ct,
            clone_ct,
            string_len,
            str_type,
        } => {
            let start = Instant::now();
            let mut tasks = Vec::with_capacity(task_ct);
            for _ in 0..task_ct {
                tasks.push(run_single(&mut rng, clone_ct, string_len, str_type));
            }
            for task in tasks {
                task.await?;
            }
            println!("elapsed: {:#?}", start.elapsed());
        }
        Cli::CsvReport {
            csv_path,
            task_ct,
            clone_ct,
            num_trials,
        } => {
            let csv = csv::Writer::from_path(csv_path)?;
            csv_report(&mut rng, csv, task_ct, num_trials, clone_ct.into()).await?;
        }
    };

    Ok(())
}
