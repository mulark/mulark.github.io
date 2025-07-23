//! Main binary entrypoint for the MELT benchmarking tool.
//!
//! Parses CLI arguments, to call into belt

use factorio_belt::core::Result;
use clap::{Parser};
use std::path::PathBuf;
use factorio_belt::{BenchmarkConfig, GlobalConfig};
use factorio_belt::benchmark::{run, RunOrder};
use factorio_belt::benchmark::discovery::find_save_files;
use handlebars::Handlebars;
use serde_json::json;
use tokio::fs::create_dir_all;

#[derive(Parser)]
#[command(name = "melt")]
#[command(about = "belt wrapper")]
struct Cli {
    #[arg(long, global = true)]
    verbose: bool,

    #[arg(long, default_value = "6000")]
    ticks: u32,

    #[arg(long, default_value = "5")]
    runs: u32,

    #[arg(long)]
    pattern: Option<String>,

    #[arg(long, default_value = ".")]
    mods_dir: Option<PathBuf>,

    #[arg(long, default_value = "grouped")]
    #[arg(
        help = "Execution order: sequential (A,B,A,B), random (A,B,B,A), or grouped (A,A,B,B)"
    )]
    run_order: RunOrder,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Generate per-tick charts for specified Factorio benchmark metrics (e.g., 'wholeUpdate,gameUpdate'). 'all' to chart all metrics."
    )]
    verbose_metrics: Vec<String>,

    test_id: u32,

    description: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse input
    let cli = Cli::parse();

    // Toggle the tracing level
    if cli.verbose {
        tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    } else {
        tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    }

    // Create a global config for all subcommands
    let global_config = GlobalConfig {
        factorio_path: None, // use autodetect
        verbose: cli.verbose,
    };
    let test_id = cli.test_id;
    let description = cli.description;

    tracing::info!("setting up template");
    let dest_dir = format!("../tests/test-{test_id:06}/");
    let new_template = format!("../tests/test-{test_id:06}/test-{test_id:06}.html.hbs");
    tracing::info!("creating dest_dir {}", dest_dir);
    create_dir_all(&dest_dir).await?;
    tracing::info!("Copying template file to {}", new_template);
    tokio::fs::copy("../template/test/test.html.hbs", &new_template).await?;


    let mut handlebars = Handlebars::new();
    let pre_data = json!({
        "test_id": &format!("{:06}", test_id),
        "description": description,
    });
    tracing::info!("Registering template");
    handlebars.register_template_file("test", &new_template).unwrap();
    tracing::info!("Rendering template (first pass)");
    let rendered_template = handlebars.render("test", &pre_data)?;
    std::fs::write(&new_template, rendered_template)?;

    let benchmark_config = BenchmarkConfig {
        saves_dir: PathBuf::from(std::env::home_dir().unwrap_or_default().join(".factorio/saves")),
        ticks: cli.ticks,
        runs: cli.runs,
        pattern: Some(format!("test-{test_id:06}.*")),
        output: Some(PathBuf::from(&new_template).file_stem().unwrap().into()),
        template_path: Some(PathBuf::from(&new_template)),
        mods_dir: cli.mods_dir,
        run_order: cli.run_order,
        verbose_metrics: cli.verbose_metrics,
        strip_prefix: Some(format!("test-{test_id:06}.")),
        smooth_window: 0
    };


    let dest_save_dir = PathBuf::from(format!("/mnt/mulark.github.io maps/test-{test_id:06}/"));
    create_dir_all(&dest_save_dir).await?;
    let saves = find_save_files(benchmark_config.saves_dir.as_ref(), benchmark_config.pattern.as_ref().map(|s| s.as_str()))?;
    for save_path in saves.iter() {
        tokio::fs::copy(save_path, dest_save_dir.join(save_path.file_name().unwrap())).await?;
    }

    let benchmark_result = run(global_config, benchmark_config).await;

    // If benchmark::run results in an error, print and exit
    if let Err(e) = benchmark_result {
        tracing::error!("{e}");

        if let Some(hint_text) = e.get_hint() {
            tracing::error!("{hint_text}");
        }

        std::process::exit(1);
    }

    Ok(())
}
