## ⚡ Run public testnet

* Modify the genesis config in chain_spec.rs
* Build spec, `./target/release/<your-project-name> build-spec --chain staging > my-staging.json`
* Change original spec to encoded raw spec, `./target/release/<your-project-name> build-spec --chain=my-staging.json --raw > my-staging-raw.json`
* Start your bootnodes, node key can be generate with command `./target/release/substrate key generate-node-key`.
  ```shell
  ./target/release/<your-project-name> \
       --node-key <your-node-key> \
       --base-path /tmp/bootnode1 \
       --chain my-staging-raw.json \
       --name bootnode1
  ```
* Start your validators
  ```shell
  ./target/release/<your-project-name> \
      --base-path  /tmp/validator1 \
      --chain   my-staging-raw.json \
      --bootnodes  /ip4/<your-bootnode-ip>/tcp/30333/p2p/<your-bootnode-peerid> \
      --name  validator1 \
      --validator 
  ```