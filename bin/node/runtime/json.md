```
{
    "Miner": {
          "nickname": "Vec<u8>",
          "region": "Vec<u8>",
          "url": "Vec<u8>",
          "capacity": "u64",
          "unit_price": "Balance",
          "violation_times": "u64",
          "total_staking": "Balance"
    },

    "Order": {
        "key": "Vec<u8>",
        "merkle_root": "[u8; 32]",
        "data_length":"u64",
        "user": "AccountId",
        "orders": "Vec<MinerOrder<AccountId, Balance>>",
        "status": "OrderStatus",
        "update_ts": "u64",
        "duration": "u64"
    },

    "MinerOrder": {
        "miner": "AccountId",
        "day_price": "Balance",
        "total_price": "Balance",
        "verify_result": "bool",
        "verify_ts": "u64",
        "confirm_ts": "u64",
        "url": "Option<Vec<u8>>"
    },

    "OrderStatus": {
        "_enum": ["Created", "Confirmed", "Expired", "Deleted"]
    },

    "MiningInfo": {
        "miner": "Option<AccountId>",
        "best_dl": "u64",
        "mining_time": "u64",
        "block": "u64"
    },

    "Difficulty": {
        "base_target": "u64",
        "net_difficulty": "u64",
        "block": "u64"
    }


}
```
