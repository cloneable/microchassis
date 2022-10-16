use crate::proto;
use tonic::{Request, Response, Status};
use tracing as log;

#[derive(Default)]
pub struct UserServiceImpl {}

#[tonic::async_trait]
impl proto::user_service_server::UserService for UserServiceImpl {
    #[tracing::instrument(skip(self), err)]
    async fn get_user(
        &self,
        request: Request<proto::GetUserRequest>,
    ) -> Result<Response<proto::GetUserResponse>, Status> {
        log::info!("GetUser called with ID {}", request.get_ref().id);
        let reply = proto::GetUserResponse {
            user: Some(proto::User {
                id: request.get_ref().id,
                name: "admin".to_string(),
            }),
        };
        Ok(Response::new(reply))
    }
}
