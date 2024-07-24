export type LmaofunBondingCurve = {
  "version": "0.1.0",
  "name": "lmaofun_bonding_curve",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "global",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "GlobalSettingsInput"
          }
        }
      ]
    },
    {
      "name": "setParams",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "global",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "newAuthority",
          "isMut": false,
          "isSigner": false,
          "isOptional": true
        },
        {
          "name": "newFeeRecipient",
          "isMut": false,
          "isSigner": false,
          "isOptional": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "GlobalSettingsInput"
          }
        }
      ]
    },
    {
      "name": "createBondingCurve",
      "accounts": [
        {
          "name": "mint",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondingCurveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "global",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadata",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMetadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "CreateBondingCurveParams"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "bondingCurve",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "virtualSolReserves",
            "type": "u64"
          },
          {
            "name": "virtualTokenReserves",
            "type": "u64"
          },
          {
            "name": "realSolReserves",
            "type": "u64"
          },
          {
            "name": "realTokenReserves",
            "type": "u64"
          },
          {
            "name": "tokenTotalSupply",
            "type": "u64"
          },
          {
            "name": "complete",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "global",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "status",
            "type": {
              "defined": "ProgramStatus"
            }
          },
          {
            "name": "initialized",
            "type": "bool"
          },
          {
            "name": "globalAuthority",
            "type": "publicKey"
          },
          {
            "name": "feeRecipient",
            "type": "publicKey"
          },
          {
            "name": "initialVirtualTokenReserves",
            "type": "u64"
          },
          {
            "name": "initialVirtualSolReserves",
            "type": "u64"
          },
          {
            "name": "initialRealTokenReserves",
            "type": "u64"
          },
          {
            "name": "initialRealSolReserves",
            "type": "u64"
          },
          {
            "name": "initialTokenSupply",
            "type": "u64"
          },
          {
            "name": "solLaunchThreshold",
            "type": "u64"
          },
          {
            "name": "feeBasisPoints",
            "type": "u32"
          },
          {
            "name": "createdMintDecimals",
            "type": "u8"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "CreateBondingCurveParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "symbol",
            "type": "string"
          },
          {
            "name": "uri",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "GlobalAuthorityInput",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "globalAuthority",
            "type": {
              "option": "publicKey"
            }
          },
          {
            "name": "feeRecipient",
            "type": {
              "option": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "GlobalSettingsInput",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "initialTokenSupply",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialRealSolReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialRealTokenReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialVirtualSolReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialVirtualTokenReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "solLaunchThreshold",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "feeBasisPoints",
            "type": {
              "option": "u32"
            }
          },
          {
            "name": "createdMintDecimals",
            "type": {
              "option": "u8"
            }
          },
          {
            "name": "status",
            "type": {
              "option": {
                "defined": "ProgramStatus"
              }
            }
          }
        ]
      }
    },
    {
      "name": "ProgramStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Running"
          },
          {
            "name": "SwapOnly"
          },
          {
            "name": "SwapOnlyNoLaunch"
          },
          {
            "name": "Paused"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "GlobalUpdateEvent",
      "fields": [
        {
          "name": "feeRecipient",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "globalAuthority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "initialVirtualTokenReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "initialVirtualSolReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "initialRealTokenReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "initialTokenSupply",
          "type": "u64",
          "index": false
        },
        {
          "name": "feeBasisPoints",
          "type": "u32",
          "index": false
        },
        {
          "name": "solLaunchThreshold",
          "type": "u64",
          "index": false
        },
        {
          "name": "createdMintDecimals",
          "type": "u8",
          "index": false
        }
      ]
    },
    {
      "name": "CreateEvent",
      "fields": [
        {
          "name": "name",
          "type": "string",
          "index": false
        },
        {
          "name": "symbol",
          "type": "string",
          "index": false
        },
        {
          "name": "uri",
          "type": "string",
          "index": false
        },
        {
          "name": "mint",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "creator",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "virtualSolReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "virtualTokenReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "tokenTotalSupply",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "AlreadyInitialized",
      "msg": "Global Already Initialized"
    },
    {
      "code": 6001,
      "name": "NotInitialized",
      "msg": "Global Not Initialized"
    },
    {
      "code": 6002,
      "name": "InvalidAuthority",
      "msg": "Invalid Authority"
    },
    {
      "code": 6003,
      "name": "ProgramNotRunning",
      "msg": "Not in Running State"
    }
  ]
};

