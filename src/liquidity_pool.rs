use scrypto::prelude::*;
use crate::utils::*;

#[blueprint]
mod pool {

    /// Pool encapsulate liquidity pool fields and methods
    /// Uses constant market maker : x*y=k to maintain the ratios of a and b tokens    
    pub struct Pool{
       /// use a more flexible dynamic way to store the vault addresses.
       /// Note: Pool object always have exactly 2 vaults
       vaults: HashMap<ResourceAddress, Vault>,

       /// Tracking token is used keep track of the ratio user's contribution 
       /// proportional to the total pool amount. This ratio will be used to calculate
       /// fees distributed and also for when user withdraws their liquidity out of the pool
       tracking_token_address: ResourceAddress,

       /// Admin badge used to mint and burn tracking token for this pool
       tracking_token_admin_badge: Vault,

       /// Decimal Amount between 0 to 100 representing the percentage fee 
       /// paid to liquidity pool (to be distributed to the liquidity providers 
       /// based on thier LP tracking token ratio )
       fee_to_pool: Decimal
    }

    impl Pool {
        /// Creates a new pool based on two resources addresses and fee amount to go to the pool
        /// validations include:
        ///  - Check the two resource addresses are not the same
        ///  - Check resources are both fungible 
        ///  - Check the input token buckets are not empty
        ///  - Check fee amount set is decimal between 0 to 100
        /// Returns LP Tracking Token (for the initial liquidity provider
        /// Note: no change amount is returned as pool ratio is not established yet
        pub fn instantiate_pool(
            token1: Bucket,
            token2: Bucket,
            fee_to_pool: Decimal) -> (PoolComponent, Bucket) {
            // Check token addresses are not the same
            assert_ne!(
                token1.resource_address(), token2.resource_address(),
                "[Pool Creation]: Liquidity pools may only be created between two different tokens."
            );

            // Check resources neither is Non-Fungible
            assert_eq!(
                borrow_resource_manager!(token1.resource_address()).resource_type().is_fungible(), true,
                "[Pool Creation]: Both assets must be fungible."
            );
            assert_eq!(
                borrow_resource_manager!(token2.resource_address()).resource_type().is_fungible(), true,
                "[Pool Creation]: Both assets must be fungible."
            );

            // Check the input token buckets are not empty
            assert!(
                !token1.is_empty() & !token2.is_empty(), 
                "[Pool Creation]: Can't create a pool from an empty bucket."
            );
            
            // Check fee amount set is decimal between 0 to 100
            assert!(
                (fee_to_pool >= Decimal::zero()) & (fee_to_pool <= dec!("100")), 
                "[Pool Creation]: Fee must be between 0 and 100"
            );                

            // Validation is done
            info!(
                "[instantiate_pool]: validation of inputs done. Inputs: token1 {:?}: {}, token2 {:?}: {}, fee_to_pool: {}", 
                token1.resource_address(), token1.amount(), token2.resource_address(), token2.amount(), fee_to_pool
            );

            // Sort and build Hashmap of the two resource token addresses
            let (bucket1, bucket2): (Bucket, Bucket) = sort_buckets(token1, token2);
            let addresses: (ResourceAddress, ResourceAddress) = (bucket1.resource_address(), bucket2.resource_address());
            
            let lp_id: String = format!("{:?}-{:?}", addresses.0, addresses.1);
            let pair_name: String = address_pair_symbol(addresses.0, addresses.1);

            info!(
                "[Pool Creation]: Creating new pool between tokens: {}, of name: {}, Ratio: {}:{}", 
                lp_id, pair_name, bucket1.amount(), bucket2.amount()
            );
            
            let mut vaults: HashMap<ResourceAddress, Vault> = HashMap::new();
            vaults.insert(bucket1.resource_address(), Vault::with_bucket(bucket1));
            vaults.insert(bucket2.resource_address(), Vault::with_bucket(bucket2));

            // Create Admin badge to give authority for minting and burning LP tracking tokens
            let tracking_token_admin_badge: Bucket = ResourceBuilder::new_fungible()
            .divisibility(DIVISIBILITY_NONE)
            .metadata("name", "Tracking Token Admin Badge")
            .metadata("symbol", "TTAB")
            .metadata("description", "This is an admin badge that has the authority to mint and burn tracking tokens")
            .metadata("lp_id", format!("{}", lp_id))
            .mint_initial_supply(1);

            // Creating the tracking tokens and minting the amount owed to the initial liquidity provider
            let tracking_tokens: Bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", format!("{} LP Tracking Token", pair_name))
                .metadata("symbol", "TT")
                .metadata("description", "A tracking token used to track the percentage ownership of liquidity providers over the liquidity pool")
                .metadata("lp_id", format!("{}", lp_id))
                .mintable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .burnable(rule!(require(tracking_token_admin_badge.resource_address())), LOCKED)
                .mint_initial_supply(100);

            // Creating the liquidity pool component and instantiating it
            let liquidity_pool = Self { 
                vaults: vaults,
                tracking_token_address: tracking_tokens.resource_address(),
                tracking_token_admin_badge: Vault::with_bucket(tracking_token_admin_badge),
                fee_to_pool: fee_to_pool,
            }
            .instantiate()
            // .globalize() NOTE: comment out if running manifests under `./manifests/pool` and using setup_pool_test.sh
            ;
            
            return (liquidity_pool, tracking_tokens);
        }

