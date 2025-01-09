# Romer Chain 

## Story 1: Genesis State

### Background
Before we can handle account registration, our network needs fundamental data structures for tracking organizations and tokens. We implement these using Commonware's storage capabilities, ensuring consistent and efficient data management across the network.

### TODO
[ ] Validator - Create Genesis Config for ROMER token and ROMER org
[ ] Create an easy interface when performing CRUD on Token and ORG
[ ] Initialize Genesis State in commonware_storage::metadata fro ROMER token and ROMER org

## Story 2: Key Manager

### Background
We need a common KeyManager that can be used across the monorepo. This KeyManager will need to be able to generate key pairs using both ED25519 and BLS Schemes.
It will also need to be able to create SessionKeys for a specified length of time. There will also need to be an ability to revoke the session key if the network or application crashes. So if the network or application crashes only the transactions received by the sequencer should be processed and then the key revoked.

## Story 3: Market Maker Registration Client

### Background

Organisations need a straightforward way to register their organizations and establish their network identity. We'll create this using ratatui for a clean terminal user interface that guides them through the registration process.

### TODO:
[ ] Organisation and Token should be pulled in from config

## Story 4: Login

### Background
User should be able to use the ratatui TUI in order to Login to the Sequencer using the FIX login message type. The Password should be the signature from the BLS signer

## Story 5: Session

### Background
The client applciation needs to be able to generate a BLS session and then with that send the FIX session message to establish a session and the heartbeat

## Story 6: Org creates a token

### Background
An Organisation like a stable coin issuer should be able to create a token using a FIX standard order that then creates a standardized token

