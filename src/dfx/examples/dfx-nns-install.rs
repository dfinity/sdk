use std::process::Command;

fn main() {
   println!("Hello");
   let mut dfx_server = Command::new("dfx").arg("start").arg("--clean").spawn().expect("Could not start dfx server");
   let installation = Command::new("dfx").arg("nns").arg("install").output().expect("Could not start dfx to install NNS");
   assert!(installation.status.success(), "NNS installation failed.");
   dfx_server.kill().expect("The dfx server died prematurely.  This should not happen.");
}
