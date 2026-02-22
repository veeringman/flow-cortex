import { AxiosInstance } from 'axios';
/**
 * FlowCortex L1 Client for REST API
 * Provides methods to interact with the FlowCortex L1 node
 */
export declare class FlowCortexL1Client {
    private baseURL;
    private client;
    constructor(nodeUrl?: string);
    /**
     * Create a new account
     */
    createAccount(account: string): Promise<{
        success: boolean;
    }>;
    /**
     * Get account balance
     */
    getBalance(account: string, token: 'Proof' | 'FloweR' | 'proof' | 'flower'): Promise<{
        account: string;
        token: string;
        balance: number;
    }>;
    /**
     * Mint tokens to an account
     */
    mint(params: {
        caller: string;
        to: string;
        token: 'Proof' | 'FloweR' | 'proof' | 'flower';
        amount: number;
        rw_set?: any;
        proof?: any;
    }): Promise<void>;
    /**
     * Transfer tokens between accounts
     */
    transfer(params: {
        from: string;
        to: string;
        token: 'Proof' | 'FloweR' | 'proof' | 'flower';
        amount: number;
        rw_set?: any;
        proof?: any;
    }): Promise<void>;
    /**
     * Submit a signed transaction
     */
    submitTx(params: {
        caller: string;
        pubkey: string | Uint8Array;
        signature: string | Uint8Array;
        tx: any;
    }): Promise<void>;
    /**
     * Get pending transaction pool
     */
    getPool(): Promise<{
        pending: any;
    }>;
    /**
     * Create a new block
     */
    createBlock(): Promise<{
        height: number;
        transactions: any[];
    }>;
    /**
     * List all blocks
     */
    listBlocks(): Promise<{
        height: number;
        transactions: any[];
    }[]>;
    /**
     * Get current state snapshot
     */
    getSnapshot(): Promise<{
        root: string;
    }>;
    /**
     * List all anchors
     */
    listAnchors(): Promise<{
        anchors: string[];
    }>;
    /**
     * Get a specific anchor by ID
     */
    getAnchor(id: string): Promise<{
        id: string;
        proof: string;
    }>;
    /**
     * Upload a capsule (wasm/bytecode)
     */
    uploadCapsule(id: string, codeBase64: string): Promise<{
        success: boolean;
    }>;
    /**
     * List all capsules
     */
    listCapsules(): Promise<{
        capsules: string[];
    }>;
    /**
     * Invoke a capsule with input data
     */
    invokeCapsule(id: string, inputBase64: string): Promise<{
        output: string;
    }>;
}
export { AxiosInstance };
