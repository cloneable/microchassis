use crate::proto;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct UserServiceImpl {}

#[tonic::async_trait]
impl proto::user_service_server::UserService for UserServiceImpl {
    async fn get_user(
        &self,
        request: Request<proto::GetUserRequest>,
    ) -> Result<Response<proto::GetUserResponse>, Status> {
        let reply = proto::GetUserResponse {
            user: Some(proto::User {
                id: request.get_ref().id,
                name: "admin".to_string(),
            }),
        };
        Ok(Response::new(reply))
    }
}
