// use bitcoind::Client as BitcoinClient;
use near_jsonrpc_client::JsonRpcClient;

pub struct OmniJsonRpc {
    near_client: JsonRpcClient,
    // ethereum_client: FillProvider,
    // bitcoin_client: BitcoinClient,
}

impl OmniJsonRpc {
    pub async fn new() -> Self {
        // let rpc_address = sandbox_worker.rpc_addr();
        // let rpc_client = JsonRpcClient::connect(rpc_address);
        let near_client = JsonRpcClient::connect("https://rpc.testnet.near.org");

        // let bitcoin_client = BitcoinClient::new("http://localhost:18443".to_string());

        Self {
            near_client,
            // bitcoin_client,
        }
    }
}
