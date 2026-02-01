use crate::alc::verify;
use crate::qct::QCTNode;

pub async fn verify_subtree(root: &QCTNode) -> bool {
    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        if !verify(&node.commitment, node.frequency.sum, &node.blinding) {
            return false;
        }

        for child in node.children.values() {
            stack.push(child);
        }
    }

    true
}
