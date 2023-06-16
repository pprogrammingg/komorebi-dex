#!/bin/bash

# Resim Reset
resim reset

# Create account
export account_address=$(resim new-account | sed -n 's/.*Account component address: //p')

# Publish account
export package_address=$(resim publish . | sed -n 's/.*Success! New Package: //p')

# Create tokens
export token_name="token_a"
export token_a=$(resim run manifests/pool/make_tokens.rtm | awk -F'Resource: ' 'NF>1{print $2}')
export token_name="token_b"
export token_b=$(resim run manifests/pool/make_tokens.rtm | awk -F'Resource: ' 'NF>1{print $2}')
export token_name="token_c"
export token_c=$(resim run manifests/pool/make_tokens.rtm | awk -F'Resource: ' 'NF>1{print $2}')


export component_address=$(resim run manifests/pool/instantiate_pool.rtm | awk -F'Component: ' 'NF>1{print $2}')
# extract tracking token out of output
# export tracking_token=$(resim run manifests/pool/instantiate_pool.rtm | awk -F'Component: ' 'NF>1{print $2}')

echo "Account Address: $account_address"
echo "Package Address: $package_address"
echo "Token A: $token_a"
echo "Token B: $token_b"
echo "Token C: $token_c"
echo "Component Address: $component_address"