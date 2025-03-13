use alloy::primitives::{address, U160, U256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use anyhow::Result;
use std::time::Instant;

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
    let url = "http://localhost:8547";
    let provider = ProviderBuilder::new().on_http(url.parse()?);

    let quoter = address!("3d4e44Eb1374240CE5F1B871ab261CD16335B76a"); // Uniswap V3 Quoter
    let weth = address!("4200000000000000000000000000000000000006");
    let usdc = address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
    
    let uniswap_quoter = V3Quoter::new(quoter, provider);
    
    let params = V3Quoter::QuoteExactInputSingleParams {
        tokenIn: weth,
        tokenOut: usdc,
        amountIn: U256::from(1e18),
        fee: 3000.try_into().unwrap(),
        sqrtPriceLimitX96: U256::ZERO.to::<U160>()
    };
    
    for _ in 0..20 {
        let start = Instant::now();
        let _ = uniswap_quoter.quoteExactInputSingle(params.clone()).call().await?;
        let elapsed = start.elapsed();

        println!("Total: {:?}", elapsed);
    }
    
    Ok(())
}