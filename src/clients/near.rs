use near_workspaces::{network::Sandbox, sandbox, Worker};

pub async fn get_near_instance() -> Result<Worker<Sandbox>, Box<dyn std::error::Error>> {
    // Spin up a local Near node.
    let sandbox_worker: Worker<Sandbox> = sandbox().await?;

    Ok(sandbox_worker)
}
