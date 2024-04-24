use zecwalletlitelib::lightclient::config::LightClientConfig;
use zecwalletlitelib::MainNetwork;

/// This function is only tested against Linux.
pub fn report_permission_error() {
    let user = std::env::var("USER").expect("Unexpected error reading value of $USER!");
    let home = std::env::var("HOME").expect("Unexpected error reading value of $HOME!");
    let current_executable = std::env::current_exe().expect("Unexpected error reporting executable path!");
    eprintln!("USER: {}", user);
    eprintln!("HOME: {}", home);
    eprintln!("Executable: {}", current_executable.display());
    if home == "/" {
        eprintln!("User {} must have permission to write to '{}.zcash/' .", user, home);
    } else {
        eprintln!("User {} must have permission to write to '{}/.zcash/' .", user, home);
    }
}

pub fn attempt_recover_seed(_password: Option<String>) {
    // Create a Light Client Config in an attempt to recover the file.
    let _config = LightClientConfig::<MainNetwork> {
        server: "0.0.0.0:0".parse().unwrap(),
        chain_name: "main".to_string(),
        sapling_activation_height: 0,
        anchor_offset: 0,
        monitor_mempool: false,
        data_dir: None,
        params: MainNetwork,
    };
}
