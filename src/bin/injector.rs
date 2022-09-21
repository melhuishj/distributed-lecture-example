use std::thread::sleep;
use std::time::Duration;

use clap::Parser;

use work::coordinator_client::CoordinatorClient;
use work::{AddWorkRequest, Work};

mod work {
    include!("../work.rs");
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser)]
    coordinator_address: String,

    #[clap(long, value_parser)]
    frequency: u32,

    #[clap(long, value_parser)]
    work_size: u32,

    #[clap(long, value_parser)]
    work_complexity: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut client = CoordinatorClient::connect(args.coordinator_address).await?;
    loop {
        let request = tonic::Request::new(AddWorkRequest {
            work: Some(Work {
                work_size: args.work_size,
                work_complexity: args.work_complexity,
            }),
        });
        let _response = client.add_work(request).await?;
        sleep(Duration::from_millis(args.frequency.into()));
    }
}
