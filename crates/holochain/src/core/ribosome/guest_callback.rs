pub mod entry_defs;
pub mod init;
pub mod migrate_agent;
pub mod post_commit;
pub mod validate;
pub mod validate_link_add;
pub mod validation_package;
use super::HostAccess;
use crate::core::ribosome::error::RibosomeError;
use crate::core::ribosome::FnComponents;
use crate::core::ribosome::Invocation;
use crate::core::ribosome::RibosomeT;
use fallible_iterator::FallibleIterator;
use holochain_zome_types::zome::ZomeName;
use holochain_zome_types::ExternOutput;

pub struct CallIterator<R: RibosomeT, I: Invocation> {
    host_access: HostAccess,
    ribosome: R,
    invocation: I,
    remaining_zomes: Vec<ZomeName>,
    remaining_components: FnComponents,
}

impl<R: RibosomeT, I: Invocation> CallIterator<R, I> {
    pub fn new(host_access: HostAccess, ribosome: R, invocation: I) -> Self {
        Self {
            host_access,
            remaining_zomes: ribosome.zomes_to_invoke(invocation.zomes()),
            ribosome,
            remaining_components: invocation.fn_components(),
            invocation,
        }
    }
}

impl<R: RibosomeT, I: Invocation + 'static> FallibleIterator for CallIterator<R, I> {
    type Item = (ZomeName, ExternOutput);
    type Error = RibosomeError;
    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        Ok(match self.remaining_zomes.first() {
            Some(zome_name) => {
                match self.remaining_components.next() {
                    Some(to_call) => {
                        match self.ribosome.maybe_call(
                            self.host_access.clone(),
                            &self.invocation,
                            zome_name,
                            &to_call.into(),
                        )? {
                            Some(result) => Some((zome_name.clone(), result)),
                            None => self.next()?,
                        }
                    }
                    // there are no more callbacks to call in this zome
                    // reset fn components and move to the next zome
                    None => {
                        self.remaining_components = self.invocation.fn_components();
                        self.remaining_zomes.remove(0);
                        self.next()?
                    }
                }
            }
            None => None,
        })
    }
}

#[cfg(test)]
#[cfg(feature = "slow_tests")]
mod tests {

    use super::CallIterator;
    use crate::core::ribosome::FnComponents;
    use crate::core::ribosome::MockInvocation;
    use crate::core::ribosome::MockRibosomeT;
    use crate::core::ribosome::ZomesToInvoke;
    use crate::fixt::FnComponentsFixturator;
    use crate::fixt::ZomeCallHostAccessFixturator;
    use crate::fixt::ZomeNameFixturator;
    use fallible_iterator::FallibleIterator;
    use holochain_zome_types::init::InitCallbackResult;
    use holochain_zome_types::zome::FunctionName;
    use holochain_zome_types::zome::ZomeName;
    use holochain_zome_types::ExternOutput;
    use mockall::predicate::*;
    use mockall::Sequence;
    use std::convert::TryInto;

    #[tokio::test(threaded_scheduler)]
    async fn call_iterator_iterates() {
        // stuff we need to test with
        let mut sequence = Sequence::new();
        let mut ribosome = MockRibosomeT::new();

        let mut invocation = MockInvocation::new();

        let host_access = ZomeCallHostAccessFixturator::new(fixt::Empty)
            .next()
            .unwrap();
        let zome_name_fixturator = ZomeNameFixturator::new(fixt::Unpredictable);
        let mut fn_components_fixturator = FnComponentsFixturator::new(fixt::Unpredictable);

        // let returning_init_invocation = init_invocation.clone();
        let zome_names: Vec<ZomeName> = zome_name_fixturator.take(3).collect();
        let fn_components: FnComponents = fn_components_fixturator.next().unwrap();

        invocation
            .expect_zomes()
            .times(1)
            .in_sequence(&mut sequence)
            .return_const(ZomesToInvoke::All);

        ribosome
            // this should happen inside the CallIterator constructor
            .expect_zomes_to_invoke()
            .times(1)
            .in_sequence(&mut sequence)
            .return_const(zome_names.clone());

        invocation
            .expect_fn_components()
            .times(1)
            .in_sequence(&mut sequence)
            .return_const(fn_components.clone());

        // zomes are the outer loop as we process all callbacks in a single zome before moving to
        // the next one
        for zome_name in zome_names.clone() {
            for fn_component in fn_components.clone() {
                // the invocation zome name and component will be called by the ribosome
                ribosome
                    .expect_maybe_call::<MockInvocation>()
                    .with(
                        always(),
                        always(),
                        eq(zome_name.clone()),
                        eq(FunctionName::from(fn_component)),
                    )
                    .times(1)
                    .in_sequence(&mut sequence)
                    .returning(|_, _, _, _| {
                        Ok(Some(ExternOutput::new(
                            InitCallbackResult::Pass.try_into().unwrap(),
                        )))
                    });
            }

            // the fn components are reset from the invocation every zome
            invocation
                .expect_fn_components()
                .times(1)
                .in_sequence(&mut sequence)
                .return_const(fn_components.clone());
        }

        let call_iterator = CallIterator::new(host_access.into(), ribosome, invocation);

        let output: Vec<(_, ExternOutput)> = call_iterator.collect().unwrap();
        assert_eq!(output.len(), zome_names.len() * fn_components.0.len());
    }
}
