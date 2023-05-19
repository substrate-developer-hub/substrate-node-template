# Aisland Node

This is the new version where the validators are subject to authorisation

# Aisland Node

Aisland is a super-charged blockchain that facilitates multiple decentralized web application with minimum integration effort for the developers.  
The main projet based on this blockchain is a decentralised market place:  
[https://openmarket.ae](https://openmarket.ae)

Aisland Node is based on [Substrate Framework 3.0](https://www.substrate.dev), the same used from [Polkadot](https://polkadot.network). 

  
## Hardware Requirements
- Ram: The Rust compiler uses a lot of ram to build the node, please use machine with at the least 8 GB RAM.  
- Disk: Disk space grows day by day, 100 GB disk is a good choice for now.  

## Installation

Follow these steps to get started with the Aisland Node:  

- First, complete the [basic Rust setup instructions](./doc/rust-setup.md).

- Use Rust's native `cargo` command to build and launch the Aisland Node:

```sh
cargo run --release -- --dev --tmp
```
You can build without launching:

```sh
cargo build --release
```


### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/aisland-node -h
```

## Run

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/aisland-node --dev
```

Purge the development chain's state:

```bash
./target/release/aisland-node purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/aisland-node -lruntime=debug --dev
```
### Testnet Node
You can run the node as part of the current Testnet:  
```bash
./target/release/aisland-node --chain assets/chain_spec_testnet_raw.json --port 30333 --name yourpreferredname --rpc-cors all
```
Please consider:
1) TESTNET can be reset to the genesis anytime; 
2) the AIS coin on TESTNET has no value, they are just for testings.  

### Testnet Validator
You can setup a validator on testnet. A validator is a node that writes the blocks of data and rewards in AISC.  
Please follow [this guide](doc/validator.md).   


