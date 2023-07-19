- Repo is based on Radex linked [here](https://github.com/radixdlt/scrypto-challenges/tree/main/1-exchanges/RaDEX/src).


- For the purpose of testing, one can run the manifests under `manifests/pool` and `manifests/komodex`. 

- Special notes for `pool` testing:
    - For manifest under the `pool` folder, uncomment `globalize()` in `instantiate_pool` method and update the package. 
    - Run `source setup_pool_test.sh`


# Tests

## komoDEX.rs

### New

| Status   |
| -------- |
| Pass     |

### New Liquidity Pool

| Params        | Amount   | Result   | Status
| --------      | -------- | -------- |--------
| token_c (x)   | 2500     |          | Pass
| token_d (y)   | 150      |          |   
| dx            |          |          |
| dy            |          |          |
| tt_amount     |          | 100      |
| r             | 0.999975 |          |

### Add Liquidity

| Params        | Amount   | Result   | Status
| --------      | -------- | -------- |--------
| token_a (x)   | 1000     |          | pass
| token_b (y)   | 50       |          |   
| dx            |          |          |
| dy            |          |          |
| tt_amount     |          |          |
| r             | 0.999975 |          |

### Remove Liquidity

| Params        | Amount   | Result   | Status
| --------      | -------- | -------- |--------
| token_a (x)   | 1000     | 400      | pass
| token_b (y)   | 50       | 20       |   
| dx            |          |          |
| dy            |          |          |
| rm tt_amount  | 40       |          |
| r             | 0.999975 |          |


### Swap

| Params        | Amount   | Result   | Status
| --------      | -------- | -------- |--------
| token_a (x)   | 1000     |          | Pass
| token_b (y)   | 50       |          |   
| dx            | 100      |          |
| dy            |          | 4.55     |
| r             | 0.999975 |          |

| Params        | Amount   | Result   | Status
| --------      | -------- | -------- |--------
| token_a (x)   | 1100     |          | Pass
| token_b (y)   | 45.45    |          |   
| dx            | 20       |          |
| dy            |          | 0.811    |
| r             | 0.999975 |          |