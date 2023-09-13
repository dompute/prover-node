use std::fmt::format;
use std::sync::Arc;

use ethers::contract::EthEvent;
use ethers::core::abi::{AbiDecode, AbiEncode};
use ethers::prelude::*;

abigen!(
    Relay,
    r#"[
        struct Callback { address programContract; bytes input; bytes returnData; }
        event ComputingRequested( address indexed who, address indexed programContract, bytes input, bytes commitment)
        function invokeCallback(Callback[] calldata callback, uint256[] calldata pubInputs, bytes calldata proof) external
        function requestComputing( address programContract, bytes calldata input, bytes calldata commitment) external
    ]"#,
);

const CHAIN_ID: u64 = 31337;

pub async fn listen_on(url: &str) -> eyre::Result<()> {
    let provider = Provider::<Ws>::connect(url).await?;

    let wallet: Wallet<_> = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        .parse()
        .unwrap();

    println!("address: {:?}", wallet.address());

    let signer = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(CHAIN_ID),
    ));

    let mut stream = provider.subscribe_blocks().await?;
    while let Some(block) = stream.next().await {
        if let None = block.hash {
            println!("Unexpected error");
            continue;
        }

        let hash = block.hash.unwrap();
        println!("block: {:?}", hash);

        let filter = Filter::new()
            .at_block_hash(hash)
            .event(&ComputingRequestedFilter::abi_signature());

        let events = provider.get_logs(&filter).await?;

        for log in events.into_iter() {
            let tx = log.transaction_hash.unwrap();
            let tx = provider.get_transaction(tx).await?.unwrap();
            let tx: relay::RequestComputingCall =
                relay::RequestComputingCall::decode(tx.input).unwrap();

            println!("program: {:?}", tx.program_contract);
            println!("input: {:?}", tx.input);
            println!("commitment: {:?}", tx.commitment);

            let relay = Relay::new(log.address, signer.clone());

            let call = relay.invoke_callback(
                vec![relay::Callback {
                    program_contract: tx.program_contract,
                    input: tx.input,
                    return_data: "0x03".parse().unwrap(),
                }],
                vec!["0x00".parse().unwrap()],
                "0x00".parse().unwrap(),
            );
            call.send().await?;
        }
    }

    Ok(())
}