        /// Checks if the given address belongs to this pool or not.
        /// 
        /// This method is used to check if a given resource address belongs to one of the tokens in this liquidity pool
        /// or not. A resource belongs to a liquidity pool if its address is in the addresses in the `vaults` HashMap.
        /// 
        /// # Arguments:
        /// 
        /// * `address` (ResourceAddress) - The address of the resource that we wish to check if it belongs to the pool.
        /// 
        /// # Returns:
        /// 
        /// * `bool` - A boolean of whether the address belongs to this pool or not.
        pub fn belongs_to_pool(
            &self, 
            address: ResourceAddress
        ) -> bool {
            return self.vaults.contains_key(&address);
        }

        /// Asserts that the given address belongs to the pool.
        /// 
        /// This is a quick assert method that checks if a given address belongs to the pool or not. If the address does
        /// not belong to the pool, then an assertion error (panic) occurs and the message given is outputted.
        /// 
        /// # Arguments:
        /// 
        /// * `address` (ResourceAddress) - The address of the resource that we wish to check if it belongs to the pool.
        /// * `label` (String) - The label of the method that called this assert method. As an example, if the swap 
        /// method were to call this method, then the label would be `Swap` so that it's clear where the assertion error
        /// took place.
        pub fn assert_belongs_to_pool(
            &self, 
            address: ResourceAddress, 
            label: String
        ) {
            assert!(
                self.belongs_to_pool(address), 
                "[{}]: The provided resource address does not belong to the pool.", 
                label
            );
        }

        /// Gets the resource addresses of the tokens in this liquidity pool and returns them as a `Vec<ResourceAddress>`.
        /// 
        /// # Returns:
        /// 
        /// `Vec<ResourceAddress>` - A vector of the resource addresses of the tokens in this liquidity pool.
        pub fn addresses(&self) -> Vec<ResourceAddress> {
            return self.vaults.keys().cloned().collect::<Vec<ResourceAddress>>();
        }

        /// Gets the name of the given liquidity pool from the symbols of the two tokens.
        /// 
        /// # Returns:
        /// 
        /// `String` - A string of the pair symbol
        pub fn name(&self) -> String {
            let addresses: Vec<ResourceAddress> = self.addresses();
            return address_pair_symbol(addresses[0], addresses[1]);
        }

        /// This method takes in a resource address and if this resource address belongs to the pool it returns the 
        /// address of the other token in this liquidity pool.
        /// 
        /// This method performs a number of checks before resource address is obtained:
        /// 
        /// * **Check 1:** Checks that the resource address given does indeed belong to this liquidity pool.
        /// 
        /// # Arguments
        /// 
        /// * `resource_address` (ResourceAddress) - The resource address for a token from the pool.
        /// 
        /// # Returns:
        /// 
        /// * `ResourceAddress` - The address of the other token in this pool.
        pub fn other_resource_address(
            &self,
            resource_address: ResourceAddress
        ) -> ResourceAddress {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(resource_address, String::from("Argument Resource Address"));

            // Checking which of the addresses was provided as an argument and returning the other address.
            let addresses: Vec<ResourceAddress> = self.addresses();
            return if addresses[0] == resource_address {addresses[1]} else {addresses[0]};
        }