### How to get AISC for Testnet
You can get 100 free AISC on Testnet using our free minter available at:  
[https://testnet.aislandi.io:8443](https://testnet.aisland.io:8443)  


### Bugs Reporting
For bug reporting, please open an issue on our Git Repo:  
[https://github.com/aisland-dao/aisland-blockchain/issues](https://github.com/aisland-dao/aisland-blockchain/issues)  
  

### Secure Web Socket
You might want to host a node on one server and then connect to it from a UI hosted on another. 
This will not be possible unless you set up a secure proxy for websocket connections. 
Let's see how we can set up WSS on a remote Aisland node.  
  
Note: this should only be done for sync nodes used as back-end for some dapps or projects. 
Never open websockets to your validator node - there's no reason to do that and it can only lead to security issues.  
  
In this guide we'll be using Debian 10.   
We'll assume you're using a similar OS, and that you have nginx installed (if not, run sudo apt-get install nginx).  
Start the node, for example:  
```bash
./target/release/aisland-node --chain testnet --rpc-cors all
```
The --rpc-cors mode needs to be set to all so that all external connections are allowed.  
To get WSS (secure websocket), you need an SSL certificate.  
Get a dedicated domain, redirect a domain name to your IP address, setting up an Nginx server for that domain, and finally following LetsEncrypt instructions for Nginx setup. 
This will auto-generate an SSL certificate and include it in your Nginx configuration. 
Now it's time to tell Nginx to use these certificates. The server block below is all you need, but keep in mind that you need to replace some placeholder values.  
Notably:  
SERVER_ADDRESS should be replaced by your domain name if you have it, or your server's IP address if not.  
CERT_LOCATION should be /etc/letsencrypt/live/YOUR_DOMAIN/fullchain.pem if you used Certbot, or /etc/ssl/certs/nginx-selfsigned.crt if self-signed.  
CERT_LOCATION_KEY should be /etc/letsencrypt/live/YOUR_DOMAIN/privkey.pem if you used Certbot, or /etc/ssl/private/nginx-selfsigned.key if self-signed.  
CERT_DHPARAM should be /etc/letsencrypt/ssl-dhparams.pem if you used Certbot, and /etc/ssl/certs/dhparam.pem if self-signed.  
Note that if you used Certbot, it should have made the path insertions below for you if you followed the official instructions. 
Here an example of configuration of nginx (/etc/nginx/sites-available/default)
```
server {

        server_name SERVER_ADDRESS;

        root /var/www/html;
        index index.html;

        location / {
          try_files $uri $uri/ =404;

          proxy_pass http://localhost:9944;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header Host $host;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
        }

        listen [::]:443 ssl ipv6only=on;
        listen 443 ssl;
        ssl_certificate /etc/letsencrypt/live/testnet.aisland.io/fullchain.pem; # managed by Certbot
        ssl_certificate_key /etc/letsencrypt/live/testnet.aisland.io/privkey.pem; # managed by Certbot
        
        ssl_session_cache shared:cache_nginx_SSL:1m;
        ssl_session_timeout 1440m;

        ssl_protocols TLSv1 TLSv1.1 TLSv1.2;
        ssl_prefer_server_ciphers on;

        ssl_ciphers "ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES256-SHA384:ECDHE-ECDSA-AES256-SHA:ECDHE-RSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-RSA-AES256-SHA256:DHE-RSA-AES256-SHA:ECDHE-ECDSA-DES-CBC3-SHA:ECDHE-RSA-DES-CBC3-SHA:EDH-RSA-DES-CBC3-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:DES-CBC3-SHA:!DSS";
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot
}
```

## Firewall Configuration

For a Aisland Node you should open the ports: 9933/TCP and 9934/TCP.  
If you want to reach the secure websocket, you should open 443/TCP.
A validator should not expose the RPC interface to the public.  
Here an example of a [firewall configuration](rpc/firewall.sh) for a Linux/Debian 10.


## Accounts  
### Address Format  
The address format used  is SS58. SS58 is a modification of Base-58-check from Bitcoin with some minor modifications. 
Notably, the format contains an address type prefix that identifies an address as belonging to a specific substrate network.  
We are using the prefix= "5".
For example a valid address is: 5DFJF7tY4bpbpcKPJcBTQaKuCDEPCpiz8TRjpmLeTtweqmXL  

### Address Generation  
A valid account only requires a private key that can sign on one of the supported curves and signature schemes. 
Most wallets take many steps from a mnemonic phrase to derive the account key, which affects the ability to use the same mnemonic phrase in multiple wallets.  
Wallets that use different steps will arrive at a different set of addresses from the same mnemonic.  

### Seed Generation  
Most wallets generate a mnemonic phrase for users to back up their wallets and generate a private key from the mnemonic.  
Not all wallets use the same algorithm to convert from mnemonic phrase to private key.  
Polkadot-js library uses the BIP39 dictionary for mnemonic generation, but use the entropy byte array to generate the private key, while full BIP39 wallets (like Ledger) use 2048 rounds of PBKDF2 on the mnemonic.  As such, the same mnemonic will not generate the same private keys. See Substrate BIP39 for more information.  

### Cryptography  
Aisland supports the following cryptographic key pairs and signing algorithms:  
- Ed25519  
- Sr25519 - Schnorr signatures on the Ristretto group  
- ECDSA signatures on secp256k1  
Note that the address for a secp256k1 key is the SS58 encoding of the hash of the public key in order to reduce the public key from 33 bytes to 32 bytes.  

### Account Data  
Account balance information is stored in a strructure "AccountData". Aisland primarily deals with two types of balances: free and reserved.  
For most operations, free balance is what you are interested in. It is the "power" of an account in staking and governance.   
Reserved balance represents funds that have been set aside by some operation and still belong to the account holder, but cannot be used.  
Locks are an abstraction over free balance that prevent spending for certain purposes.   
Several locks can operate on the same account, but they overlap rather than add.  
Locks are automatically added onto accounts when tasks are done on the network (e.g. leasing a parachain slot or voting).   
For example, an account could have a free balance of 200 AISC with two locks on it: 150 AISC for Transfer purposes and 100 AISC for Reserve purposes.  
The account could not make a transfer that brings its free balance below 150 AISC, but an operation could result in reserving AISC such that the free balance is below AISC, but above 100 AISC.  
Bonding tokens for staking and voting in governance referenda both utilize locks.  
Vesting is another abstraction that uses locks on free balance.  
Vesting sets a lock that decreases over time until all the funds are transferable.  

### Balances Module
The Balances module provides functionality for handling accounts and balances.  
The Balances module provides functions for:  

- Getting and setting free balances.  
- Retrieving total, reserved and unreserved balances.  
- Repatriating a reserved balance to a beneficiary account that exists.  
- Transferring a balance between accounts (when not reserved).  
- Slashing an account balance.  
- Account creation and removal.  
- Managing total issuance.  
- Setting and managing locks.  
[Further details are available here](https://substrate.dev/rustdocs/v3.0.0/pallet_balances/index.html)  


## Development Tools

You can interact with our testing node:
``` 
testnode.aisland.io 
```
using the web app hosted here:    
[https://polkadot.js.org/apps](https://polkadot.js.org/apps)  
To configure, click on the top left menu option and set the "Custom Node" field with 'wss://testnode.aisland.io'  
You may get and error about "Not recognised data types".  
Click on "Settings","Developer" and copy/paste the [data types of this blockchain](assets/types.json).  


## Development Libraries

[JavaScript - Polkadot-JS API](https://polkadot.js.org/docs/api/)  
The Polkadot-JS API is a javascript library of interfaces for communicating with Substrate nodes like Aisland node.  
The API provides application developers the ability to query a node and submit signed transaction using Javascript.  

[JavaScript Polkadot-JS Extension](https://github.com/polkadot-js/apps)    
The Polkadot-JS Extension is a simple extension for managing accounts in a browser extension and allowing the signing of extrinsics using these accounts.  
It also provides simple interface for interacting with extension-compliant dApps.  

[Python - py-substrate-interface](https://github.com/polkascan/py-substrate-interface)   
py-substrate-interface is a Python library for interacting with the Aisland RPC.   
It supports a wide range of capabilities and powers the Polkascan multi-chain block explorer. This library is maintained by Polkascan Foundation.  

[Rust - Substrate-subtxt](https://github.com/paritytech/substrate-subxt)
A Rust library to submit extrinsics to Aisland node via RPC.  

[Kotlin - Substrate-client-Kotlin](https://github.com/NodleCode/substrate-client-kotlin)
Substrate-client-kotlin is client library to interact with a substrate-based chain like Aisland. 
It uses the API available from the RPC endpoint only (no sidecar). As of today it provides the following functionality:
- compatible with substrate 3.0 
- ed25519 wallet creation
- get account info (balance)
- sign extrinsic and send (immortal era)
- estimate fee








 
