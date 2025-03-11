use alloy::primitives::{address, U256};
use alloy::providers::{Provider, ProviderBuilder};
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

// Function to benchmark a provider with multiple iterations
async fn benchmark_provider(provider_url: &str, iterations: usize) -> Result<(Vec<u128>, f64)> {
    let provider = ProviderBuilder::new().on_http(provider_url.parse()?);
    let contract_address = address!("7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
    let weth = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    let usdc = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    
    let uniswap = UniswapV2::new(contract_address, provider);
    
    let mut times = Vec::with_capacity(iterations);
    let mut total_time = 0;
    
    for i in 0..iterations {
        let builder = uniswap.getAmountsOut(U256::from(1e18), vec![weth, usdc]);
        
        let start = Instant::now();
        let output = builder.call().await?;
        let elapsed = start.elapsed().as_millis();
        
        total_time += elapsed;
        times.push(elapsed);
        
        println!("Iteration {}: {:?} - {} ms", i + 1, output, elapsed);
    }
    
    let avg_time = total_time as f64 / iterations as f64;
    Ok((times, avg_time))
}

#[tokio::main]
async fn main() -> Result<()> {
    let external_url = "https://eth.merkle.io";
    let local_url = "http://localhost:8545";
    let iterations = 3; // Number of iterations for each benchmark
    
    println!("Benchmarking external RPC ({})", external_url);
    let (external_times, external_avg) = match benchmark_provider(external_url, iterations).await {
        Ok(result) => result,
        Err(e) => {
            println!("Error benchmarking external RPC: {}", e);
            (vec![], 0.0)
        }
    };
    
    println!("\nBenchmarking local RPC ({})", local_url);
    let (local_times, local_avg) = match benchmark_provider(local_url, iterations).await {
        Ok(result) => result,
        Err(e) => {
            println!("Error benchmarking local RPC: {}", e);
            (vec![], 0.0)
        }
    };
    
    println!("\n===== Benchmark Results =====");
    println!("External RPC:");
    for (i, time) in external_times.iter().enumerate() {
        println!("  Iteration {}: {} ms", i + 1, time);
    }
    println!("  Average: {:.2} ms", external_avg);
    
    println!("\nLocal RPC:");
    for (i, time) in local_times.iter().enumerate() {
        println!("  Iteration {}: {} ms", i + 1, time);
    }
    println!("  Average: {:.2} ms", local_avg);
    
    // Calculate overall average
    if !external_times.is_empty() && !local_times.is_empty() {
        let overall_avg = (external_avg + local_avg) / 2.0;
        println!("\nOverall Average: {:.2} ms", overall_avg);
    } else {
        println!("\nCannot calculate overall average due to errors");
    }
    
    Ok(())
}
