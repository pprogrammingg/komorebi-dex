use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

/// Test are in Progress - not completed or functional yet
#[test]
fn test_komorebi() {
    // Setup the environment
    let mut test_runner = TestRunner::builder().build();

    // Create an account
    let (public_key, _private_key, admin_component) = test_runner.new_allocated_account();
    println!("{:?} Public Key is \n", public_key);

    let mut token_a_info: BTreeMap<String, String> = BTreeMap::new();
    token_a_info.insert("name".to_string(), "token_a".to_string());
    token_a_info.insert("symbol".to_string(), "TOK_A".to_string());

    let mut token_b_info: BTreeMap<String, String> = BTreeMap::new();
    token_b_info.insert("name".to_string(), "token_b".to_string());
    token_b_info.insert("symbol".to_string(), "TOK_B".to_string());

    let admin_manifest = ManifestBuilder::new()
        .new_token_fixed(token_a_info, dec!("10000000"))
        .new_token_fixed(token_b_info, dec!("10000000"))
        .new_badge_fixed(BTreeMap::new(), Decimal::one())
        .call_method(admin_component, "deposit_batch", manifest_args!(ManifestExpression::EntireWorktop))
        .build();

    let admin_manifest_receipt = test_runner.execute_manifest(&mut test_runner, admin_manifest);

    // 
    // Publish package
    let package_address = test_runner.compile_and_publish(this_package!());

    // Test the `instantiate_pool` function.
    // let manifest = ManifestBuilder::new()
    //     .call_function(
    //         package_address,
    //         "Pool",
    //         "instantiate_pool",
    //         manifest_args![token_a_bucket,token_b_bucket,fee_to_pool]
    //     )
    //     .build();
    // let receipt = test_runner.execute_manifest_ignoring_fee(
    //     manifest,
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
    // );
    // println!("{:?}\n", receipt);
    // let component = receipt.expect_commit(true).new_component_addresses()[0];

    // // Test the `free_token` method.
    // let manifest = ManifestBuilder::new()
    //     .call_method(component, "free_token", manifest_args!())
    //     .call_method(
    //         account_component,
    //         "deposit_batch",
    //         manifest_args!(ManifestExpression::EntireWorktop),
    //     )
    //     .build();
    // let receipt = test_runner.execute_manifest_ignoring_fee(
    //     manifest,
    //     vec![NonFungibleGlobalId::from_public_key(&public_key)],
    // );
    // println!("{:?}\n", receipt);
    // receipt.expect_commit_success();
}
