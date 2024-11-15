use near_workspaces::{network::Sandbox, sandbox, Worker};

pub async fn get_near_instance() -> Result<Worker<Sandbox>, Box<dyn std::error::Error>> {
    // Spin up a local Near node.
    let sandbox_worker: Worker<Sandbox> = sandbox().await?;

    // Configure sandbox accounts
    let _alice = sandbox_worker.dev_create_account().await?;
    let _bob = sandbox_worker.dev_create_account().await?;

    Ok(sandbox_worker)
}
