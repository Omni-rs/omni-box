use omni_box::OmniBox;

#[tokio::main]
async fn main() {
    let omni_box = OmniBox::new().await;

    println!("Omnibox up and running!");

    let anvil_chain = omni_box.chains.get(&Network::Ethereum).unwrap();
}
