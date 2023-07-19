#!/bin/bash

# Resim Reset
resim reset

# Create account
export account_address=$(resim new-account | sed -n 's/.*Account component address: //p')

# Publish account
export package_address=$(resim publish . | sed -n 's/.*Success! New Package: //p')

# Create tokens
export token_name="token_a"
export token_a=$(resim run manifests/make_tokens.rtm | awk -F'Resource: ' 'NF>1{print $2}')
export token_name="token_b"
export token_b=$(resim run manifests/make_tokens.rtm | awk -F'Resource: ' 'NF>1{print $2}')
export token_name="token_c"
export token_c=$(resim run manifests/make_tokens.rtm | awk -F'Resource: ' 'NF>1{print $2}')
export token_name="token_d"
export token_d=$(resim run manifests/make_tokens.rtm | awk -F'Resource: ' 'NF>1{print $2}')

export dex_component_address=$(resim run manifests/komo_dex/new.rtm | awk -F'Component: ' 'NF>1{print $2}')

echo "Account Address: $account_address"
echo "Package Address: $package_address"
echo "Token A: $token_a"
echo "Token B: $token_b"
echo "Token C: $token_c"
echo "Token D: $token_d"
echo "DEX Component Address: $dex_component_address"