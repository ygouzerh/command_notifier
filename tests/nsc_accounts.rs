use command_notifier::nsc_accounts_utils::{
    create_nsc_account,
    create_nsc_user,
    delete_nsc_account,
    delete_nsc_user,
    check_if_creds_exists,
    get_creds_path
};
use std::panic;
use std::process::Command;

#[test]
fn test_creds_path() {
    let creds_base_path = "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds";
    let operator_name = "ServerBackend";
    let account_name = "test_account";
    let username = "test_user_01";

    let creds_path = get_creds_path(creds_base_path, operator_name, account_name, username);

    assert_eq!(creds_path, "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds/ServerBackend/test_account/test_user_01.creds", "Creds path is incorrect");

    println!("Creds path: {}", creds_path);

}

#[test]
fn test_check_if_creds_exists_fail() {
    let creds_base_path = "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds";
    let operator_name = "ServerBackend";
    let account_name = "test_account";
    let username = "djqwdjqwdjqwlkdjql2312djqwd";

    let result = check_if_creds_exists(creds_base_path, operator_name, account_name, username);

    assert!(result.is_err(), "Should fail to get creds path");
}

#[test]
fn test_check_if_creds_exists_ok() {
    let account_name = "test_account";
    let username = "test_user_01";

    let result = panic::catch_unwind(|| {
        let creds_base_path = "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds";
        let operator_name = "ServerBackend";

        create_nsc_account(account_name).unwrap();

        create_nsc_user(account_name, username).unwrap();

        let result = check_if_creds_exists(creds_base_path, operator_name, account_name, username);

        assert!(result.is_ok(), "Failed to get creds path: {:?}", result);

        let creds_path = result.unwrap();

        assert!(!creds_path.is_empty(), "Creds path should not be empty");

        assert_eq!(creds_path, "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds/ServerBackend/test_account/test_user_01.creds", "Creds path is incorrect");

        println!("Creds path: {}", creds_path);
    });

    cleanup_nsc_user(account_name, username);
    cleanup_nsc_account(account_name);

    assert!(result.is_ok());
}


#[test]
fn test_create_nsc_account() {
    let result = panic::catch_unwind(|| {
        let account_name = "test_account";
        // let creds_path = "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds/ServerBackend";
        let result = create_nsc_account(account_name);

        assert!(result.is_ok(), "Failed to create NATS account: {:?}", result);

        let account_id = result.unwrap();

        assert!(!account_id.is_empty(), "Account ID should not be empty");

        println!("Account ID: {}", account_id);

    });

    // Perform cleanup regardless of the test result
    cleanup_nsc_account("test_account");

    // Propagate the panic, if any
    assert!(result.is_ok());
}

#[test]
fn test_create_nsc_user() {
    let account_name = "test_account";
    let username = "test_user_01";

    let result = panic::catch_unwind(|| {

        let result = create_nsc_account(account_name);

        assert!(result.is_ok(), "Failed to create NATS account");

        let result = create_nsc_user(account_name, username);

        assert!(result.is_ok(), "Failed to create NATS user");
    });

    cleanup_nsc_user(account_name, username);
    cleanup_nsc_account(account_name);

    assert!(result.is_ok());
}

#[test]
fn test_delete_nsc_account() {
    let account_name = "test_account";
    let result = panic::catch_unwind(|| {
        let result = create_nsc_account(account_name);

        assert!(result.is_ok(), "Failed to create NATS account");

        let result = delete_nsc_account(account_name);

        assert!(result.is_ok(), "Failed to delete NATS account");

        let account_output = Command::new("nsc")
            .arg("describe")
            .arg("account")
            .arg(account_name)
            .output();

        // Assert that the output contains "not in accounts"
        let account_output = account_output.unwrap();
        let account_output = String::from_utf8_lossy(&account_output.stderr);
        assert!(account_output.contains("not in accounts"), "Account should not exist");

    });

    cleanup_nsc_account(account_name);

    assert!(result.is_ok());
}

#[test]
fn test_delete_nsc_user() {
    let account_name = "test_account";
    let username = "test_user_01";

    let result = panic::catch_unwind(|| {
        let result = create_nsc_account(account_name);

        assert!(result.is_ok(), "Failed to create NATS account");

        let result = create_nsc_user(account_name, username);

        assert!(result.is_ok(), "Failed to create NATS user");

        let result = delete_nsc_user(username);

        assert!(result.is_ok(), "Failed to delete NATS user");

        let user_output = Command::new("nsc")
            .arg("describe")
            .arg("user")
            .arg("--account")
            .arg(account_name)
            .arg(username)
            .output();

        // Assert that the output contains "not in users"
        let user_output = user_output.unwrap();
        let user_output = String::from_utf8_lossy(&user_output.stderr);
        assert!(user_output.contains("does not exist"), "User should not exist");
    });

    cleanup_nsc_user(account_name, username);

    cleanup_nsc_account(account_name);

    assert!(result.is_ok());
}


#[cfg(test)]
fn cleanup_nsc_account(account_name: &str) {
    // Clean up the generated account and user
    let _ = Command::new("nsc")
        .arg("delete")
        .arg("account")
        .arg(account_name)
        .output();
}

#[cfg(test)]
fn cleanup_nsc_user(account_name: &str, username: &str) {
    // Clean up the generated account and user
    let _ = Command::new("nsc")
        .arg("delete")
        .arg("user")
        .arg("--account")
        .arg(account_name)
        .arg(username)
        .output();
}