use std::env;

fn main() -> Result<(), std::io::Error> {
    let contract_abi =
        <contract::__LIQUID_ABI_GEN as liquid_lang::GenerateAbi>::generate_abi();
    let mut final_abi = Vec::with_capacity(
        contract_abi.event_abis.len() + contract_abi.external_fn_abis.len() + 1,
    );
    final_abi.extend(
        contract_abi
            .event_abis
            .iter()
            .map(|abi| serde_json::to_string(abi))
            .collect::<Result<Vec<_>, _>>()
            .expect("the ABI of event must be a well-formatted JSON object"),
    );
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
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or("target".into());
    std::fs::create_dir(&target_dir).ok();
    std::fs::write(format!("{}/kv_table_test.abi", target_dir), contents)?;
    Ok(())
}
