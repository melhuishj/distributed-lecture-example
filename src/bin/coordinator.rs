use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Instant;

use clap::Parser;
use tonic::{transport::Server, Code, Request, Response, Status};
use uuid::Uuid;

use work::coordinator_server::{Coordinator, CoordinatorServer};
use work::{
    get_summary_response::WorkerLog, AddWorkRequest, AddWorkResponse, GetSummaryRequest,
    GetSummaryResponse, GetWorkRequest, GetWorkResponse, Work, WorkCompletedRequest,
    WorkCompletedResponse,
};

mod work {
    include!("../work.rs");
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser)]
    listen_address: String,
}

#[derive(Default)]
pub struct CoordinatorImpl {
    work_queued: Mutex<VecDeque<(String, Work)>>,
    work_assigned: Mutex<HashMap<String, (Work, String)>>,
    work_completed: Mutex<HashMap<String, Vec<(Work, Instant)>>>,
}

impl CoordinatorImpl {
    fn new() -> CoordinatorImpl {
        Self {
            work_queued: Mutex::new(VecDeque::new()),
            work_assigned: Mutex::new(HashMap::new()),
            work_completed: Mutex::new(HashMap::new()),
        }
    }
}

#[tonic::async_trait]
impl Coordinator for CoordinatorImpl {
    async fn get_work(
        &self,
        request: Request<GetWorkRequest>,
    ) -> Result<Response<GetWorkResponse>, Status> {
        let work = self.work_queued.lock().unwrap().pop_front();
        if work.is_none() {
            return Ok(Response::new(GetWorkResponse {
                work_id: "".to_string(),
                work: None,
            }));
        }
        let (id, work) = work.unwrap();
        let name = request.into_inner().worker_name;
        self.work_assigned
            .lock()
            .unwrap()
            .insert(id.clone(), (work.clone(), name));
        let response = GetWorkResponse {
            work_id: id,
            work: Some(work),
        };
        Ok(Response::new(response))
    }

    async fn work_completed(
        &self,
        request: Request<WorkCompletedRequest>,
    ) -> Result<Response<WorkCompletedResponse>, Status> {
        let id = request.into_inner().work_id;
        let work = self.work_assigned.lock().unwrap().remove(&id);
        if work.is_none() {
            return Err(Status::new(Code::InvalidArgument, "Work id does not exist"));
        }
        let (work, worker_name) = work.unwrap();
        if !self
            .work_completed
            .lock()
            .unwrap()
            .contains_key(&worker_name)
        {
            self.work_completed
                .lock()
                .unwrap()
                .insert(worker_name.clone(), vec![]);
        }
        let now = Instant::now();
        self.work_completed
            .lock()
            .unwrap()
            .get_mut(&worker_name)
            .unwrap()
            .push((work, now.clone()));
        self.work_completed
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|(_worker_name, worker_work)| {
                worker_work.retain(|(_work, time)| time.elapsed().as_secs() < 60);
            });
        let response = WorkCompletedResponse {};
        Ok(Response::new(response))
    }

    async fn add_work(
        &self,
        request: Request<AddWorkRequest>,
    ) -> Result<Response<AddWorkResponse>, Status> {
        let id = Uuid::new_v4();
        let work = request.into_inner().work;
        if work.is_none() {
            return Err(Status::new(Code::InvalidArgument, "No work supplied"));
        }
        let work = work.unwrap();
        self.work_queued
            .lock()
            .unwrap()
            .push_back((id.to_string(), work));
        println!(
            "Work added to queue; queued work items: {}",
            self.work_queued.lock().unwrap().len()
        );
        let response = AddWorkResponse {
            work_id: id.to_string(),
        };
        Ok(Response::new(response))
    }

    async fn get_summary(
        &self,
        request: Request<GetSummaryRequest>,
    ) -> Result<Response<GetSummaryResponse>, Status> {
        println!("Request from {:?}", request.remote_addr());
        let worker_logs = self
            .work_completed
            .lock()
            .unwrap()
            .iter()
            .map(|(worker_name, works)| {
                let (works_to_send, _): (Vec<_>, Vec<_>) = works.iter().cloned().unzip();
                return WorkerLog {
                    worker_name: worker_name.clone(),
                    work_completed: works_to_send,
                };
            })
            .collect();
        let (_, queued_work): (Vec<_>, Vec<_>) =
            self.work_queued.lock().unwrap().iter().cloned().unzip();
        let response = GetSummaryResponse {
            worker_log: worker_logs,
            queued_work: queued_work,
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr: SocketAddr = args.listen_address.parse()?;
    let coordinator = CoordinatorImpl::new();

    println!("Coordinator server listening on {}", addr.clone());

    Server::builder()
        .add_service(CoordinatorServer::new(coordinator))
        .serve(addr)
        .await?;

    Ok(())
}
