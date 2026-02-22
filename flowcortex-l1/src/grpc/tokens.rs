use crate::rpc::SharedNode;

#[derive(Clone)]
pub struct TokensService {
    pub node: SharedNode,
}
