/**
 * Wallet Module - Handles cryptographic operations and transaction signing
 */

/**
 * Current connected wallet state
 */
let currentWallet = null;

/**
 * Convert hex string to Uint8Array
 */
function hexToBytes(hex) {
    if (hex.startsWith('0x')) hex = hex.slice(2);
    if (hex.length % 2) throw new Error('Odd hex length');
    
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < bytes.length; i++) {
        bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
    }
    return bytes;
}

/**
 * Convert Uint8Array to hex string
 */
function bytesToHex(bytes) {
    return Array.from(bytes)
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}

/**
 * Connect wallet with Ed25519 keypair
 */
export function connectWallet(pubkeyHex, privkeyHex) {
    try {
        currentWallet = {
            pubkey: hexToBytes(pubkeyHex),
            privkey: hexToBytes(privkeyHex),
            address: pubkeyHex.slice(0, 16) + '...'
        };
        return { success: true, wallet: currentWallet };
    } catch (error) {
        return { success: false, error: error.message };
    }
}

/**
 * Disconnect wallet
 */
export function disconnectWallet() {
    currentWallet = null;
}

/**
 * Check if wallet is connected
 */
export function isWalletConnected() {
    return currentWallet !== null;
}

/**
 * Get current wallet info
 */
export function getWallet() {
    return currentWallet;
}

/**
 * Sign a transaction using Ed25519
 */
export function signTransaction(tx) {
    if (!currentWallet) {
        throw new Error('Wallet not connected');
    }
    
    if (typeof nacl === 'undefined') {
        throw new Error('TweetNaCl library not loaded');
    }
    
    // Serialize transaction
    const txBytes = new TextEncoder().encode(JSON.stringify(tx));
    
    // Sign with Ed25519
    const signature = nacl.sign.detached(txBytes, currentWallet.privkey);
    
    return {
        caller: tx.kind?.Transfer?.from || tx.kind?.Mint?.to || 'unknown',
        pubkey: Array.from(currentWallet.pubkey),
        signature: Array.from(signature),
        tx
    };
}

/**
 * Create a transfer transaction
 */
export function createTransferTx(from, to, token, amount) {
    return {
        kind: {
            Transfer: { from, to, token, amount }
        },
        rw_set: { reads: [], writes: [] },
        proof: null
    };
}

/**
 * Create a mint transaction
 */
export function createMintTx(to, token, amount) {
    return {
        kind: {
            Mint: { to, token, amount }
        },
        rw_set: { reads: [], writes: [] },
        proof: null
    };
}

/**
 * Create an anchor proof transaction
 */
export function createAnchorTx(id, proofBase64) {
    return {
        kind: {
            AnchorProof: {
                id,
                proof: Array.from(atob(proofBase64))
            }
        },
        rw_set: { reads: [], writes: [] },
        proof: null
    };
}
