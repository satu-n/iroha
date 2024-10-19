#![no_std]

#[cfg(not(test))]
extern crate panic_halt;
extern crate alloc;

use alloc::format;
use dlmalloc::GlobalDlmalloc;
use iroha_smart_contract::{prelude::*, Iroha, data_model::Level};

#[global_allocator]
static ALLOC: GlobalDlmalloc = GlobalDlmalloc;

getrandom::register_custom_getrandom!(iroha_smart_contract::stub_getrandom);

#[iroha_smart_contract::main]
fn main(host: Iroha, context: Context) {
    let headers = host
        .query(FindBlockHeaders)
        .execute_all()
        .dbg_unwrap();

    host.submit(
        &Log::new(Level::ERROR, format!("wasm context: {:#?}", context))
    ).dbg_unwrap();

    host.submit(
        &Log::new(Level::ERROR, format!("found block headers: {:#?}", headers))
    ).dbg_unwrap();
}
