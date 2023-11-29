// keccak_test.rs
use hex::{decode, encode};
use ethers::utils::keccak256;

#[test]
// 用于测试随机值计算出的keccak256 是否符合0x7777
fn test_keccak256_hash() {
    let current_challenge_hex = "0x7245544800000000000000000000000000000000000000000000000000000000";
    // 随机32位
    // 检测ID
    let potential_solution_hex = "0x367a23796ff00d4522d0ec736609f8d8b31afef78d339ca0ae081b0aff388b4b";
    let current_challenge = decode(&current_challenge_hex[2..]).unwrap();
    let potential_solution = decode(&potential_solution_hex[2..]).unwrap();

    let mut input_encoded = [0u8; 64];
    input_encoded[..32].copy_from_slice(&potential_solution);
    input_encoded[32..].copy_from_slice(&current_challenge);

    let hashed_solution = keccak256(&input_encoded);
    let hashed_solution_hex = "0x".to_owned() + &encode(&hashed_solution);
    println!("{}",hashed_solution_hex);
    // 对应难度
    assert!(hashed_solution_hex.starts_with("0x7777"), "potential_solution 实际hash{:?}", hashed_solution_hex);
}