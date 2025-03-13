use alloy::sol;
use alloy::sol_types::SolCall;
use anyhow::Result;
use node_db::NodeDB;
use revm::primitives::{address, U256};
use revm::primitives::TransactTo;
use revm::Evm;
use std::time::Instant;
use alloy::primitives::U160;


sol!(
    #[sol(rpc)]
    contract ERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }
);

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract V3Quoter {
        struct QuoteExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint256 amountIn;
            uint24 fee;
            uint160 sqrtPriceLimitX96;
        }
        function quoteExactInputSingle(QuoteExactInputSingleParams memory params)
        external
        returns (
            uint256 amountOut,
            uint160 sqrtPriceX96After,
            uint32 initializedTicksCrossed,
            uint256 gasEstimate
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Addresses
    let quoter = address!("3d4e44Eb1374240CE5F1B871ab261CD16335B76a");
    let weth = address!("4200000000000000000000000000000000000006");
    let usdc = address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");

    // Build the database. AlloyDB is a database implementation that will fetch any missing data
    // from the rpc
    let db_path = "/mnt/data/nodes/base/data";
    let mut db = NodeDB::new(db_path.to_string()).unwrap();

    // Setup an evm instance with the database
    let mut evm = Evm::builder().with_db(&mut db).build();

    // Construct the calldata for the router
    let calldata = V3Quoter::quoteExactInputSingleCall {
        params: V3Quoter::QuoteExactInputSingleParams {
            tokenIn: weth,
            tokenOut: usdc,
            fee: 3000.try_into().unwrap(),
            amountIn: U256::from(1e18),
            sqrtPriceLimitX96: U160::ZERO,
        },
    }
    .abi_encode();

    evm.tx_mut().data = calldata.into();
    evm.tx_mut().transact_to = TransactTo::Call(quoter);

    for _ in 0..20 {
        let start = Instant::now();
        let _ = evm.transact().unwrap();
        let elapsed = start.elapsed();

        println!("Took: {:?}", elapsed);
    }
    Ok(())
}
