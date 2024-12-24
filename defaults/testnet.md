# testnet

Given:

- A local docker network

When:

- Add a new peer to 4-peers network

Then:

- Peers establish connections each other
- The new peer catches up blocks and reaches the latest state

## How to reproduce

cargo install --path ./crates/iroha_cli --locked
docker network create sharednet
cd defaults

cluster 0:
docker compose -p c0 -f docker-compose.yml up -d
curl http://127.0.0.1:8083/status
curl http://127.0.0.1:8083/peers

cluster 1:
docker compose -p c1 -f docker-compose.single.yml up -d
curl http://127.0.0.1:8079/status
curl http://127.0.0.1:8079/peers

cluster 0:
iroha peer register --key ed0120A36D508BDB1DBEFA4DF361F035A21DFD0F44C4DED67657537718DBF03C60E4C9
curl http://127.0.0.1:8083/status
curl http://127.0.0.1:8083/peers
docker compose -p c0 logs -f

cluster 1:
curl http://127.0.0.1:8079/status
curl http://127.0.0.1:8079/peers
docker compose -p c1 logs -f
