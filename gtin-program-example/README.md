# GTIN program example 
This is an ongaoing project for building a sample agreement program between consourtiom organizations. An organization can issue a document and other organizations can set their status about the document. In this initial phase an organization will issue a GTIN and other organizations can update their status. The current functionality only supports publishing a GTIN and we are working on the update_org_state functionality. 

### Build the payload

`cd payload`

`cargo build`

`./target/debug/payload`

### Build the WASM smart contract
`cd process`

`cargo build --target wasm32-unknown-unknown --release`

##### Set the currect path for wasm in the gtin.yaml file

## Ruinning the sabre smart contract
### Make sure
-The latest sabre tp and cli built and installed from the master branch of the sabre git repo

-Admin key is set properly in the sawtooth settings
### Running the example
```
sabre cr --create gitin_example --owner {admin public key} --url http://127.0.0.1:8008
sabre upload --filename /home/ubuntu/gitin-example/gitin.yaml  --url http://127.0.0.1:8008 

sabre ns --create 123456 --owner {admin public key} --url http://127.0.0.1:8008
sabre perm  123456 gitin_example --read --write --url http://127.0.0.1:8008

sawtooth block list

sabre exec --contract gitin_example:1.0 --payload /home/ubuntu/gitin-example/payload/payload --inputs  123456 --outputs  123456 --url http://127.0.0.1:8008

sawtooth block list

```
After executing the example you will see a block is been added and if the advanced log level is enabled from the sabre tp, you should see a similar log
```
2019-05-07 12:04:52 INFO  [sawtooth_sabre::handler] Action: Execute Contract ["00ec0194f427efefa74c1230c3e93c35104dcbaa8ff71ba4537583ed83c0449d607c4e", "00ec0213eff503a74c727e16b5c0fbd348c24b74755ba7e47bc6f50b1851a1385468d9", "00ec0078fe085f6edb13bbd519c9282dff5d7fc3bc4966867eaac0cf2b21ffd3622ecd", "00ec00d4eb478d386d0d216048a54575feda3450f2fcc42b3d15f80e258c16010da412", "1cf126", "cad11d"] ["00ec0194f427efefa74c1230c3e93c35104dcbaa8ff71ba4537583ed83c0449d607c4e", "00ec0213eff503a74c727e16b5c0fbd348c24b74755ba7e47bc6f50b1851a1385468d9", "00ec0078fe085f6edb13bbd519c9282dff5d7fc3bc4966867eaac0cf2b21ffd3622ecd", "1cf126"]
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_module] Payload written to memory
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_module] Signer written to memory
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_module] Signature written to memory
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_externals] ALLOC Execution time: 0 secs 0 ms
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_externals] create_collection: 208
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_externals] ALLOC Execution time: 0 secs 0 ms
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_externals] adding to collection: 278
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_externals] Attempting to set state, entries: [("1cf126f0cec9c365d0614fe7fa52f2b8b33834c3a1530c310dc8ee8aa8f9fa009d783e", [161, 108, 104, 101, 108, 108, 111, 119, 95, 119, 111, 114, 108, 100, 1])]
2019-05-07 12:04:52 INFO  [sawtooth_sabre::wasm_executor::wasm_externals] SET_STATE Execution time: 0 secs 2 ms
2019-05-07 12:04:52 INFO  [sawtooth_sdk::processor] TP_PROCESS_REQUEST sending TpProcessResponse: OK
```

We are working on a cli application to visualize the state.