        /// Calculates the k in the constant market maker equation: `x * y = k`.
        /// 
        /// # Returns:
        /// 
        /// `Decimal` - A decimal value of the reserves amount of Token A and Token B multiplied by one another.
        pub fn k(&self) -> Decimal {
            let addresses: Vec<ResourceAddress> = self.addresses();
            return self.vaults[&addresses[0]].amount() * self.vaults[&addresses[1]].amount()
        }

        /// This method calculates the amount of output tokens that would be received for a given amount of an input
        /// token. This is calculated through the constant market maker function `x * y = k`. 
        /// 
        /// This method performs a number of checks before the calculation is done:
        /// 
        /// * **Check 1:** Checks that the provided resource address belongs to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `input_resource_address` (ResourceAddress) - The resource address of the input token.
        /// * `input_amount` (Decimal) - The amount of input tokens to calculate the output for.
        /// 
        /// # Returns:
        /// 
        /// * `Decimal` - The output amount for the given input.
        /// 
        /// # Note:
        /// 
        /// This method is equivalent to finding `dy` in the equation `(x + rdx)(y - dy) = xy` where the symbols used
        /// mean the following:
        /// 
        /// * `x` - The amount of reserves of token x (the input token)
        /// * `y` - The amount of reserves of token y (the output token)
        /// * `dx` - The amount of input tokens
        /// * `dy` - The amount of output tokens
        /// * `r` - The fee modifier where `r = (100 - fee) / 100`
        pub fn calculate_output_amount(
            &self,
            input_resource_address: ResourceAddress,
            input_amount: Decimal
        ) -> Decimal {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(input_resource_address, String::from("Calculate Output"));

            let x: Decimal = self.vaults[&input_resource_address].amount();
            let y: Decimal = self.vaults[&self.other_resource_address(input_resource_address)].amount();
            let dx: Decimal = input_amount;
            let r: Decimal = (dec!("100") - self.fee_to_pool) / dec!("100");

            let dy: Decimal = (dx * r * y) / ( x + r * dx );
            return dy;
        }

        /// This method calculates the amount of input tokens that would be required to receive the specified amount of
        /// output tokens. This is calculated through the constant market maker function `x * y = k`. 
        /// 
        /// This method performs a number of checks before the calculation is done:
        /// 
        /// * **Check 1:** Checks that the provided resource address belongs to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `output_resource_address` (ResourceAddress) - The resource address of the output token.
        /// * `output_amount` (Decimal) - The amount of output tokens to calculate the input for.
        /// 
        /// # Returns:
        /// 
        /// * `Decimal` - The input amount for the given output.
        /// 
        /// # Note:
        /// 
        /// This method is equivalent to finding `dx` in the equation `(x + rdx)(y - dy) = xy` where the symbols used
        /// mean the following:
        /// 
        /// * `x` - The amount of reserves of token x (the input token)
        /// * `y` - The amount of reserves of token y (the output token)
        /// * `dx` - The amount of input tokens
        /// * `dy` - The amount of output tokens
        /// * `r` - The fee modifier where `r = (100 - fee) / 100`
        pub fn calculate_input_amount(
            &self,
            output_resource_address: ResourceAddress,
            output_amount: Decimal
        ) -> Decimal {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(output_resource_address, String::from("Calculate Input"));

            let x: Decimal = self.vaults[&self.other_resource_address(output_resource_address)].amount();
            let y: Decimal = self.vaults[&output_resource_address].amount();
            let dy: Decimal = output_amount;
            let r: Decimal = (dec!("100") - self.fee_to_pool) / dec!("100");

            let dx: Decimal = (dy * x) / (r * (y - dy));
            return dx;
        }

