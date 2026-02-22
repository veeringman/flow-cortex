import axios, { AxiosInstance } from 'axios';

/**
 * FlowCortex L1 Client for REST API
 * Provides methods to interact with the FlowCortex L1 node
 */
export class FlowCortexL1Client {
  private baseURL: string;
  private client: AxiosInstance;

  constructor(nodeUrl: string = 'http://127.0.0.1:3000') {
    this.baseURL = nodeUrl;
    this.client = axios.create({
      baseURL: this.baseURL,
      timeout: 10000,
    });
  }

  /**
   * Create a new account
   */
  async createAccount(account: string): Promise<{ success: boolean }> {
    const response = await this.client.post('/account', { account });
    return { success: response.status === 201 };
  }

  /**
   * Get account balance
   */
  async getBalance(account: string, token: 'Proof' | 'FloweR' | 'proof' | 'flower'): Promise<{ account: string; token: string; balance: number }> {
    const response = await this.client.get(`/balance/${account}/${String(token).toLowerCase()}`);
    return response.data;
  }

  /**
   * Mint tokens to an account
   */
  async mint(params: {
    caller: string;
    to: string;
    token: 'Proof' | 'FloweR' | 'proof' | 'flower';
    amount: number;
    rw_set?: any;
    proof?: any;
  }): Promise<void> {
    await this.client.post('/mint', {
      ...params,
      token: String(params.token).toLowerCase(),
    });
  }

  /**
   * Transfer tokens between accounts
   */
  async transfer(params: {
    from: string;
    to: string;
    token: 'Proof' | 'FloweR' | 'proof' | 'flower';
    amount: number;
    rw_set?: any;
    proof?: any;
  }): Promise<void> {
    await this.client.post('/transfer', {
      ...params,
      token: String(params.token).toLowerCase(),
    });
  }

  /**
   * Submit a signed transaction
   */
  async submitTx(params: {
    caller: string;
    pubkey: string | Uint8Array;
    signature: string | Uint8Array;
    tx: any;
  }): Promise<void> {
    // Convert to base64 if needed
    const payload = {
      ...params,
      pubkey: typeof params.pubkey === 'string' ? params.pubkey : Buffer.from(params.pubkey).toString('base64'),
      signature: typeof params.signature === 'string' ? params.signature : Buffer.from(params.signature).toString('base64'),
    };
    await this.client.post('/tx', payload);
  }

  /**
   * Get pending transaction pool
   */
  async getPool(): Promise<{ pending: any }> {
    const response = await this.client.get('/pool');
    return response.data;
  }

  /**
   * Create a new block
   */
  async createBlock(): Promise<{ height: number; transactions: any[] }> {
    const response = await this.client.post('/block');
    return response.data;
  }

  /**
   * List all blocks
   */
  async listBlocks(): Promise<{ height: number; transactions: any[] }[]> {
    const response = await this.client.get('/blocks');
    return response.data;
  }

  /**
   * Get current state snapshot
   */
  async getSnapshot(): Promise<{ root: string }> {
    const response = await this.client.get('/snapshot');
    return response.data;
  }

  /**
   * List all anchors
   */
  async listAnchors(): Promise<{ anchors: string[] }> {
    const response = await this.client.get('/anchors');
    return response.data;
  }

  /**
   * Get a specific anchor by ID
   */
  async getAnchor(id: string): Promise<{ id: string; proof: string }> {
    const response = await this.client.get(`/anchor/${id}`);
    return response.data;
  }

  /**
   * Upload a capsule (wasm/bytecode)
   */
  async uploadCapsule(id: string, codeBase64: string): Promise<{ success: boolean }> {
    const response = await this.client.post('/capsule', {
      id,
      code: codeBase64,
    });
    return response.data;
  }

  /**
   * List all capsules
   */
  async listCapsules(): Promise<{ capsules: string[] }> {
    const response = await this.client.get('/capsule');
    return response.data;
  }

  /**
   * Invoke a capsule with input data
   */
  async invokeCapsule(id: string, inputBase64: string): Promise<{ output: string }> {
    const response = await this.client.post(`/capsule/${id}/invoke`, {
      input: inputBase64,
    });
    return response.data;
  }
}

export { AxiosInstance };
