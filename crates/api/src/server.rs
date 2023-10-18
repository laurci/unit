use log::info;
use tonic::transport::Server;
use unit_index::Index;
use unit_pubsub::PubSub;
use unit_utils::Result;

use crate::{
    auth,
    service::admin::{AdminServer, AdminService},
    service::crossbar::{CrossbarServer, CrossbarService},
};

pub async fn start_grpc_api(addr: String, index: Index, pubsub: PubSub) -> Result<()> {
    let addr = addr.parse()?;

    let admin_service = AdminService::new(index);
    let admin_server = AdminServer::with_interceptor(admin_service, auth::check_auth);

    let crossbar_service = CrossbarService::new(pubsub);
    let crossbar_server = CrossbarServer::with_interceptor(crossbar_service, auth::check_auth);

    info!("grpc api listening on http://{}", &addr);

    Server::builder()
        .add_service(admin_server)
        .add_service(crossbar_server)
        .serve(addr)
        .await?;

    Ok(())
}
