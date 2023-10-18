mod rpc_admin {
    tonic::include_proto!("unit.admin");
}

use std::{path::PathBuf, sync::Mutex};

use log::info;
use tonic::{Request, Response, Status};
use unit_abi::header::decode_abi_header;
use unit_index::Index;
use unit_utils::{gen_uuid, Result};

use crate::config::CONFIG;

use self::rpc_admin::admin_server::Admin;
pub use self::rpc_admin::admin_server::AdminServer;

pub struct AdminService {
    index: Mutex<Index>,
}

impl AdminService {
    pub fn new(index: Index) -> Self {
        Self {
            index: Mutex::new(index),
        }
    }
}

fn write_code(id: String, code: &[u8]) -> Result<String> {
    let name = format!("app-{id}.wasm");
    let full_path = PathBuf::from(CONFIG.storage_location.clone()).join(&name);

    std::fs::write(full_path, code)?;

    Ok(name)
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn update_app(
        &self,
        request: Request<rpc_admin::UpdateAppRequest>,
    ) -> Result<Response<rpc_admin::UpdateAppResponse>, Status> {
        let request = request.into_inner();

        let Ok(header) = decode_abi_header(&request.code) else {
            return Err(Status::invalid_argument(
                "Malformed or missing unit ABI header",
            ));
        };

        let id = gen_uuid();

        let Ok(name) = write_code(id, &request.code) else {
            return Err(Status::internal("Failed to write code"));
        };

        let entry = unit_index::IndexEntry {
            path: name,
            abi_header: header,
        };

        let Ok(mut index) = self.index.lock() else {
            return Err(Status::internal("Failed to lock index"));
        };

        let Ok(_) = index.add_or_update_entry(entry.clone()) else {
            return Err(Status::internal("Failed to update index"));
        };

        info!("updated app code: {}", &entry.abi_header.name);

        Ok(Response::new(rpc_admin::UpdateAppResponse {}))
    }
}