        /// Deposits a bucket of tokens into this liquidity pool.
        /// 
        /// This method determines if a given bucket of tokens belongs to the liquidity pool or not. If it's found that
        /// they belong to the pool, then this method finds the appropriate vault to store the tokens and deposits them
        /// to that vault.
        /// 
        /// This method performs a number of checks before the deposit is made:
        /// 
        /// * **Check 1:** Checks that the resource address given does indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `bucket` (Bucket) - A buckets of the tokens to deposit into the liquidity pool
        fn deposit(
            &mut self,
            bucket: Bucket 
        ) {
            // Checking if the passed resource address belongs to this pool.
            self.assert_belongs_to_pool(bucket.resource_address(), String::from("Deposit"));

            self.vaults.get_mut(&bucket.resource_address()).unwrap().put(bucket);
        }

        /// Withdraws tokens from the liquidity pool.
        /// 
        /// This method is used to withdraw a specific amount of tokens from the liquidity pool. 
        /// 
        /// This method performs a number of checks before the withdraw is made:
        /// 
        /// * **Check 1:** Checks that the resource address given does indeed belong to this liquidity pool.
        /// * **Check 2:** Checks that the there is enough liquidity to perform the withdraw.
        /// 
        /// # Arguments:
        /// 
        /// * `resource_address` (ResourceAddress) - The address of the resource to withdraw from the liquidity pool.
        /// * `amount` (Decimal) - The amount of tokens to withdraw from the liquidity pool.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the withdrawn tokens.
        fn withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal
        ) -> Bucket {
            // Performing the checks to ensure tha the withdraw can actually go through
            self.assert_belongs_to_pool(resource_address, String::from("Withdraw"));
            
            // Getting the vault of that resource and checking if there is enough liquidity to perform the withdraw.
            let vault: &mut Vault = self.vaults.get_mut(&resource_address).unwrap();
            assert!(
                vault.amount() >= amount,
                "[Withdraw]: Not enough liquidity available for the withdraw."
            );

            return vault.take(amount);
        }

