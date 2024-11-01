// SATO doc
//! Trigger given per domain to control multi-signature accounts and corresponding triggers

fn visit_multisig_register(executor: &mut Executor, isi: &MultisigRegister) {
    // Any account in domain can call multisig accounts registry to register any multisig account in the domain
    // TODO Restrict access to the multisig signatories?
    // TODO Impose proposal and approval process?
    if isi.account().domain() == executor.context().authority.domain() {
        execute!(executor, isi);
    }

    deny!(executor, "multisig account and its registrant must be in the same domain")
}

#[iroha_trigger::main]
fn main(host: Iroha, context: Context) {
    let EventBox::ExecuteTrigger(event) = context.event else {
        dbg_panic("trigger misused: must be triggered only by a call");
    };
    let args: MultisigAccountArgs = event
        .args()
        .try_into_any()
        .dbg_expect("args should be for a multisig account");
    let domain_id = context
        .id
        .name()
        .as_ref()
        .strip_prefix("multisig_accounts_")
        .and_then(|s| s.parse::<DomainId>().ok())
        .dbg_unwrap();
    let account_id = AccountId::new(domain_id, args.account);

    host.submit(&Register::account(Account::new(account_id.clone())))
        .dbg_expect("accounts registry should successfully register a multisig account");

    let multisig_transactions_registry_id: TriggerId = format!(
        "multisig_transactions_{}_{}",
        account_id.signatory(),
        account_id.domain()
    )
    .parse()
    .dbg_unwrap();

    let multisig_transactions_registry = Trigger::new(
        multisig_transactions_registry_id.clone(),
        Action::new(
            WasmSmartContract::from_compiled(MULTISIG_TRANSACTIONS_WASM.to_vec()),
            Repeats::Indefinitely,
            account_id.clone(),
            ExecuteTriggerEventFilter::new().for_trigger(multisig_transactions_registry_id.clone()),
        ),
    );

    host.submit(&Register::trigger(multisig_transactions_registry))
        .dbg_expect("accounts registry should successfully register a transactions registry");

    host.submit(&SetKeyValue::trigger(
        multisig_transactions_registry_id.clone(),
        "signatories".parse().unwrap(),
        Json::new(&args.signatories),
    ))
    .dbg_unwrap();

    host.submit(&SetKeyValue::trigger(
        multisig_transactions_registry_id.clone(),
        "quorum".parse().unwrap(),
        Json::new(&args.quorum),
    ))
    .dbg_unwrap();

    host.submit(&SetKeyValue::trigger(
        multisig_transactions_registry_id.clone(),
        "transaction_ttl_ms".parse().unwrap(),
        Json::new(&args.transaction_ttl_ms),
    ))
    .dbg_unwrap();

    let role_id: RoleId = format!(
        "multisig_signatory_{}_{}",
        account_id.signatory(),
        account_id.domain()
    )
    .parse()
    .dbg_unwrap();

    host.submit(&Register::role(
        // Temporarily grant a multisig role to the trigger authority to delegate the role to the signatories
        Role::new(role_id.clone(), context.authority.clone()),
    ))
    .dbg_expect("accounts registry should successfully register a multisig role");

    for signatory in args.signatories.keys().cloned() {
        let is_multisig_again = {
            let sub_role_id: RoleId = format!(
                "multisig_signatory_{}_{}",
                signatory.signatory(),
                signatory.domain()
            )
            .parse()
            .dbg_unwrap();

            host.query(FindRoleIds)
                .filter_with(|role_id| role_id.eq(sub_role_id))
                .execute_single_opt()
                .dbg_unwrap()
                .is_some()
        };

        if is_multisig_again {
            // Allow the transactions registry to write to the sub registry
            let sub_registry_id: TriggerId = format!(
                "multisig_transactions_{}_{}",
                signatory.signatory(),
                signatory.domain()
            )
            .parse()
            .dbg_unwrap();

            host.submit(&Grant::account_permission(
                CanExecuteTrigger {
                    trigger: sub_registry_id,
                },
                account_id.clone(),
            ))
            .dbg_expect(
                "accounts registry should successfully grant permission to the multisig account",
            );
        }

        host.submit(&Grant::account_role(role_id.clone(), signatory))
            .dbg_expect(
                "accounts registry should successfully grant the multisig role to signatories",
            );
    }

    host.submit(&Revoke::account_role(role_id.clone(), context.authority))
        .dbg_expect(
        "accounts registry should successfully revoke the multisig role from the trigger authority",
    );
}
