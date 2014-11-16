#![feature(if_let)]
use std::io;
use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::process::{ProcessExit, Command};
use std::path::Path;
use std::os;

fn main() {

	io::println("Searching for LibJIT");
	let ref out_path = Path::new(os::getenv("OUT_DIR").unwrap());
	let ref lib_path = Path::new("/usr/lib/libjit.so");
	let ref eventual_lib_path = Path::new("native/jit/.libs/libjit.so");
	let ref dest_path = out_path.join("libjit.so");
	let ref submod_path = Path::new("native");
	if lib_path.exists() {
		fs::copy(lib_path, dest_path).unwrap();
	} else if !dest_path.exists() {
		if !eventual_lib_path.exists() {
			io::println("No LibJIT found in /usr/lib/ so updating LibJIT");
			if !submod_path.exists() {
				io::println("Initialising submodule");
				run(Command::new("git").arg("submodule").arg("init"));
			}
			run(Command::new("git").arg("submodule").arg("update"));
			let old_cwd = os::getcwd();
			os::change_dir(submod_path);
			io::println("Configuring LibJIT");
			run(Command::new("sh").arg("auto_gen.sh"));
			run_cmd("./configure");

			io::println("Building LibJIT");
			run_cmd("make");
			os::change_dir(&old_cwd);
		}
		fs::copy(eventual_lib_path, dest_path).unwrap();
	}
	println!("cargo:rustc-flags=-L {}", out_path.display());
}
fn run_cmd(command: &str) {
	run(&mut Command::new(command))
}
fn run(command: &mut Command) {
	let process = command.output().unwrap();
	if !process.status.success() {
		panic!("Command '{}' failed with output: \n{}\n{}", command, String::from_utf8_lossy(process.output[]), String::from_utf8_lossy(process.error[]));
	}
}