        /// Adds liquidity to this liquidity pool in exchange for liquidity provider tracking tokens.
        /// 
        /// This method calculates the appropriate amount of liquidity that may be added to the liquidity pool from the
        /// two token buckets provided in this method call. This method then adds the liquidity and issues tracking 
        /// tokens to the liquidity provider to keep track of their percentage ownership over the pool. 
        /// 
        /// This method performs a number of checks before liquidity is added to the pool:
        /// 
        /// * **Check 1:** Checks that the buckets passed are of tokens that belong to this liquidity pool.
        /// * **Check 2:** Checks that the buckets passed are not empty.
        /// 
        /// From the perspective of adding liquidity, these are all of the checks that need to be done. The Pool 
        /// component does not need to perform any additional checks when liquidity is being added.
        /// 
        /// # Arguments:
        /// 
        /// * `token1` (Bucket) - A bucket containing the amount of the first token to add to the pool.
        /// * `token2` (Bucket) - A bucket containing the amount of the second token to add to the pool.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the remaining tokens of the `token1` type.
        /// * `Bucket` - A bucket of the remaining tokens of the `token2` type.
        /// * `Bucket` - A bucket of the tracking tokens issued to the liquidity provider.
        /// 
        /// # Note:
        /// 
        /// This method uses the ratio of the tokens in the reserve to the ratio of the supplied tokens to determine the
        /// appropriate amount of tokens which need to be supplied. To better explain it, let's use some symbols to make
        /// the ratios a little bit clearer. Say that `m` and `n` are the tokens reserves of the two tokens stored in 
        /// the vaults respectively. Say that `dm` and `dn` are positive non-zero `Decimal` numbers of the amount of 
        /// liquidity which the provider wishes to add to the liquidity pool. If `(m / n)/(dm / dn) = 1` then all of the
        /// tokens sent in the transactions will be added to the liquidity. However, what about the other cases where 
        /// this is not equal to one? We could say that we have three cases in total:
        /// 
        /// * `(m / n) = (dm / dn)` - There is no excess of tokens and all of the tokens given to the method may be 
        /// added to the liquidity pool. The no excess on both sides case could also happen if a liquidity pool has been
        /// emptied out and this is the new round of new liquidity being added. In this case, the buckets of tokens will
        /// be taken all with no excess or anything remaining.
        /// * `(m / n) < (dm / dn)` - In this case, there would be an excess of `dm` meaning that `dn` would be consumed
        /// fully while `dm` would be consumed partially.
        /// * `(m / n) > (dm / dn)` - In this case, there would be an excess of `dn` meaning that `dm` would be consumed
        /// fully while `dn` would be consumed partially.
        /// 
        /// This method takes into account all three of these cases and appropriately accounts for them.
        pub fn add_liquidity(
            &mut self,
            token1: Bucket,
            token2: Bucket,
        ) -> (Bucket, Bucket, Bucket) {
            // Checking if the tokens belong to this liquidity pool.
            self.assert_belongs_to_pool(token1.resource_address(), String::from("Add Liquidity"));
            self.assert_belongs_to_pool(token2.resource_address(), String::from("Add Liquidity"));

            // Checking that the buckets passed are not empty
            assert!(!token1.is_empty(), "[Add Liquidity]: Can not add liquidity from an empty bucket");
            assert!(!token2.is_empty(), "[Add Liquidity]: Can not add liquidity from an empty bucket");
            info!(
                "[Add Liquidity]: Requested adding liquidity of amounts, {:?}: {}, {:?}: {}", 
                token1.resource_address(), token1.amount(), token2.resource_address(), token2.amount()
            );

            // Sorting out the two buckets passed and getting the values of `dm` and `dn`.
            let (mut bucket1, mut bucket2): (Bucket, Bucket) = sort_buckets(token1, token2);
            let dm: Decimal = bucket1.amount();
            let dn: Decimal = bucket2.amount();

            // Getting the values of m and n from the liquidity pool vaults (What is already in the pool)
            let m: Decimal = self.vaults[&bucket1.resource_address()].amount();
            let n: Decimal = self.vaults[&bucket2.resource_address()].amount();
            info!(
                "[Add Liquidity]: Current reserves: {:?}: {}, {:?}: {}",
                bucket1.resource_address(), m, bucket2.resource_address(), n
            );

            // Computing the amount of tokens to deposit into the liquidity pool from each one of the buckets passed
            let (amount1, amount2): (Decimal, Decimal) = if ((m == Decimal::zero()) | (n == Decimal::zero())) | ((m * dn) == (n * dm)) { // Case 1
                info!("Case 1");
                (dm, dn)
            } else if (m / n) < (dm / dn) { // Case 2
                info!("Case 2");
                (dn * m / n, dn)
            } else { // Case 3
                info!("Case 3");
                (dm, dm * n / m)
            };
            info!(
                "[Add Liquidity]: Liquidity amount to add: {:?}: {}, {:?}: {}", 
                bucket1.resource_address(), amount1, bucket2.resource_address(), amount2
            );

            // Depositing the amount of tokens calculated into the liquidity pool
            self.deposit(bucket1.take(amount1));
            self.deposit(bucket2.take(amount2));

            // Computing the amount of tracking tokens that the liquidity provider is owed and minting them. In the case
            // that the liquidity pool has been completely emptied out (tracking_tokens_manager.total_supply() == 0)  
            // then the first person to supply liquidity back into the pool again would be given 100 tracking tokens.
            let tracking_tokens_manager: ResourceManager = borrow_resource_manager!(self.tracking_token_address);
            let tracking_amount: Decimal = if tracking_tokens_manager.total_supply() == Decimal::zero() { 
                dec!("100.00") 
            } else {
                amount1 * tracking_tokens_manager.total_supply() / m
            };
            let tracking_tokens: Bucket = self.tracking_token_admin_badge.authorize(|| {
                tracking_tokens_manager.mint(tracking_amount)
            });
            info!("[Add Liquidity]: Owed amount of tracking tokens: {}", tracking_amount);

            // Returning the remaining tokens from `token1`, `token2`, and the tracking tokens
            return (bucket1, bucket2, tracking_tokens);
        }

