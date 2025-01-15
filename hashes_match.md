# Do hashes match between peer and client?

Terminal D:

```bash
docker compose -f docker-compose.single.yml up
```

Terminal Ev (on the container):

```bash
iroha_client_cli -c config/client.json events pipeline
```

Terminal Bl (on the container):

```bash
iroha_client_cli -c config/client.json blocks 1
```

Terminal C (on the container):

```bash
cat config/transaction.json | iroha_client_cli -c config/client.json json
```

## Result

Terminal C (on the container):

```log
"0A4A741EC5C68C9F108E0CB85986BE0844D9608A63D7D33F56F6082FEEFC6157"
```

Terminal D:

```log
iroha0-1  | 2025-01-15T16:20:21.015883Z TRACE iroha::torii::routing::subscription: event=Pipeline(PipelineEvent { entity_kind: Transaction, status: Committed, hash: 0a4a741ec5c68c9f108e0cb85986be0844d9608a63d7d33f56f6082feefc6157 })
```

Terminal Ev (on the container):

```log
{
  "Pipeline": {
    "entity_kind": "Transaction",
    "status": "Committed",
    "hash": "0A4A741EC5C68C9F108E0CB85986BE0844D9608A63D7D33F56F6082FEEFC6157"
  }
}
{
  "Pipeline": {
    "entity_kind": "Block",
    "status": "Committed",
    "hash": "268512A8FD132ED0F80854972FDD09B96A9B81242F4A4AA52EBF67AB916C05DD"
  }
}
```

Terminal Bl (on the container):

```log
{
  "version": "1",
  "content": {
    "signatures": [
      {
        "public_key": "ed01201C61FAF8FE94E253B93114240394F79A607B7FA55F9E5A41EBEC74B88055768B",
        "payload": [
          196,
          40,
          44,
          53,
          149,
          45,
          231,
          124,
          19,
          31,
          131,
          226,
          170,
          237,
          243,
          27,
          251,
          28,
          185,
          236,
          192,
          37,
          89,
          225,
          142,
          35,
          209,
          94,
          166,
          136,
          171,
          253,
          165,
          233,
          90,
          151,
          45,
          174,
          71,
          94,
          75,
          227,
          15,
          121,
          199,
          163,
          240,
          14,
          107,
          171,
          227,
          166,
          241,
          191,
          180,
          57,
          182,
          157,
          64,
          65,
          108,
          155,
          157,
          15
        ]
      }
    ],
    "payload": {
      "header": {
        "height": 4,
        "timestamp_ms": 1736958021015,
        "previous_block_hash": "13411D8E8EE1A7DCBFEDB7459DD8154FA4ABD851A849ED6C34BECF812960A297",
        "transactions_hash": "3CCCD344B146509828197E9FECA4A073C716FD3C6927ABEB5A8212A96DCF8B35",
        "view_change_index": 1,
        "consensus_estimation_ms": 4000
      },
      "commit_topology": [
        {
          "address": "iroha0:1337",
          "public_key": "ed01201C61FAF8FE94E253B93114240394F79A607B7FA55F9E5A41EBEC74B88055768B"
        }
      ],
      "transactions": [
        {
          "value": {
            "version": "1",
            "content": {
              "signatures": [
                {
                  "public_key": "ed01207233BFC89DCBD68C19FDE6CE6158225298EC1131B6A130D1AEB454C1AB5183C0",
                  "payload": [
                    34,
                    32,
                    197,
                    110,
                    197,
                    2,
                    21,
                    60,
                    72,
                    209,
                    23,
                    39,
                    175,
                    76,
                    37,
                    237,
                    86,
                    115,
                    21,
                    65,
                    47,
                    79,
                    92,
                    212,
                    131,
                    169,
                    23,
                    186,
                    62,
                    128,
                    20,
                    127,
                    186,
                    169,
                    136,
                    175,
                    179,
                    238,
                    223,
                    27,
                    41,
                    156,
                    191,
                    186,
                    208,
                    130,
                    252,
                    134,
                    116,
                    51,
                    98,
                    249,
                    3,
                    0,
                    170,
                    139,
                    1,
                    214,
                    24,
                    203,
                    99,
                    109,
                    195,
                    13
                  ]
                }
              ],
              "payload": {
                "creation_time_ms": 1736958020910,
                "authority": "alice@wonderland",
                "instructions": {
                  "Instructions": [
                    {
                      "Log": {
                        "LogLevel": "ERROR",
                        "msg": {
                          "String": "inspection marker"
                        }
                      }
                    }
                  ]
                },
                "time_to_live_ms": 100000,
                "nonce": null,
                "metadata": {}
              }
            }
          },
          "error": null
        }
      ],
      "event_recommendations": []
    }
  }
}
```
