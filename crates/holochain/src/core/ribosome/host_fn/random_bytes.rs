use crate::core::ribosome::error::RibosomeResult;
use crate::core::ribosome::CallContext;
use crate::core::ribosome::RibosomeT;
use holochain_crypto::crypto_init_sodium;
use holochain_crypto::crypto_randombytes_buf;
use holochain_crypto::crypto_secure_buffer;
use holochain_crypto::DynCryptoBytes;
use holochain_zome_types::bytes::Bytes;
use holochain_zome_types::RandomBytesInput;
use holochain_zome_types::RandomBytesOutput;
use std::sync::Arc;

/// return n crypto secure random bytes from the standard holochain crypto lib
pub fn random_bytes(
    _ribosome: Arc<impl RibosomeT>,
    _call_context: Arc<CallContext>,
    input: RandomBytesInput,
) -> RibosomeResult<RandomBytesOutput> {
    let _ = crypto_init_sodium();
    let mut buf: DynCryptoBytes = crypto_secure_buffer(input.into_inner() as _)?;

    tokio_safe_block_on::tokio_safe_block_forever_on(async {
        crypto_randombytes_buf(&mut buf).await
    })?;

    let random_bytes = buf.read();

    Ok(RandomBytesOutput::new(Bytes::from(random_bytes.to_vec())))
}

#[cfg(test)]
#[cfg(feature = "slow_tests")]
pub mod wasm_test {
    use crate::core::ribosome::host_fn::random_bytes::random_bytes;

    use crate::fixt::CallContextFixturator;
    use crate::fixt::WasmRibosomeFixturator;
    use crate::fixt::ZomeCallHostAccessFixturator;
    use ::fixt::prelude::*;
    use holochain_wasm_test_utils::TestWasm;
    use holochain_zome_types::RandomBytesInput;
    use holochain_zome_types::RandomBytesOutput;
    use std::convert::TryInto;
    use std::sync::Arc;

    #[tokio::test(threaded_scheduler)]
    /// we can get some random data out of the fn directly
    async fn random_bytes_test() {
        let ribosome = WasmRibosomeFixturator::new(crate::fixt::curve::Zomes(vec![]))
            .next()
            .unwrap();
        let call_context = CallContextFixturator::new(fixt::Unpredictable)
            .next()
            .unwrap();
        const LEN: usize = 10;
        let input = RandomBytesInput::new(LEN.try_into().unwrap());

        let output: RandomBytesOutput =
            random_bytes(Arc::new(ribosome), Arc::new(call_context), input).unwrap();

        println!("{:?}", output);

        assert_ne!(&[0; LEN], output.into_inner().as_ref(),);
    }

    #[tokio::test(threaded_scheduler)]
    /// we can get some random data out of the fn via. a wasm call
    async fn ribosome_random_bytes_test() {
        let test_env = holochain_state::test_utils::test_cell_env();
        let env = test_env.env();
        let mut workspace =
            crate::core::workflow::CallZomeWorkspace::new(env.clone().into()).unwrap();
        crate::core::workflow::fake_genesis(&mut workspace.source_chain)
            .await
            .unwrap();
        let workspace_lock = crate::core::workflow::CallZomeWorkspaceLock::new(workspace);

        const LEN: usize = 5;
        let mut host_access = fixt!(ZomeCallHostAccess);
        host_access.workspace = workspace_lock;
        let output: RandomBytesOutput = crate::call_test_ribosome!(
            host_access,
            TestWasm::RandomBytes,
            "random_bytes",
            RandomBytesInput::new(5 as _)
        );
        assert_ne!(&[0; LEN], output.into_inner().as_ref(),);
    }
}
