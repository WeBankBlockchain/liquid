fn main() -> Result<(), std::io::Error> {
    let collaboration_abi =
        <collaboration::__LiquidTicTacToe as liquid_lang::GenerateABI>::generate_abi();
    std::fs::create_dir("target").ok();
    std::fs::write(
        "target/tic_tac_toe.abi",
        serde_json::to_string(&collaboration_abi.contract_abis)?,
    )?;
    Ok(())
}