        /// Removes the percentage of the liquidity owed to this liquidity provider.
        /// 
        /// This method is used to calculate the amount of tokens owed to the liquidity provider and take them out of
        /// the liquidity pool and return them to the liquidity provider. If the liquidity provider wishes to only take
        /// out a portion of their liquidity instead of their total liquidity they can provide a `tracking_tokens` 
        /// bucket that does not contain all of their tracking tokens (example: if they want to withdraw 50% of their
        /// liquidity, they can put 50% of their tracking tokens into the `tracking_tokens` bucket.). When the liquidity
        /// provider is given the tokens that they are owed, the tracking tokens are burned.
        /// 
        /// This method performs a number of checks before liquidity removed from the pool:
        /// 
        /// * **Check 1:** Checks to ensure that the tracking tokens passed do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tracking_tokens` (Bucket) - A bucket of the tracking tokens that the liquidity provider wishes to 
        /// exchange for their share of the liquidity.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A Bucket of the share of the liquidity provider of the first token.
        /// * `Bucket` - A Bucket of the share of the liquidity provider of the second token.
        pub fn remove_liquidity(
            &mut self,
            tracking_tokens: Bucket
        ) -> (Bucket, Bucket) {
            // Checking the resource address of the tracking tokens passed to ensure that they do indeed belong to this
            // liquidity pool.
            assert_eq!(
                tracking_tokens.resource_address(), self.tracking_token_address,
                "[Remove Liquidity]: The tracking tokens given do not belong to this liquidity pool."
            );

            // Calculating the percentage ownership that the tracking tokens amount corresponds to
            let tracking_tokens_manager: ResourceManager = borrow_resource_manager!(self.tracking_token_address);
            let percentage: Decimal = tracking_tokens.amount() / tracking_tokens_manager.total_supply();

            info!("User about to withdraw {} of the liquidity", percentage);
            
            // Burning the tracking tokens
            self.tracking_token_admin_badge.authorize(|| {
                tracking_tokens.burn();
            });

            // Withdrawing the amount of tokens owed to this liquidity provider
            let addresses: Vec<ResourceAddress> = self.addresses();
            let bucket1: Bucket = self.withdraw(addresses[0], self.vaults[&addresses[0]].amount() * percentage);
            let bucket2: Bucket = self.withdraw(addresses[1], self.vaults[&addresses[1]].amount() * percentage);

            return (bucket1, bucket2);
        }

        /// Performs the swap of tokens and takes the pool fee in the process
        /// 
        /// This method is used to perform the swapping of one token with another token. This is a low level method
        /// that does not perform a lot of checks on the tokens being swapped, slippage, or things of that sort. It is
        /// up to the caller of the this method (typically another method / function) to perform the checks needed. 
        /// When swaps are performed through this method, the associated fee of the pool is taken when this swap method
        /// is called.
        /// 
        /// This method performs a number of checks before the swap is performed:
        /// 
        /// * **Check 1:** Checks that the tokens in the bucket do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tokens` (Bucket) - A bucket containing the input tokens that will be swapped for other tokens.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the other tokens.
        pub fn swap(
            &mut self,
            tokens: Bucket
        ) -> Bucket {
            // Checking if the tokens belong to this liquidity pool.
            self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap"));

            // For debugging purposes, get current vault reserves
            let resource_address_1 = tokens.resource_address();
            let resource_address_2 = self.other_resource_address(tokens.resource_address());
            let vault_one_amount: Decimal = self.vaults[&resource_address_1].amount();
            let vault_two_amount: Decimal = self.vaults[&resource_address_2].amount();

            info!(
                "[Add Liquidity]: Current reserves: {:?}: {}, {:?}: {}",
                resource_address_1, vault_one_amount, resource_address_2, vault_two_amount
            );
            
            info!("[Swap]: K before swap: {}", self.k());

            // Calculating the output amount for the given input amount of tokens and withdrawing it from the vault
            let output_amount: Decimal = self.calculate_output_amount(tokens.resource_address(), tokens.amount());
            info!("[Swap]: output amount is : {}", output_amount);
            let output_tokens: Bucket = self.withdraw(
                self.other_resource_address(tokens.resource_address()), 
                output_amount
            );

            // Depositing the tokens into the liquidity pool and returning a bucket of the swapped tokens.
            self.deposit(tokens);
            info!("[Swap]: K after swap: {}", self.k());
            return output_tokens;
        }

