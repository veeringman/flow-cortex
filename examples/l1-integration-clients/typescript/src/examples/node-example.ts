import { FlowCortexL1Client } from '../client';

/**
 * Node.js example demonstrating all FlowCortex L1 REST API operations
 */
async function main() {
  const client = new FlowCortexL1Client('http://127.0.0.1:3000');

  try {
    console.log('=== FlowCortex L1 REST Client Examples ===\n');

    // Example 1: Create accounts
    console.log('1. Creating accounts...');
    await client.createAccount('alice');
    await client.createAccount('bob');
    console.log('  ✓ Accounts created\n');

    // Example 2: Get balance
    console.log('2. Getting balance...');
    const balance = await client.getBalance('admin', 'Proof');
    console.log(`  Account: ${balance.account}`);
    console.log(`  Token: ${balance.token}`);
    console.log(`  Balance: ${balance.balance}\n`);

    // Example 3: Mint tokens
    console.log('3. Minting tokens...');
    await client.mint({
      caller: 'admin',
      to: 'alice',
      token: 'Proof',
      amount: 1000,
    });
    console.log('  ✓ Tokens minted\n');

    // Example 4: Check minted balance
    const newBalance = await client.getBalance('alice', 'Proof');
    console.log(`4. Alice's new balance: ${newBalance.balance}\n`);

    // Example 5: Transfer tokens
    console.log('5. Transferring tokens...');
    await client.transfer({
      from: 'alice',
      to: 'bob',
      token: 'Proof',
      amount: 100,
    });
    console.log('  ✓ Tokens transferred\n');

    // Example 6: Get pending pool
    console.log('6. Getting pending transaction pool...');
    const pool = await client.getPool();
    console.log(`  Pool: ${JSON.stringify(pool, null, 2)}\n`);

    // Example 7: List blocks
    console.log('7. Listing blocks...');
    const blocks = await client.listBlocks();
    console.log(`  Total blocks: ${blocks.length}`);
    blocks.slice(0, 3).forEach((block, i) => {
      console.log(`    Block ${i}: height=${block.height}, txs=${block.transactions?.length || 0}`);
    });
    console.log('');

    // Example 8: Get snapshot
    console.log('8. Getting snapshot...');
    const snapshot = await client.getSnapshot();
    console.log(`  Root: ${snapshot.root}\n`);

    // Example 9: Create block
    console.log('9. Creating block...');
    const block = await client.createBlock();
    console.log(`  New block: height=${block.height}\n`);

    // Example 10: Upload capsule
    console.log('10. Uploading capsule...');
    const sampleCode = Buffer.from('sample capsule code').toString('base64');
    await client.uploadCapsule('my_capsule_1', sampleCode);
    console.log('  ✓ Capsule uploaded\n');

    // Example 11: List capsules
    console.log('11. Listing capsules...');
    const capsules = await client.listCapsules();
    console.log(`  Total capsules: ${capsules.capsules.length}`);
    capsules.capsules.forEach((id, i) => {
      console.log(`    Capsule ${i}: ${id}`);
    });
    console.log('');

    // Example 12: Invoke capsule
    console.log('12. Invoking capsule...');
    const input = Buffer.from('test input').toString('base64');
    const output = await client.invokeCapsule('my_capsule_1', input);
    console.log(`  Output: ${output.output}\n`);

    console.log('=== All examples completed successfully! ===');
  } catch (error: any) {
    console.error('Error:', error.response?.data || error.message);
    process.exit(1);
  }
}

main();
