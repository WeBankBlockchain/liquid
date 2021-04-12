use std::env;

fn main() -> Result<(), std::io::Error> {
    let collaboration_abi =
        <collaboration::__LIQUID_ABI_GEN as liquid_lang::GenerateAbi>::generate_abi();
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or("target".into());
    std::fs::create_dir(&target_dir).ok();
    std::fs::write(
        format!("{}/voting.abi", target_dir),
        serde_json::to_string(&collaboration_abi.contract_abis)?,
    )?;
    Ok(())
}
