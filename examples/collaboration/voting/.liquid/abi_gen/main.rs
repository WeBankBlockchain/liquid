fn main() -> Result<(), std::io::Error> {
    let collaboration_abi =
        <collaboration::__LIQUID_ABI_GEN as liquid_lang::GenerateAbi>::generate_abi();
    std::fs::create_dir("target").ok();
    std::fs::write(
        "target/voting.abi",
        serde_json::to_string(&collaboration_abi.contract_abis)?,
    )?;
    Ok(())
}
