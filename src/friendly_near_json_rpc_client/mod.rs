use near_crypto::{InMemorySigner, PublicKey};
use near_jsonrpc_client::methods::send_tx::RpcSendTransactionRequest;
use near_jsonrpc_client::methods::tx::{
    RpcTransactionError, RpcTransactionResponse, RpcTransactionStatusRequest, TransactionInfo,
};
use near_jsonrpc_client::{methods::query::RpcQueryRequest, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::action::{Action, DeployContractAction};
use near_primitives::transaction::{Transaction, TransactionV0};
use near_primitives::types::BlockReference;
use near_primitives::views::TxExecutionStatus;
use near_primitives::{hash::CryptoHash, views::QueryRequest};
use near_sdk::AccountId;
use std::time::{Duration, Instant};

// local modules
pub mod near_network_config;

// import local modules
use near_network_config::{get_rpc_url, NearNetworkConfig};

use crate::NearAccount;

pub struct FriendlyNearJsonRpcClient {
    client: JsonRpcClient,
    account_config: NearAccount,
}

const TIMEOUT: Duration = Duration::from_secs(300);

impl FriendlyNearJsonRpcClient {
    pub fn new(network: NearNetworkConfig, account_config: NearAccount) -> Self {
        Self {
            client: Self::get_near_rpc_client(network),
            account_config,
        }
    }

    // custom functions

    /// Deploy a contract to the NEAR blockchain using the default account
    pub async fn deploy_contract(
        &self,
        contract_wasm: Vec<u8>,
    ) -> Result<RpcTransactionResponse, Box<dyn std::error::Error>> {
        let account_id = self.account_config.account_id.clone();

        let (nonce, block_hash) = self
            .get_nonce_and_block_hash(account_id.clone(), self.account_config.public_key.clone())
            .await
            .unwrap();

        let nonce = nonce + 1;

        let deploy_action = Action::DeployContract(DeployContractAction {
            code: contract_wasm,
        });

        let signer: InMemorySigner = InMemorySigner::from_secret_key(
            account_id.clone(),
            self.account_config.private_key.clone(),
        );

        let near_tx: Transaction = Transaction::V0(TransactionV0 {
            signer_id: account_id.clone(),
            public_key: signer.public_key(),
            nonce,
            receiver_id: account_id,
            block_hash,
            actions: vec![deploy_action],
        });

        let signer: near_crypto::Signer = signer.into();

        // Sign and send the transaction
        let request = RpcSendTransactionRequest {
            signed_transaction: near_tx.sign(&signer),
            wait_until: TxExecutionStatus::Final,
        };

        self.send_transaction(request).await
    }

    pub async fn send_transaction(
        &self,
        request: RpcSendTransactionRequest,
    ) -> Result<RpcTransactionResponse, Box<dyn std::error::Error>> {
        let sent_at: Instant = Instant::now();

        match self.client.call(request.clone()).await {
            Ok(response) => Ok(response),
            Err(err) => {
                if matches!(err.handler_error(), Some(RpcTransactionError::TimeoutError))
                    || err.to_string().contains("408 Request Timeout")
                {
                    let tx_hash = request.signed_transaction.get_hash();
                    let sender_account_id =
                        request.signed_transaction.transaction.signer_id().clone();
                    self.wait_for_transaction(tx_hash, sender_account_id, sent_at)
                        .await
                } else {
                    Err(err.into())
                }
            }
        }
    }

    pub fn get_near_rpc_client(network: NearNetworkConfig) -> JsonRpcClient {
        let rpc_url = get_rpc_url(network);
        JsonRpcClient::connect(rpc_url)
    }

    // private functions
    async fn wait_for_transaction(
        &self,
        tx_hash: CryptoHash,
        sender_account_id: AccountId,
        sent_at: Instant,
    ) -> Result<RpcTransactionResponse, Box<dyn std::error::Error>> {
        loop {
            let response = self
                .client
                .call(RpcTransactionStatusRequest {
                    transaction_info: TransactionInfo::TransactionId {
                        tx_hash,
                        sender_account_id: sender_account_id.clone(),
                    },
                    wait_until: TxExecutionStatus::Final,
                })
                .await;

            if sent_at.elapsed() > TIMEOUT {
                return Err("Time limit exceeded for the transaction to be recognized".into());
            }

            match response {
                Ok(response) => {
                    return Ok(response);
                }
                Err(err) => {
                    if matches!(err.handler_error(), Some(RpcTransactionError::TimeoutError))
                        || err.to_string().contains("408 Request Timeout")
                    {
                        continue;
                    }
                    return Err(err.into());
                }
            }
        }
    }

    async fn get_nonce_and_block_hash(
        &self,
        account_id: AccountId,
        public_key: PublicKey,
    ) -> Result<(u64, CryptoHash), Box<dyn std::error::Error>> {
        let access_key_query_response = self
            .client
            .call(RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: QueryRequest::ViewAccessKey {
                    account_id: account_id.clone(),
                    public_key: public_key.clone(),
                },
            })
            .await
            .expect("Failed to call RPC");

        match access_key_query_response.kind {
            QueryResponseKind::AccessKey(access_key) => {
                Ok((access_key.nonce, access_key_query_response.block_hash))
            }
            _ => panic!("Failed to extract current nonce"),
        }
    }
}
