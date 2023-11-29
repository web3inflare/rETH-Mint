use rand::prelude::*;
use hex::{decode, encode};
use ethers::utils::keccak256;
use std::sync::{Arc, Mutex};
use std::{thread, sync::atomic::{AtomicBool, Ordering}};
use ethers::types::{Eip1559TransactionRequest, U256, Bytes,Chain};
use ethers::prelude::SignerMiddleware;
use ethers::signers::{LocalWallet};
use ethers::signers::Signer;
use ethers::middleware::{Middleware};
use serde::Deserialize;
use ethers::providers::{Provider, Http};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use reqwest;
use log::{info, warn, error};
use config::File;
use config::Config;

const CURRENT_CHALLENGE_HEX: &str = "0x7245544800000000000000000000000000000000000000000000000000000000";


#[derive(Deserialize, Debug)]
struct  GasResponse {
    data: GasData,
}
#[derive(Deserialize, Debug)]
struct GasData {
    rapid : u64, // 超快
    fast: u64, //快
    standard: u64, // 标准
    slow:u64, // 慢
}
// 配置文件 
#[derive(Deserialize)]
struct MyConfig {
    private_key: String,// 密钥
    max_transactions: u32, // MINT次数
    rpc_url: String, // RPC接口
    max_attempts: u32, //检测上链时间
    num_threads: u32, // ID检测线程
    network:String, //目标网络
    difficulty:String,// 难度
    gas_type:String, // Gas费用等级
}
fn find_solution(&threads: &u32,current_challenge_hex: &str,difficulty: &str) -> String {
    let found = Arc::new(AtomicBool::new(false));
    let solution = Arc::new(Mutex::new(None));
    let current_challenge = decode(&current_challenge_hex[2..]).unwrap();
    let difficulty = difficulty.to_owned();

    let mut handles = vec![];

    for _ in 0..threads {
        let found_clone = Arc::clone(&found);
        let solution_clone = Arc::clone(&solution);
        let current_challenge_clone = current_challenge.clone();
        let  difficulty_clone  = difficulty.clone();

        let handle = thread::spawn(move || {
            let mut rng = thread_rng();
            while !found_clone.load(Ordering::SeqCst) {
                let random_value: [u8; 32] = rng.gen();
                let potential_solution_hex = encode(random_value);
                let potential_solution = decode(&potential_solution_hex).unwrap();

                let mut input_encoded = [0u8; 64];
                input_encoded[..32].copy_from_slice(&potential_solution);
                input_encoded[32..].copy_from_slice(&current_challenge_clone);

                let hashed_solution = keccak256(&input_encoded);
                let hashed_solution_hex = "0x".to_owned() + &encode(&hashed_solution);

                if hashed_solution_hex.starts_with(&difficulty_clone) {
                    found_clone.store(true, Ordering::SeqCst);
                    let mut sol = solution_clone.lock().unwrap();
                    *sol = Some(potential_solution_hex);
                    break;
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let sol = solution.lock().unwrap();
    let found_solution = sol.as_ref().unwrap();
    let id_hex = "0x".to_owned() + found_solution;
    let json_data = format!(r#"{{"p":"rerc-20","op":"mint","tick":"rETH","id":"{}","amt":"10000"}}"#, id_hex);
    let data_string = "data:application/json,".to_owned() + &json_data.to_string();
    encode(data_string.as_bytes())
}

async fn get_max_fee_per_gas(gas_type: &str) -> Result<u64, reqwest::Error>{
    // 获取最新的gas费用
    let url = "https://beaconcha.in/api/v1/execution/gasnow";
    loop {
        match reqwest::get(url).await{
            Ok(resp) =>match resp.json::<GasResponse>().await {
                Ok(gas_response) =>{
                    let gas_price = match gas_type {
                        "rapid" => gas_response.data.rapid,
                        "fast" => gas_response.data.fast,
                        "standard" => gas_response.data.standard,
                        "slow" => gas_response.data.slow,
                        _ => gas_response.data.standard, // 默认使用 standard
                    };
                    return Ok(gas_price)
                }
                Err(e)=>{
                    println!("[-] 获取Gas费用失败，重试中...{:?}",e);
                    sleep(Duration::from_secs(1)).await;
                }
                
            }
            Err(e) =>{
                println!("[-] 获取Gas费用失败，重试中...{:?}",e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

}
fn load_config() -> Result<MyConfig, config::ConfigError> {
    Config::builder()
    .add_source(File::with_name("Settings.toml"))
    .build()?
    .try_deserialize::<MyConfig>()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 计算开始时间
    log4rs::init_file("log4rs.yaml", Default::default())?;
    let start_time = Instant::now();
    let cfg: MyConfig = load_config().expect("Failed to load configuration");

    let private_key: String = cfg.private_key;
    let max_transactions: u32 = cfg.max_transactions;
    let rpc_url: String = cfg.rpc_url;
    let max_attempts:u32 = cfg.max_attempts;
    let num_threads:u32 = cfg.num_threads;
    let difficulty:String =  cfg.difficulty;
    let gas_type:String =  cfg.gas_type;
    let network: String  = cfg.network.to_lowercase();
    let networks = &network.to_lowercase();
    let chain_id = match networks.as_str() {
        "mainnet" => Chain::Mainnet,
        "ropsten" => Chain::Ropsten,
        "rinkeby" => Chain::Rinkeby,
        "goerli" => Chain::Goerli,
        "kovan" => Chain::Kovan,
        _ => panic!("未知的网络名称: {}", networks),
    };
    info!("[*]目标网络: {} 难度: {} Gas_type: {} Mint数量: {} 检测时间: {} 线程: {} ",chain_id,difficulty,gas_type,max_transactions,max_attempts,num_threads);

    let wallet = private_key
    .parse::<LocalWallet>()?
    .with_chain_id(chain_id);
    // 获取钱包地址
    let wallet_address = wallet.address();
    info!("Wallet address: {:?}", wallet_address);
    // 开始循环
    // 循环次数
    let mut successful_transactions = 0;
    while successful_transactions < max_transactions {
        info!("[*] 开始第{}次发送",successful_transactions+1);
        info!("[*] 当前成功MINT数量:{}",successful_transactions);
    // 获取发送data 的数据
    let data_hex = find_solution(&num_threads,CURRENT_CHALLENGE_HEX,&difficulty);
    // 初始化rpc接口
    let provider = Provider::<Http>::try_from(
        rpc_url.clone()
    )?;
    // 初始化客户端
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    // 获取钱包信息
    let balance =provider.get_balance(wallet_address, None).await?.as_u128() as f64 / 1e18_f64;
    let nonce = provider.get_transaction_count(wallet_address, None).await?;
    if balance == 0.0 {
        panic!("[*] 钱包地址 {} 的余额不足，程序无法运行", wallet_address);
    }
    info!("[*] 钱包地址 {} nonce：{} 余额: {:18}",wallet_address,nonce,balance);
    // 自动获取gas费用
    let max_fee_per_gas = get_max_fee_per_gas(&gas_type).await?;
    // 生成交易
    let tx = Eip1559TransactionRequest::new()
    .from(wallet_address)
    .to(wallet_address)
    .gas(U256::from(32000)) // 设置gas限制
    .max_priority_fee_per_gas(U256::from(2_000_000_000u64)) // 设置优先费用，默认2 Gwei
    .max_fee_per_gas(U256::from(max_fee_per_gas)) 
    .value(0) // 设置发送的金额
    .nonce(nonce) // 设置链ID
    .data(Bytes::from(hex::decode(data_hex)?))
    .chain_id(chain_id); // 设置数据    
    // 发送交易
    let pending_tx = client.send_transaction(tx, None).await?;
    // 获取hash值
    let tx_hash = pending_tx.tx_hash();
    info!("[*] 交易hash {:?}",tx_hash);
    // 检测是否成功提交hash至区块链上
    let mut receipt = None;
    let mut attempts = 0;
    // 检测30秒
    // const MAX_ATTEMPTS: u32 = 30;
    while receipt.is_none() && attempts < max_attempts {
        match provider.get_transaction_receipt(tx_hash).await {
            Ok(Some(r)) => receipt = Some(r),
            Ok(None) => {
                warn!("[*] 正在检测{:?} 状态 [未成功上链]",tx_hash);
                sleep(Duration::from_secs(1)).await;
                attempts += 1;
            },
            Err(e) => {
                error!("[-] 发送失败错误原因: {:?}", e);
                break;
            }
        }
    }
    // 判断结果
    match receipt {
        Some(receipt) => {
            info!("[*] 发送成功 Hash {:?} 状态[成功上链]", receipt.transaction_hash);
            successful_transactions +=1;
        },
        None => error!("[-]发送失败 {:?}",tx_hash),
    }
    };
    let elapsed_time = start_time.elapsed().as_secs();
    info!("[*] 程序运行总时间：{}秒", elapsed_time);
    Ok(())
}
