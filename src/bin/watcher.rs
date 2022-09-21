use std::thread::sleep;
use std::time::Duration;

use clap::Parser;

use work::coordinator_client::CoordinatorClient;
use work::GetSummaryRequest;

mod work {
    include!("../work.rs");
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser)]
    coordinator_address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut client = CoordinatorClient::connect(args.coordinator_address).await?;
    loop {
        let request = tonic::Request::new(GetSummaryRequest {});
        let response = client.get_summary(request).await?;
        let response = response.into_inner();

        println!(
            "{0: <15} | {1: <10} | {2: <10} | {3: <10}",
            "Name".to_string(),
            "Count".to_string(),
            "Work".to_string(),
            "Rate".to_string()
        );
        println!("--------------------------------------------------------");

        let queued_work = response.queued_work;
        let queued_work_count = queued_work.len();
        let queued_work_total: u32 = queued_work
            .iter()
            .map(|work| work.work_complexity * work.work_size)
            .sum();
        println!(
            "{0: <15} | {1: <10} | {2: <10} | {3: <10}",
            "Coordinator".to_string(),
            queued_work_count,
            queued_work_total,
            "-".to_string()
        );

        let worker_log = response.worker_log;
        worker_log.iter().for_each(|log| {
            let name = &log.worker_name;
            let work_count = log.work_completed.len();
            let total_work: u32 = log
                .work_completed
                .iter()
                .map(|work| work.work_complexity * work.work_size)
                .sum();
            let rate = total_work / 60;
            println!(
                "{0: <15} | {1: <10} | {2: <10} | {3: <10}",
                name, work_count, total_work, rate
            );
        });
        println!("");
        println!("");
        println!("");
        sleep(Duration::from_millis(10000));
    }
}
