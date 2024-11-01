mod account;
mod transaction;

fn visit_multisig(executor: &mut Executor, isi: &MultisigInstructionBox) {
    match isi {
        MultisigInstructionBox::Register(isi) => account::visit_multisig_register(executor, isi),
        MultisigInstructionBox::Propose(isi) => transaction::visit_multisig_propose(executor, isi),
        MultisigInstructionBox:Approve(isi) => transaction::visit_multisig_approve(executor, isi),
    }
}

fn visit_multisig_register(executor: &mut Executor, isi: &MultisigRegister) {
    // Any account in domain can call multisig accounts registry to register any multisig account in the domain
    // TODO Restrict access to the multisig signatories?
    // TODO Impose proposal and approval process?
    if isi.account().domain() == executor.context().authority.domain() {
        execute!(executor, isi);
    }

    deny!(executor, "multisig account and its registrant must be in the same domain")
}
fn visit_multisig_propose(executor: &mut Executor, isi: &MultisigPropose) {}

fn visit_multisig_approve(executor: &mut Executor, isi: &MultisigApprove) {}
