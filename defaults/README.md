# BUG: Re-registered peer not reflected until restart

When a peer is registered, unregistered, and registered again to a network, the peer has to restart for the network to update.

- Observed not only in local docker network but also in testnet
- Not observed when unregister an existing peer in swarm and register it again

## How to reproduce

```bash
git remote add sato https://github.com/s8sato/iroha.git
git fetch sato test/re_register_peer
git checkout FETCH_HEAD
cargo install --path ./crates/iroha_cli --locked
docker network create sharednet
cd defaults
```

cluster 0:

```bash
docker compose -p c0 -f docker-compose.yml up -d
curl http://127.0.0.1:8083/status
```

```json
{"peers":3,"blocks":1,"txs_approved":4,"txs_rejected":0,"uptime":{"secs":5,"nanos":230000000},"view_changes":0,"queue_size":0}
```

cluster 1:

```bash
docker compose -p c1 -f docker-compose.single.yml up -d
curl http://127.0.0.1:8079/status
```

```json
{"peers":0,"blocks":0,"txs_approved":0,"txs_rejected":0,"uptime":{"secs":0,"nanos":0},"view_changes":0,"queue_size":0}
```

cluster 0:

```bash
iroha peer register --key ed0120A36D508BDB1DBEFA4DF361F035A21DFD0F44C4DED67657537718DBF03C60E4C9
curl http://127.0.0.1:8083/status
```

```json
{"peers":4,"blocks":2,"txs_approved":5,"txs_rejected":0,"uptime":{"secs":65,"nanos":915000000},"view_changes":0,"queue_size":0}
```

cluster 1:

```bash
curl http://127.0.0.1:8079/status
```

```json
{"peers":4,"blocks":2,"txs_approved":5,"txs_rejected":0,"uptime":{"secs":73,"nanos":642000000},"view_changes":0,"queue_size":0}
```

cluster 0:

```bash
iroha peer unregister --key ed0120A36D508BDB1DBEFA4DF361F035A21DFD0F44C4DED67657537718DBF03C60E4C9
curl http://127.0.0.1:8083/status
```

```json
{"peers":3,"blocks":3,"txs_approved":6,"txs_rejected":0,"uptime":{"secs":197,"nanos":850000000},"view_changes":0,"queue_size":0}
```

cluster 1:

```bash
curl http://127.0.0.1:8079/status
```

```json
{"peers":0,"blocks":3,"txs_approved":6,"txs_rejected":0,"uptime":{"secs":231,"nanos":785000000},"view_changes":0,"queue_size":0}
```

cluster 0:

```bash
iroha peer register --key ed0120A36D508BDB1DBEFA4DF361F035A21DFD0F44C4DED67657537718DBF03C60E4C9
curl http://127.0.0.1:8083/status
```

```json
{"peers":3,"blocks":4,"txs_approved":7,"txs_rejected":0,"uptime":{"secs":634,"nanos":168000000},"view_changes":0,"queue_size":0}
```

EXPECTED: `"peers":4`

cluster 1:

```bash
curl http://127.0.0.1:8079/status
```

```json
{"peers":0,"blocks":3,"txs_approved":6,"txs_rejected":0,"uptime":{"secs":1866,"nanos":82000000},"view_changes":0,"queue_size":0}
```

EXPECTED: `"peers":4,"blocks":4`

cluster 1:

```bash
docker compose -p c1 restart
curl http://127.0.0.1:8079/status
```

```json
{"peers":4,"blocks":4,"txs_approved":7,"txs_rejected":0,"uptime":{"secs":2554,"nanos":670000000},"view_changes":0,"queue_size":0}
```

cluster 0:

```bash
curl http://127.0.0.1:8083/status
```

```json
{"peers":4,"blocks":4,"txs_approved":7,"txs_rejected":0,"uptime":{"secs":2606,"nanos":132000000},"view_changes":0,"queue_size":0}
```
