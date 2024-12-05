use near_workspaces::network::Sandbox;
use near_workspaces::{Account, Worker};

use crate::clients::get_near_instance;

#[derive(Debug)]
pub struct NearTestContext {
    pub client: Worker<Sandbox>,
    pub alice: Account,
    pub bob: Account,
}

/// The NearTestContext is a struct that contains the Near client and two accounts for testing.
/// Each account is created with a default balance of 100 NEAR tokens.
/// You can access the Near client, Alice's account, and Bob's account using the client(), alice(), and bob() methods.
/// Additionally you can use the client() method to interact with the Near client.
impl NearTestContext {
    pub async fn new() -> Self {
        let sandbox_worker: near_workspaces::Worker<Sandbox> = get_near_instance().await.unwrap();

        // Configure sandbox accounts
        let alice = sandbox_worker.dev_create_account().await.unwrap();
        let bob = sandbox_worker.dev_create_account().await.unwrap();

        NearTestContext {
            client: sandbox_worker,
            alice,
            bob,
        }
    }

    pub fn client(&self) -> &Worker<Sandbox> {
        &self.client
    }

    pub fn alice(&self) -> &Account {
        &self.alice
    }

    pub fn bob(&self) -> &Account {
        &self.bob
    }

    pub async fn create_account(&self) -> Account {
        self.client.dev_create_account().await.unwrap()
    }
}
