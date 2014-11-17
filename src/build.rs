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
			io::println("Configuring LibJIT");
			run(Command::new("sh").arg("auto_gen.sh").cwd(submod_path));
			run(Command::new("./configure").cwd(submod_path));

			io::println("Building LibJIT");
			run(Command::new("make").cwd(submod_path));
		}
		fs::copy(eventual_lib_path, dest_path).unwrap();
	}
	println!("cargo:rustc-flags=-L {}", out_path.display());
    println!("cargo:root={}", out_path.display());
    println!("cargo:rustc-flags=-l jit:static");
}
fn run(command: &mut Command) {
	let process = command.output().unwrap();
	if !process.status.success() {
		panic!("Command '{}' failed with output: \n{}\n{}", command, String::from_utf8_lossy(process.output[]), String::from_utf8_lossy(process.error[]));
	}
}