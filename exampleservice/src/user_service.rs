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
        let get_user_request = request.get_ref();
        log::info!("GetUser called with ID {}", get_user_request.id);

        if get_user_request.id <= 0 {
            return Err(Status::invalid_argument(
                "GetUserRequest.id invalid or unset",
            ));
        }
        if get_user_request.id != 1 {
            return Err(Status::not_found("User not found"));
        }

        Ok(Response::new(proto::GetUserResponse {
            user: Some(proto::User {
                id: get_user_request.id,
                name: "admin".to_string(),
            }),
        }))
    }
}
