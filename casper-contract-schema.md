# Casper Contract Schema

For every contract we need to specify:
- Casper Contract Schema Version 
- Metadata:
    - Toolchain: Compiler, Version
- Data:
    - Entry points
    - Events
    - Types
    - Named Keys

## Types serialization
- [?] Include type name in front of the type. 
    - Advantage: disunbiguity,
    - Disadvantage: larger payload -> larger gas cost.

## Future work
- IDE intergration
- Codegen for JS, Rust, C#

## Tasks
- [types] Specify the way to define CLTypes in JSON.
- [types] Specify structs and enums serialization.

## Example schema
```json
{
  "name": "Erc20",
  "entrypoints": [
    {
      "name": "init",
      "is_mutable": true,
      "args": [
        {
          "name": "name",
          "ty": "String"
        },
        {
          "name": "symbol",
          "ty": "String"
        },
        {
          "name": "decimals",
          "ty": "U8"
        },
        {
          "name": "initial_supply",
          "ty": "U256"
        }
      ],
      "return_ty": "Unit"
    },
    {
      "name": "transfer",
      "is_mutable": true,
      "args": [
        {
          "name": "recipient",
          "ty": "Key"
        },
        {
          "name": "amount",
          "ty": "U256"
        }
      ],
      "return_ty": "Unit"
    },
    {
      "name": "transfer_from",
      "is_mutable": true,
      "args": [
        {
          "name": "owner",
          "ty": "Key"
        },
        {
          "name": "recipient",
          "ty": "Key"
        },
        {
          "name": "amount",
          "ty": "U256"
        }
      ],
      "return_ty": "Unit"
    },
    {
      "name": "approve",
      "is_mutable": true,
      "args": [
        {
          "name": "spender",
          "ty": "Key"
        },
        {
          "name": "amount",
          "ty": "U256"
        }
      ],
      "return_ty": "Unit"
    },
    {
      "name": "name",
      "is_mutable": false,
      "args": [],
      "return_ty": "String"
    },
    {
      "name": "symbol",
      "is_mutable": false,
      "args": [],
      "return_ty": "String"
    },
    {
      "name": "decimals",
      "is_mutable": false,
      "args": [],
      "return_ty": "U8"
    },
    {
      "name": "total_supply",
      "is_mutable": false,
      "args": [],
      "return_ty": "U256"
    },
    {
      "name": "balance_of",
      "is_mutable": false,
      "args": [
        {
          "name": "address",
          "ty": "Key"
        }
      ],
      "return_ty": "U256"
    },
    {
      "name": "allowance",
      "is_mutable": false,
      "args": [
        {
          "name": "owner",
          "ty": "Key"
        },
        {
          "name": "spender",
          "ty": "Key"
        }
      ],
      "return_ty": "U256"
    }
  ],
  "events": [
    { "name": "OnTransfer", "ty": "Transfer" },
    { "name": "OnBurn", "ty": "Transfer" }
  ],
  "types": [{
    "name": "Transfer",
    "fields": [
      {
        "name": "from",
        "ty": {
          "Option": "Key"
        }
      },
      {
        "name": "to",
        "ty": {
          "Option": "Key"
        }
      },
      {
        "name": "amount",
        "ty": "U256"
      }
    ]
  },
  {
    "name": "Transfers",
    "fields": [
      {
        "name": "transfers",
        "ty": {
          "Vec": "Transfer"
        }
      }
    ]
  }
  ],
  "named_keys": [
    {
      "name": "STATE",
      "type": "Dictionary"
    },
    {
      "name": "TOTAL_SUPPLY",
      "type": "U256"
    }
  ]
}

```