use alloy::eips::BlockId;
use alloy::providers::ProviderBuilder;
use alloy::sol;
use alloy::sol_types::{SolCall, SolValue};
use anyhow::Result;
use revm::db::{AlloyDB, CacheDB};
use revm::primitives::{address, keccak256, Bytes, U256};
use revm::primitives::{AccountInfo, TransactTo};
use revm::Evm;
use std::time::Instant;

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
    contract UniswapV2 {
        function swapExactTokensForTokens(
          uint amountIn,
          uint amountOutMin,
          address[] calldata path,
          address to,
          uint deadline
        ) external returns (uint[] memory amounts);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://eth.merkle.io";
    let provider = ProviderBuilder::new().on_http(url.parse()?);

    // Addresses
    let account = address!("18B06aaF27d44B756FCF16Ca20C1f183EB49111f");
    let weth = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
    let usdc = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
    let router = address!("7a250d5630B4cF539739dF2C5dAcb4c659F2488D");

    // Build the database. AlloyDB is a database implementation that will fetch any missing data
    // from the rpc
    let alloy_db = AlloyDB::new(provider, BlockId::latest()).unwrap();
    let mut db = CacheDB::new(alloy_db);

    // The database will start off as a clean slate. We are swapping from weth ->usdc so we want to
    // mock a starting balance. We can do this by deriving the appropriate slot and populating it
    // with a balacne
    let one_ether = U256::from(1_000_000_000_000_000_000u128);
    let hashed_acc_balance_slot = keccak256((account, U256::from(3)).abi_encode());
    db.insert_account_storage(weth, hashed_acc_balance_slot.into(), U256::from(1e18))?;
    let acc_info = AccountInfo {
        nonce: 0_u64,
        balance: one_ether,
        code_hash: keccak256(Bytes::new()),
        code: None,
    };
    db.insert_account_info(account, acc_info);

    // Approve the router to spend the balance. You can just compute the slot and insert it into
    // the DB to save the following code, but I provide it for completeness
    // favor of demonstration we will execute it with revm
    let mut evm = Evm::builder()
        .with_db(&mut db)
        .modify_tx_env(|tx| {
            tx.caller = account;
            tx.value = U256::ZERO;
        })
        .build();

    // Approve the router to spend 1 ETH
    let approve_calldata = ERC20::approveCall {
        spender: router,
        amount: U256::from(1e18),
    }
    .abi_encode();
    evm.tx_mut().transact_to = TransactTo::Call(weth);
    evm.tx_mut().data = approve_calldata.into();
    evm.transact_commit()?;

    // Router is now approved to spend our weth!
    let calldata = UniswapV2::swapExactTokensForTokensCall {
        amountIn: U256::from(1e18),
        amountOutMin: U256::ZERO,
        path: vec![weth, usdc],
        to: account,
        deadline: U256::MAX,
    }
    .abi_encode();

    evm.tx_mut().data = calldata.into();
    evm.tx_mut().transact_to = TransactTo::Call(router);
    let ref_tx = evm.transact().unwrap();

    let result = ref_tx.result;
    println!("{:?}", result);

    Ok(())
}
