use alloy::{
    network::{Ethereum, EthereumWallet},
    node_bindings::AnvilInstance,
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, ProviderBuilder, RootProvider,
    },
    signers::local::PrivateKeySigner,
    transports::http::{Client, Http},
};

use crate::clients::get_anvil_instance;

type Provider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

type DefaultProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

#[derive(Debug)]
pub struct EVMTestContext {
    pub anvil: AnvilInstance,
    pub provider: DefaultProvider,
    pub alice: EthereumWallet,
    pub bob: EthereumWallet,
}

impl Default for EVMTestContext {
    fn default() -> Self {
        let anvil = get_anvil_instance().unwrap();
        Self::new(anvil)
    }
}

impl EVMTestContext {
    pub fn new(anvil: AnvilInstance) -> Self {
        // Configure the signers for the first two Anvil accounts (Alice and Bob).
        let alice_signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let bob_signer: PrivateKeySigner = anvil.keys()[1].clone().into();
        let alice = EthereumWallet::from(alice_signer);
        let bob = EthereumWallet::from(bob_signer);

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(anvil.endpoint_url());

        Self {
            anvil,
            alice,
            bob,
            provider,
        }
    }

    pub const fn alice(&self) -> &EthereumWallet {
        &self.alice
    }

    pub const fn bob(&self) -> &EthereumWallet {
        &self.bob
    }

    pub fn get_provider(&self, wallet: EthereumWallet) -> Provider {
        // Create a provider with the wallet.
        let rpc_url = self.anvil.endpoint().parse().unwrap();
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url)
    }
}
