use near_crypto::{InMemorySigner, PublicKey};
use near_jsonrpc_client::methods;
use near_jsonrpc_client::methods::send_tx::RpcSendTransactionRequest;
use near_jsonrpc_client::methods::tx::{
    RpcTransactionError, RpcTransactionResponse, RpcTransactionStatusRequest, TransactionInfo,
};
use near_jsonrpc_client::{methods::query::RpcQueryRequest, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::action::{Action, DeployContractAction, FunctionCallAction};
use near_primitives::transaction::{Transaction, TransactionV0};
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::TxExecutionStatus;
use near_primitives::{hash::CryptoHash, views::QueryRequest};
use near_sdk::AccountId;
use std::error::Error;
use std::time::{Duration, Instant};

// local modules
pub mod near_network_config;
pub mod parser;

// import local modules
use crate::NearAccount;
use near_network_config::{get_rpc_url, NearNetworkConfig};
use parser::ParseResult;

pub struct FriendlyNearJsonRpcClient {
    client: JsonRpcClient,
    account_config: NearAccount,
    signer: InMemorySigner,
}

const TIMEOUT: Duration = Duration::from_secs(300);

impl FriendlyNearJsonRpcClient {
    pub fn new(network: NearNetworkConfig, account_config: NearAccount) -> Self {
        let account_id = account_config.account_id.clone();
        let private_key = account_config.private_key.clone();
        let signer: InMemorySigner = InMemorySigner::from_secret_key(account_id, private_key);

        Self {
            client: Self::get_near_rpc_client(network),
            account_config,
            signer,
        }
    }

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

        let near_tx: Transaction = Transaction::V0(TransactionV0 {
            signer_id: account_id.clone(),
            public_key: self.signer.public_key(),
            nonce,
            receiver_id: account_id,
            block_hash,
            actions: vec![deploy_action],
        });

        let signer: near_crypto::Signer = self.signer.clone().into();

        // Sign and send the transaction
        let request = RpcSendTransactionRequest {
            signed_transaction: near_tx.sign(&signer),
            wait_until: TxExecutionStatus::Final,
        };

        self.send_transaction_request(request).await
    }

    pub async fn send_transaction_request(
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

    /// Function to call a contract with a generic return type
    pub async fn call_contract<T>(
        &self,
        method_name: &str,
        args: serde_json::Value,
    ) -> Result<T, Box<dyn Error>>
    where
        T: ParseResult,
    {
        let account_id = self.account_config.account_id.clone();

        let request = RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id,
                method_name: method_name.to_string(),
                args: FunctionArgs::from(args.to_string().into_bytes()),
            },
        };

        let response = self.client.call(request).await?;

        // Parse result
        if let QueryResponseKind::CallResult(call_result) = response.kind {
            let result_str = String::from_utf8(call_result.result)?;
            return T::parse(result_str);
        }

        Err("Failed to parse contract call result".into())
    }

    pub async fn call_contract_with_account_id<T>(
        &self,
        account_id: &str,
        method_name: &str,
        args: serde_json::Value,
    ) -> Result<T, Box<dyn Error>>
    where
        T: ParseResult,
    {
        let request = RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: account_id.parse().unwrap(),
                method_name: method_name.to_string(),
                args: FunctionArgs::from(args.to_string().into_bytes()),
            },
        };

        let response = self.client.call(request).await?;

        // Parse result
        if let QueryResponseKind::CallResult(call_result) = response.kind {
            let result_str = String::from_utf8(call_result.result)?;
            return T::parse(result_str);
        }

        Err("Failed to parse contract call result".into())
    }

    pub async fn send_action(
        &self,
        action: FunctionCallAction,
    ) -> Result<RpcTransactionResponse, Box<dyn Error>> {
        let account_id = self.account_config.account_id.clone();

        let (nonce, block_hash) = self
            .get_nonce_and_block_hash(account_id.clone(), self.account_config.public_key.clone())
            .await
            .unwrap();

        let nonce = nonce + 1;

        let signing_action = Action::FunctionCall(Box::new(action));

        let near_tx: Transaction = Transaction::V0(TransactionV0 {
            signer_id: account_id.clone(),
            public_key: self.signer.public_key(),
            nonce,
            receiver_id: account_id.clone(),
            block_hash,
            actions: vec![signing_action],
        });
        let signer: near_crypto::Signer = self.signer.clone().into();

        let signed_transaction = near_tx.sign(&signer);

        // Send the transaction
        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction,
            wait_until: TxExecutionStatus::Final,
        };

        self.send_transaction_request(request).await
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
