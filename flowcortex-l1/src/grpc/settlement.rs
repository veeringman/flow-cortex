use crate::rpc::SharedNode;

#[derive(Clone)]
pub struct SettlementService {
    pub node: SharedNode,
}
