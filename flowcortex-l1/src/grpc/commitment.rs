use crate::rpc::SharedNode;

#[derive(Clone)]
pub struct CommitmentAnchorService {
    pub node: SharedNode,
}
