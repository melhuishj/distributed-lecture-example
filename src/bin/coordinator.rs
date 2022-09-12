use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

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

#[derive(Default)]
pub struct CoordinatorImpl {
    work_waiting: Arc<Mutex<VecDeque<(String, Work)>>>,
    work_assigned: Arc<Mutex<HashMap<String, (Work, String)>>>,
    work_completed: Arc<Mutex<HashMap<String, Vec<Work>>>>,
}

impl CoordinatorImpl {
    fn new() -> CoordinatorImpl {
        Self {
            work_waiting: Arc::new(Mutex::new(VecDeque::new())),
            work_assigned: Arc::new(Mutex::new(HashMap::new())),
            work_completed: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tonic::async_trait]
impl Coordinator for CoordinatorImpl {
    async fn get_work(
        &self,
        request: Request<GetWorkRequest>,
    ) -> Result<Response<GetWorkResponse>, Status> {
        println!("Request from {:?}", request.remote_addr());
        let work = self.work_waiting.lock().unwrap().pop_front();
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
        println!("Request from {:?}", request.remote_addr());
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
        self.work_completed
            .lock()
            .unwrap()
            .get_mut(&worker_name)
            .unwrap()
            .push(work);
        let response = WorkCompletedResponse {};
        Ok(Response::new(response))
    }

    async fn add_work(
        &self,
        request: Request<AddWorkRequest>,
    ) -> Result<Response<AddWorkResponse>, Status> {
        println!("Request from {:?}", request.remote_addr());
        let id = Uuid::new_v4();
        let work = request.into_inner().work;
        if work.is_none() {
            return Err(Status::new(Code::InvalidArgument, "No work supplied"));
        }
        let work = work.unwrap();
        self.work_waiting
            .lock()
            .unwrap()
            .push_back((id.to_string(), work));
        println!(
            "Work added to queue; queued work items: {}",
            self.work_waiting.lock().unwrap().len()
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
            .map(|(worker_name, works)| WorkerLog {
                worker_name: worker_name.clone(),
                work_completed: works.to_vec(),
            })
            .collect();

        let response = GetSummaryResponse {
            worker_log: worker_logs,
        };
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let coordinator = CoordinatorImpl::new();

    println!("Coordinator server listening on {}", addr);

    Server::builder()
        .add_service(CoordinatorServer::new(coordinator))
        .serve(addr)
        .await?;

    Ok(())
}
