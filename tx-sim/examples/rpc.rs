use alloy::primitives::{address, U256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use anyhow::Result;
use std::time::Instant;

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    contract UniswapV2 {
        function getAmountsOut(uint amountIn, address[] memory path) public view returns (uint[] memory amounts);
    }
}

// Simulate a transaction using the eth_simualte
#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://eth.merkle.io";
    let ca = address!("7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
    let provider = ProviderBuilder::new().on_http(url.parse()?);

    let weth = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    let usdc = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

    let uniswap = UniswapV2::new(ca, provider);

    let builder = uniswap.getAmountsOut(U256::from(1e18), vec![weth, usdc]);

    let start = Instant::now();
    let output = builder.call().await?;
    let end = start.elapsed();

    println!("{:?} {}", output, end.as_millis());

    Ok(())
}
