use omni_box::OmniBox;

#[tokio::main]
async fn main() {
    let _omni_box = OmniBox::new().await;

    println!("Omnibox up and running!");
}
