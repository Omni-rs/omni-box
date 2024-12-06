use omni_box::OmniBox;

#[tokio::main]
async fn main() {
    let omni_box = OmniBox::new().await;

    println!("Omnibox up and running!");

    let btc_context = omni_box.btc_context;
    let evm_context = omni_box.evm_context;
    let near_context = omni_box.near_context;

    println!("BTC Context: {:?}", btc_context);
    println!("EVM Context: {:?}", evm_context);
    println!("NEAR Context: {:?}", near_context);

    // Print the BTC addresses
    let alice_legacy = btc_context.alice_legacy;
    let alice_segwit = btc_context.alice_segwit;

    println!("Alice Legacy: {:?}", alice_legacy);
    println!("Alice Segwit: {:?}", alice_segwit);

    let bob_legacy = btc_context.bob_legacy;
    let bob_segwit = btc_context.bob_segwit;

    println!("Bob Legacy: {:?}", bob_legacy);
    println!("Bob Segwit: {:?}", bob_segwit);

    // Print the EVM accounts
    let evm_alice = evm_context.alice;
    let evm_bob = evm_context.bob;

    println!("EVM Alice: {:?}", evm_alice);
    println!("EVM Bob: {:?}", evm_bob);

    // Print the NEAR accounts
    let near_alice = near_context.alice;
    let near_bob = near_context.bob;

    println!("NEAR Alice: {:?}", near_alice);
    println!("NEAR Bob: {:?}", near_bob);
}
