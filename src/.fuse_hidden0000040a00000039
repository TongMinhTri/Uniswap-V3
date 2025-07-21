use ethers::{
    prelude::*,
    types::{Address, U256},
};

abigen!(
    IERC20Minimal,
    r#"[
        function balanceOf(address) view returns (uint256)
        function transfer(address, uint256) returns (bool)
    ]"#,
);

pub async fn safe_transfer(
    contract: IERC20Minimal<Provider<Http>>,
    to: Address,
    value: U256,
) -> Result<(), ContractError<Provider<Http>>> {
    let tx = contract.transfer(to, value);

    Ok(())
}
