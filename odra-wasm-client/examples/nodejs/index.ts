import { 
    GetBalanceResult,
    OdraWasmClient,
    Address,
    Verbosity,
    Bytes,
    Cep18Client
} from "odra-wasm-client";

import {
  HttpHandler,
  RpcClient,
  KeyAlgorithm,
  PrivateKey,
  makeCsprTransferDeploy
} from 'casper-js-sdk';



const address = new Address("account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364");
const client = new OdraWasmClient("http://95.165.150.165:7777", Verbosity.High);

const cep18 = new Cep18Client(client, address);
