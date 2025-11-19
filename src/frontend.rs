
fn main() {
    let _ = env_logger::builder().try_init();

    print_big_banner_puetce();
}

/// Prints the PUETCE banner
/// 
/// - it is pronounced PWAYCHAY
fn print_big_banner_puetce() {
    // developers note: this doesnt mean anything, github copilot just hallucinated it and i thought it looked cool
    println!(r#"
██████╗ ██╗   ██╗███████╗████████╗ ██████╗███████╗
██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔════╝██╔════╝
██████╔╝██║   ██║█████╗     ██║   ██║     █████╗
██╔═══╝ ██║   ██║██╔══╝     ██║   ██║     ██╔══╝
██║     ╚██████╔╝███████╗   ██║   ╚██████╗███████╗
╚═╝      ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝╚══════╝
"#);
}