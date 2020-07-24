fn main() -> Result<(), std::io::Error> {
    let contract_abi = <contract::VecExample as liquid_lang::GenerateABI>::generate_abi();
    let mut final_abi = Vec::with_capacity(contract_abi.external_fn_abis.len() + 1);
    final_abi.push(serde_json::to_string(&contract_abi.constructor_abi)?);
    final_abi.extend(
        contract_abi
            .external_fn_abis
            .iter()
            .map(|abi| serde_json::to_string(abi))
            .collect::<Result<Vec<_>, _>>()
            .expect("the ABI of external functions must be a well-formatted JSON object"),
    );
    let contents = final_abi.join(",");
    let contents = format!("[{}]", contents);
    std::fs::create_dir("target").ok();
    std::fs::write("target/vec_example.json", contents)?;
    Ok(())
}
