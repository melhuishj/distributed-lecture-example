use std::thread::sleep;
use std::time::Duration;

use clap::Parser;

use work::coordinator_client::CoordinatorClient;
use work::{GetWorkRequest, WorkCompletedRequest};

mod work {
    include!("../work.rs");
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser)]
    name: String,

    #[clap(long, value_parser)]
    coordinator_address: String,

    #[clap(long, value_parser)]
    memory_size: u32,

    #[clap(long, value_parser)]
    cpu_speed: u32,
}

#[derive(Default)]
pub struct CoordinatorImpl {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut client = CoordinatorClient::connect(args.coordinator_address).await?;
    loop {
        let request = tonic::Request::new(GetWorkRequest {
            worker_name: args.name.clone(),
        });

        let response = client.get_work(request).await?;
        let response = response.into_inner();
        let work = response.work;
        if work.is_none() {
            println!("No work, sleeping and trying again");
            sleep(Duration::from_millis(1000));
            continue;
        }
        let work = work.unwrap();
        let mem_times = work.work_size as f64 / args.memory_size as f64;
        let cpu_times = work.work_complexity as f64 / args.cpu_speed as f64;
        let wait = mem_times.ceil() as u64 * cpu_times.ceil() as u64 * 1000;
        sleep(Duration::from_millis(wait));
        let done_request = WorkCompletedRequest {
            work_id: response.work_id,
        };
        client.work_completed(done_request).await?;
    }
}
