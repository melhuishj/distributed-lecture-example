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

#[derive(Default)]
pub struct CoordinatorImpl {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut client = CoordinatorClient::connect(args.coordinator_address).await?;
    loop {
        let request = tonic::Request::new(GetSummaryRequest {});
        let response = client.get_summary(request).await?;
        let response = response.into_inner();
        println!("Work summary: {:?}", response);
        sleep(Duration::from_millis(10000));
    }
}
