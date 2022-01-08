#![cfg(feature = "test-bpf")]
mod utils;

use mpl_token_metadata::state::{UseMethod, Uses};

use mpl_token_metadata::{
    error::MetadataError,
    id, instruction,
    state::{Key, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH},
    utils::puffed_out_string,
};
use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::{
    instruction::InstructionError,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use utils::*;
mod verify_collection {
    use mpl_token_metadata::state::Collection;

    use super::*;

    #[tokio::test]
    async fn success() {
        let mut context = program_test().start_with_context().await;

        let test_collection = Metadata::new();
        test_collection
            .create_v2(
                &mut context,
                "Test".to_string(),
                "TST".to_string(),
                "uri".to_string(),
                None,
                10,
                false,
                None,
                None,
            )
            .await
            .unwrap();

        let name = "Test".to_string();
        let symbol = "TST".to_string();
        let uri = "uri".to_string();
        let test_metadata = Metadata::new();

        let puffed_name = puffed_out_string(&name, MAX_NAME_LENGTH);
        let puffed_symbol = puffed_out_string(&symbol, MAX_SYMBOL_LENGTH);
        let puffed_uri = puffed_out_string(&uri, MAX_URI_LENGTH);

        let uses = Some(Uses {
            available: 1,
            remaining: 1,
            use_method: UseMethod::Single,
        });
        test_metadata
            .create_v2(
                &mut context,
                name,
                symbol,
                uri,
                None,
                10,
                false,
                Some(Collection {
                    key: test_collection.mint.pubkey(),
                    verified: false,
                }),
                uses.to_owned(),
            )
            .await
            .unwrap();

        let metadata = test_metadata.get_data(&mut context).await;

        assert_eq!(metadata.data.name, puffed_name);
        assert_eq!(metadata.data.symbol, puffed_symbol);
        assert_eq!(metadata.data.uri, puffed_uri);
        assert_eq!(metadata.data.seller_fee_basis_points, 10);
        assert_eq!(metadata.data.creators, None);
        assert_eq!(metadata.uses, uses.to_owned());

        assert_eq!(
            metadata.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert_eq!(metadata.collection.unwrap().verified, false);

        assert_eq!(metadata.primary_sale_happened, false);
        assert_eq!(metadata.is_mutable, false);
        assert_eq!(metadata.mint, test_metadata.mint.pubkey());
        assert_eq!(metadata.update_authority, context.payer.pubkey());
        assert_eq!(metadata.key, Key::MetadataV1);

        let auth = context.payer.pubkey().clone();
        test_metadata
            .verify_collection(
                &mut context,
                test_collection.pubkey,
                auth,
                test_collection.mint.pubkey(),
            )
            .await
            .unwrap();

        let metadata_after = test_metadata.get_data(&mut context).await;
        assert_eq!(
            metadata_after.collection.to_owned().unwrap().key,
            test_collection.mint.pubkey()
        );
        assert_eq!(metadata_after.collection.unwrap().verified, true);
    }
}
