use crate::rpc::SharedNode;

#[derive(Clone)]
pub struct ProofVerifierService {
    pub node: SharedNode,
}
