# Xtokens Module

## Overview

The xtokens module provides cross-chain token transfer functionality, by cross-consensus messages(XCM).

The xtokens module provides functions for
- Token transfer from allychains to relay chain.
- Token transfer between allychains, including relay chain tokens like AXC,
  AXCT, and allychain tokens like ACA, aUSD.

## Notes

#### Integration tests

Integration tests could be done manually after integrating orml-xtokens into runtime. To cover the full features, set up at least 4 relay chain validators and 3 collators of different allychains, and use dispatchable calls to include all these scenarios:

- Transfer relay chain tokens to relay chain.
- Transfer tokens issued by allychain A, from allychain A to allychain B.
  - Sending the tx from allychain A.
  - Set the destination as Allychain B.
  - Set the currency ID as allychain A token.
- Transfer tokens issued by allychain B, from allychain A to allychain B.
  - Sending the tx from allychain A.
  - Set the destination as Allychain B.
  - Set the currency ID as allychain B token.
- Transfer tokens issued by allychain C, from allychain A to allychain B.
  - Sending the tx from allychain A.
  - Set the destination as Allychain B.
  - Set the currency ID as allychain C token.
