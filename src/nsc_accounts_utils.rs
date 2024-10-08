use std::process::Command;

pub fn get_creds_path(creds_base_path: &str, operator_name: &str, account_name: &str, username: &str) -> String {
    format!("{}/{}/{}/{}.creds", creds_base_path, operator_name, account_name, username)
}

pub fn check_if_creds_exists(creds_base_path: &str, operator_name: &str, account_name: &str, username: &str) -> Result<String, String> {
    let path_str = get_creds_path(creds_base_path, operator_name, account_name, username);
    let path = std::path::Path::new(&path_str);
    if !path.exists() {
        return Err(format!("Credentials file not found: {}", path_str));
    }
    Ok(path_str)
}

pub fn create_nsc_account(account_name: &str) -> Result<String, String> {

    // Create the NATS account
    let account_output = Command::new("nsc")
        .arg("add")
        .arg("account")
        .arg("--name")
        .arg(account_name)
        .output()
        .map_err(|e| format!("Failed to create NATS account: {}", e))?;

    if !account_output.status.success() {
        let stderr = String::from_utf8_lossy(&account_output.stderr);
        return Err(format!("Failed to create NATS account: {}", stderr));
    }

    // Retrieve account id

    let account_id_output = Command::new("nsc")
        .arg("describe")
        .arg("account")
        .arg(account_name)
        .arg("--field")
        .arg("sub")
        .output()
        .map_err(|e| format!("Failed to describe account: {}", e))?;

    if !account_id_output.status.success() {
        let stderr = String::from_utf8_lossy(&account_id_output.stderr);
        return Err(format!("Failed to get account id: {}", stderr));
    }

    let account_id = String::from_utf8_lossy(&account_id_output.stdout)
        .trim()
        .trim_matches('"')
        .to_string();

    Ok(account_id)
}

pub fn delete_nsc_account(account_name: &str) -> Result<bool, String> {

    let output = Command::new("nsc")
        .arg("delete")
        .arg("account")
        .arg(account_name)
        .output()
        .map_err(|e| format!("Failed to delete account: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to delete account: {}", stderr));
    }
    Ok(true)

}

pub fn get_account_jwt(account_name: &str) -> Result<String, String> {
    // nsc describe account <account_name> -- raw
    let output = Command::new("nsc")
        .arg("describe")
        .arg("account")
        .arg(account_name)
        .arg("--raw")
        .output()
        .map_err(|e| format!("Failed to get account jwt: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to get account jwt: {}", stderr));
    }

    let jwt = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(jwt)

}

pub fn create_nsc_user(account_name: &str, username: &str) -> Result<bool, String> {

    // Create the user
    let output = Command::new("nsc")
        .arg("add")
        .arg("user")
        .arg("--name")
        .arg(&username)
        .arg("--account")
        .arg(&account_name)
        .output()
        .map_err(|e| format!("Failed to create user: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create user: {}", stderr));
    }

    Ok(true)
}

pub fn delete_nsc_user(account_name: &str, username: &str) -> Result<bool, String> {

    let output = Command::new("nsc")
        .arg("delete")
        .arg("user")
        .arg("--name")
        .arg(username)
        .arg("--account")
        .arg(&account_name)
        .output()
        .map_err(|e| format!("Failed to delete user: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to delete user: {}", stderr));
    }
    Ok(true)
}