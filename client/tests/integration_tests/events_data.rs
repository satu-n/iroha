#![allow(clippy::restriction)]

use std::{sync::mpsc, thread};

use eyre::Result;
use iroha_core::config::Configuration;
use iroha_data_model::prelude::*;
use test_network::{Peer as TestPeer, *};

#[test]
fn data_events() -> Result<()> {
    let (_rt, _peer, mut client) = <TestPeer>::start_test_with_runtime();
    wait_for_genesis_committed(vec![client.clone()], 0);
    let pipeline_time = Configuration::pipeline_time();

    // spawn event reporter
    let mut listener = client.clone();
    let (init_sender, init_receiver) = mpsc::channel();
    let (event_sender, event_receiver) = mpsc::channel();
    let event_filter = DataEventFilter::default().into();
    let _handle = thread::spawn(move || {
        let event_iterator = listener.listen_for_events(event_filter).unwrap();
        init_sender.send(()).unwrap();
        for event in event_iterator.flatten() {
            iroha_logger::error!(?event, "listen!");
            event_sender.send(event).unwrap()
        }
    });

    // submit instructions to produce events
    let domains: Vec<Domain> = (0..3)
        .map(|domain_index| Domain::test(&domain_index.to_string()))
        .collect();
    let registers: [Instruction; 3] = domains
        .iter()
        .map(|domain| RegisterBox::new(IdentifiableBox::from(domain.clone())).into())
        .collect::<Vec<Instruction>>()
        .try_into()
        .unwrap();
    let fail = FailBox::new("fail");
    let instructions = vec![
        // domain "0"
        // pair
        //      domain "1"
        //      if false fail else sequence
        //          fail
        //          domain "2"
        registers[0].clone(),
        Pair::new::<Instruction, _>(
            registers[1].clone(),
            IfInstruction::with_otherwise(
                false,
                fail.clone(),
                SequenceBox::new(vec![fail.into(), registers[2].clone()]),
            ),
        )
        .into(),
    ];
    init_receiver.recv()?;
    client.submit_all(instructions)?;
    thread::sleep(pipeline_time * 2);

    // assertion
    for expected_event in domains
        .into_iter()
        .map(Register::new)
        .map(DataEvent::from)
        .map(Event::from)
    {
        let actual_event = event_receiver.recv()?;
        assert_eq!(actual_event, expected_event);
    }

    Ok(())
}
