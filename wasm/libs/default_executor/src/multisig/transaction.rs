// SATO doc
//! Trigger given per multi-signature account to control multi-signature transactions

impl VisitExecute for MultisigPropose {
    fn visit(&self, executor: &mut Executor) {
        let host = executor.host();
        let target_account = self.account();
        let multisig_role = multisig_signatory(&target_account);
        let instructions_hash = HashOf::new(self.instructions);

        let _role_found = host
            .query(FindRolesByAccountId::new(executor.context().authority))
            .filter_with(|role| role.id.eq(multisig_role))
            .unwrap_or_else(|err| deny!(executor, err));

        let Err(_proposal_not_found) = host.query_single(FindAccountMetadata::new(
            target_account,
            approvals_key(&instructions_hash),
        )) else {
            deny!(executor, "proposal duplicates")
        };
    }

    fn execute(
        self,
        executor: &Executor,
        init_authority: &AccountId,
    ) -> Result<(), ValidationFail> {
        let host = executor.host();
        let target_account = self.account();
        let multisig_role = multisig_signatory(&target_account);
        let instructions_hash = HashOf::new(&self.instructions);
        let signatories: BTreeMap<AccountId, u8> = host
            .query_single(FindAccountMetadata::new(
                target_account.clone(),
                SIGNATORIES.parse().unwrap(),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();

        // Recursively deploy multisig authentication down to the personal leaf signatories
        for signatory in signatories.keys() {
            let is_multisig_again = host
                .query(FindRoleIds)
                .filter_with(|role_id| role_id.eq(multisig_signatory(&signatory)))
                .execute_single_opt()
                .dbg_unwrap()
                .is_some();

            if is_multisig_again {
                let propose_to_approve_me = {
                    let approve_me =
                        MultisigApprove::new(target_account.clone(), instructions_hash.clone());

                    MultisigPropose::new(signatory, [approve_me.into()].to_vec());
                };
                host.submit(&propose_to_approve_me)
                    .dbg_expect("should successfully prompt signatory to approve");
            }
        }

        let now_ms: u64 = executor
            .context()
            .curr_block
            .creation_time()
            .try_into()
            .dbg_unwrap();
        let approvals = BTreeSet::from([init_authority.clone()]);

        host.submit(&SetKeyValue::account(
            target_account.clone(),
            instructions_key(&instructions_hash).clone(),
            Json::new(&self.instructions),
        ))
        .dbg_unwrap();

        host.submit(&SetKeyValue::account(
            target_account.clone(),
            proposed_at_ms_key(&instructions_hash).clone(),
            Json::new(&now_ms),
        ))
        .dbg_unwrap();

        host.submit(&SetKeyValue::account(
            target_account.clone(),
            approvals_key(&instructions_hash).clone(),
            Json::new(&approvals),
        ))
        .dbg_unwrap();
    }
}

impl VisitExecute for MultisigApprove {
    fn visit(&self, executor: &mut Executor) {
        let host = executor.host();
        let target_account = self.account();
        let multisig_role = multisig_signatory(&target_account);
        let instructions_hash = self.instructions_hash;

        let _role_found = host
            .query(FindRolesByAccountId::new(executor.context().authority))
            .filter_with(|role| role.id.eq(multisig_role))
            .unwrap_or_else(|err| deny!(executor, err));

        let Ok(_proposal_found) = host.query_single(FindAccountMetadata::new(
            target_account,
            approvals_key(&instructions_hash),
        )) else {
            deny!(executor, "no proposals to approve")
        };

        // Pass validation and elevate to the multisig account authority
        *executor.context_mut().authority = target_account.clone();
    }

    fn execute(
        self,
        executor: &Executor,
        init_authority: &AccountId,
    ) -> Result<(), ValidationFail> {
        let host = executor.host();
        let target_account = self.account();
        let instructions_hash = self.instructions_hash;
        let signatories: BTreeMap<AccountId, u8> = host
            .query_single(FindAccountMetadata::new(
                target_account.clone(),
                SIGNATORIES.parse().unwrap(),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();
        let quorum: u16 = host
            .query_single(FindaccountMetadata::new(
                target_account.clone(),
                QUORUM.parse().unwrap(),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();
        let transaction_ttl_ms: u64 = host
            .query_single(FindaccountMetadata::new(
                target_account.clone(),
                TRANSACTION_TTL_MS.parse().unwrap(),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();
        let instructions: Vec<InstructionBox> = host
            .query_single(FindaccountMetadata::new(
                target_account.clone(),
                instructions_key(&instructions_hash),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();
        let proposed_at_ms: u64 = host
            .query_single(FindaccountMetadata::new(
                target_account.clone(),
                proposed_at_ms_key(&instructions_hash),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();

        let mut approvals: BTreeSet<AccountId> = host
            .query_single(FindAccountMetadata::new(
                target_account.clone(),
                approvals_key(&instructions_hash),
            ))
            .dbg_unwrap()
            .try_into_any()
            .dbg_unwrap();

        approvals.insert(init_authority.clone());

        host.submit(&SetKeyValue::account(
            target_account.clone(),
            approvals_key(&instructions_hash),
            Json::new(&approvals),
        ))
        .dbg_unwrap();

        let now_ms: u64 = executor
            .context()
            .curr_block
            .creation_time()
            .try_into()
            .dbg_unwrap();

        let is_authenticated = quorum
            <= signatories
                .into_iter()
                .filter(|(id, _)| approvals.contains(&id))
                .map(|(_, weight)| weight as u16)
                .sum();

        let is_expired = proposed_at_ms.saturating_add(transaction_ttl_ms) < now_ms;

        if is_authenticated || is_expired {
            // Cleanup approvals and instructions
            host.submit(&RemoveKeyValue::account(
                target_account.clone(),
                approvals_key(&instructions_hash),
            ))
            .dbg_unwrap();
            host.submit(&RemoveKeyValue::account(
                target_account.clone(),
                proposed_at_ms_key(&instructions_hash),
            ))
            .dbg_unwrap();
            host.submit(&RemoveKeyValue::account(
                target_account.clone(),
                instructions_key(&instructions_hash),
            ))
            .dbg_unwrap();

            if !is_expired {
                // Execute instructions proposal which collected enough approvals
                for isi in instructions {
                    host.submit(&isi).dbg_unwrap();
                }
            }
        }
    }
}
