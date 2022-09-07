#!/bin/bash

# USAGE:
# cargo build
# bash scripts/test_env.sh setup
# bash scripts/tests/transfer_assets.sh

set -ex
TEST=${TEST:-"./test"}
CMD="$TEST/iroha_client_cli --config $TEST/config.json"

$CMD domain register --id="looking_glass"
sleep 2
$CMD account register \
    --id="mad_hatter@looking_glass" \
    --key="ed0120a753146e75b910ae5e2994dc8adea9e7d87e5d53024cfa310ce992f17106f92c"
sleep 2
$CMD asset register \
    --id="tea#looking_glass" \
    --value-type=Quantity
sleep 2
$CMD asset mint \
    --account="mad_hatter@looking_glass" \
    --asset="tea#looking_glass" \
    --quantity="100"
sleep 2
$CMD account grant --id "mad_hatter@looking_glass" --permission permission_token.json
sleep 2
$CMD asset transfer --from mad_hatter@looking_glass --to white_rabbit@looking_glass --asset-id tea#looking_glass --quantity 5
sleep 2
$CMD asset get --account="mad_hatter@looking_glass" --asset="tea#looking_glass" | grep -q 'Quantity(95)'
