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
}