export const IDL: LmaofunBondingCurve = {
  "version": "0.1.0",
  "name": "lmaofun_bonding_curve",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "global",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "GlobalSettingsInput"
          }
        }
      ]
    },
    {
      "name": "setParams",
      "accounts": [
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "global",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "newAuthority",
          "isMut": false,
          "isSigner": false,
          "isOptional": true
        },
        {
          "name": "newFeeRecipient",
          "isMut": false,
          "isSigner": false,
          "isOptional": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "GlobalSettingsInput"
          }
        }
      ]
    },
    {
      "name": "createBondingCurve",
      "accounts": [
        {
          "name": "mint",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "authority",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "bondingCurve",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bondingCurveTokenAccount",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "global",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "metadata",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "tokenMetadataProgram",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "CreateBondingCurveParams"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "bondingCurve",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "virtualSolReserves",
            "type": "u64"
          },
          {
            "name": "virtualTokenReserves",
            "type": "u64"
          },
          {
            "name": "realSolReserves",
            "type": "u64"
          },
          {
            "name": "realTokenReserves",
            "type": "u64"
          },
          {
            "name": "tokenTotalSupply",
            "type": "u64"
          },
          {
            "name": "complete",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "global",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "status",
            "type": {
              "defined": "ProgramStatus"
            }
          },
          {
            "name": "initialized",
            "type": "bool"
          },
          {
            "name": "globalAuthority",
            "type": "publicKey"
          },
          {
            "name": "feeRecipient",
            "type": "publicKey"
          },
          {
            "name": "initialVirtualTokenReserves",
            "type": "u64"
          },
          {
            "name": "initialVirtualSolReserves",
            "type": "u64"
          },
          {
            "name": "initialRealTokenReserves",
            "type": "u64"
          },
          {
            "name": "initialRealSolReserves",
            "type": "u64"
          },
          {
            "name": "initialTokenSupply",
            "type": "u64"
          },
          {
            "name": "solLaunchThreshold",
            "type": "u64"
          },
          {
            "name": "feeBasisPoints",
            "type": "u32"
          },
          {
            "name": "createdMintDecimals",
            "type": "u8"
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "CreateBondingCurveParams",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "symbol",
            "type": "string"
          },
          {
            "name": "uri",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "GlobalAuthorityInput",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "globalAuthority",
            "type": {
              "option": "publicKey"
            }
          },
          {
            "name": "feeRecipient",
            "type": {
              "option": "publicKey"
            }
          }
        ]
      }
    },
    {
      "name": "GlobalSettingsInput",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "initialTokenSupply",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialRealSolReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialRealTokenReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialVirtualSolReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "initialVirtualTokenReserves",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "solLaunchThreshold",
            "type": {
              "option": "u64"
            }
          },
          {
            "name": "feeBasisPoints",
            "type": {
              "option": "u32"
            }
          },
          {
            "name": "createdMintDecimals",
            "type": {
              "option": "u8"
            }
          },
          {
            "name": "status",
            "type": {
              "option": {
                "defined": "ProgramStatus"
              }
            }
          }
        ]
      }
    },
    {
      "name": "ProgramStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Running"
          },
          {
            "name": "SwapOnly"
          },
          {
            "name": "SwapOnlyNoLaunch"
          },
          {
            "name": "Paused"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "GlobalUpdateEvent",
      "fields": [
        {
          "name": "feeRecipient",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "globalAuthority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "initialVirtualTokenReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "initialVirtualSolReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "initialRealTokenReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "initialTokenSupply",
          "type": "u64",
          "index": false
        },
        {
          "name": "feeBasisPoints",
          "type": "u32",
          "index": false
        },
        {
          "name": "solLaunchThreshold",
          "type": "u64",
          "index": false
        },
        {
          "name": "createdMintDecimals",
          "type": "u8",
          "index": false
        }
      ]
    },
    {
      "name": "CreateEvent",
      "fields": [
        {
          "name": "name",
          "type": "string",
          "index": false
        },
        {
          "name": "symbol",
          "type": "string",
          "index": false
        },
        {
          "name": "uri",
          "type": "string",
          "index": false
        },
        {
          "name": "mint",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "creator",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "virtualSolReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "virtualTokenReserves",
          "type": "u64",
          "index": false
        },
        {
          "name": "tokenTotalSupply",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "AlreadyInitialized",
      "msg": "Global Already Initialized"
    },
    {
      "code": 6001,
      "name": "NotInitialized",
      "msg": "Global Not Initialized"
    },
    {
      "code": 6002,
      "name": "InvalidAuthority",
      "msg": "Invalid Authority"
    },
    {
      "code": 6003,
      "name": "ProgramNotRunning",
      "msg": "Not in Running State"
    }
  ]
};
