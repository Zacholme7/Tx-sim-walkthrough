use alloy::sol;
use alloy::sol_types::SolCall;
use anyhow::Result;
use node_db::NodeDB;
use revm::primitives::{address, U256};
use revm::primitives::TransactTo;
use revm::Evm;
use std::time::Instant;

sol!(
    #[sol(rpc)]
    contract CurveOut {
        function get_dy(uint256 i, uint256 j, uint256 dx) external view returs (uint256);
    }
);

#[tokio::main]
async fn main() -> Result<()> {
    let pool = address!("11C1fBd4b3De66bC0565779b35171a6CF3E71f59");

    let db_path = "/mnt/data/nodes/base/data";
    let mut db = NodeDB::new(db_path.to_string()).unwrap();

    // the function calldata
    let calldata = CurveOut::get_dyCall {
        i: U256::from(0),
        j: U256::from(1), 
        dx: U256::from(1e18)
    }.abi_encode();

    let mut evm = Evm::builder()
        .with_db(&mut db)
        .modify_tx_env(|tx| {
            tx.transact_to = TransactTo::Call(pool);
            tx.data = calldata.into();
            tx.value = U256::ZERO;
        })
        .build();

    // do the transaction
    for _ in 0..20 {
        let start = Instant::now();
        let _ = evm.transact().unwrap();
        let elapsed = start.elapsed();

        println!("Took: {:?}", elapsed);
    }

    Ok(())
}