        /// Swaps all of the given tokens for the other token.
        /// 
        /// This method is used to swap all of the given token (let's say Token A) for their equivalent amount of the
        /// other token (let's say Token B). This method supports slippage in the form of the `min_amount_out` where
        /// the caller is given the option to specify the minimum amount of Token B that they're willing to accept for
        /// the swap to go through. If the output amount does not satisfy the `min_amount_out` specified by the user 
        /// then this method fails and all of the parties involved get their tokens back.
        /// 
        /// This method performs a number of checks before the swap is performed:
        /// 
        /// * **Check 1:** Checks that the tokens in the bucket do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tokens` (Bucket) - A bucket containing the input tokens that will be swapped for other tokens.
        /// * `min_amount_out` (Decimal) - The minimum amount of tokens that the caller is willing to accept before the 
        /// method fails.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the other tokens.
        pub fn swap_exact_tokens_for_tokens(
            &mut self,
            tokens: Bucket,
            min_amount_out: Decimal
        ) -> Bucket {
            // Checking that the bucket passed does indeed belong to this liquidity pool
            self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap Exact"));
            
            // Performing the token swap and checking if the amount is suitable for the caller or not. This is one of 
            // the best and coolest things that I have seen in Scrypto so far. Even though in the `self.swap(tokens)` 
            // line to took the tokens from the vault and are now ready to give it to the user, if the assert statement
            // fails then everything that took place in this method call goes back to how it was before hand. 
            // Essentially reverting history and going back in time to say that the withdraw from the vault never took
            // place and that the funds are still in the vault.
            let output_tokens: Bucket = self.swap(tokens);
            assert!(output_tokens.amount() >= min_amount_out, "[Swap Exact]: min_amount_out not satisfied.");

            return output_tokens;
        }

        /// Swaps tokens for a specific amount of tokens
        /// 
        /// This method is used when the user wants to swap a token for a specific amount of another token. This method
        /// calculates the input amount required to get the desired output and if the amount required is provided in the
        /// tokens bucket then the swap takes place and the user gets back two buckets: a bucket of the remaining input
        /// tokens and another bucket of the swapped tokens.
        /// 
        /// This method performs a number of checks before the swap is performed:
        /// 
        /// * **Check 1:** Checks that the tokens in the bucket do indeed belong to this liquidity pool.
        /// 
        /// # Arguments:
        /// 
        /// * `tokens` (Bucket) - A bucket containing the tokens that the user wishes to swap.
        /// * `output_amount` (Decimal) - A decimal of the specific amount of output that the user wishes to receive 
        /// from this swap.
        /// 
        /// # Returns:
        /// 
        /// * `Bucket` - A bucket of the other tokens.
        /// * `Bucket` - A bucket of the remaining input tokens.
        pub fn swap_tokens_for_exact_tokens(
            &mut self,
            mut tokens: Bucket,
            output_amount: Decimal
        ) -> (Bucket, Bucket) {
            // Checking that the bucket passed does indeed belong to this liquidity pool
            self.assert_belongs_to_pool(tokens.resource_address(), String::from("Swap For Exact"));

            // Calculating the amount of input tokens that would be required to produce the desired amount of output 
            // tokens
            let input_required: Decimal = self.calculate_input_amount(
                self.other_resource_address(tokens.resource_address()), 
                output_amount
            );
            assert!(
                tokens.amount() >= input_required,
                "[Swap For Exact]: Not enough input for the desired amount of output. Input required is {}",
                input_required
            );

            // Depositing the amount of input required into the vaults and taking out the requested amount
            info!("[Swap For Exact]: K before swap: {}", self.k());
            self.deposit(tokens.take(input_required));
            let output_tokens: Bucket = self.withdraw(
                self.other_resource_address(tokens.resource_address()), 
                output_amount
            );
            info!("[Swap For Exact]: K after swap: {}", self.k());
            info!("[Swap For Exact]: Amount gievn out: {}", output_tokens.amount());
            return (output_tokens, tokens);
        }
    }
}