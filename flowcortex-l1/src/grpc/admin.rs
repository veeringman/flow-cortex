use crate::rpc::SharedNode;

#[derive(Clone)]
pub struct AdminService {
    pub node: SharedNode,
}
