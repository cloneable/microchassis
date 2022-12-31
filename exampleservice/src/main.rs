mod proto;
mod user_service;

use tonic::transport::Server;
use tracing as log;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let shutdown_broadcast = microchassis::init()?;

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
		.set_serving::<proto::user_service_server::UserServiceServer<
			user_service::UserServiceImpl,
		>>()
        .await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(
            tonic_health::proto::GRPC_HEALTH_V1_FILE_DESCRIPTOR_SET,
        )
        .build()?;

    let user_service = user_service::UserServiceImpl::default();

    log::info!("Server starting");

    let shutdown_signal = shutdown_broadcast.subscribe();

    Server::builder()
        .add_service(reflection_service)
        .add_service(health_service)
        .add_service(proto::user_service_server::UserServiceServer::new(user_service))
        .serve_with_shutdown("[::1]:50051".parse()?, shutdown_signal.recv())
        .await?;

    log::info!("Server shutting down");

    microchassis::shutdown()?;

    Ok(())
}
