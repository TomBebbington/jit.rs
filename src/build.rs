#![feature(if_let)]
extern crate "pkg-config" as pkg_config;
use std::io;
use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::process::{ProcessExit, Command};
use std::path::Path;
use std::os;

fn main() {
	io::println("Searching for LibJIT");
	match pkg_config::find_library("jit") {
		Ok(()) => return,
		Err(..) => {}
	}
	let ref out_dir = Path::new(os::getenv("OUT_DIR").expect("OUT_DIR needed for compilation"));
	let ref final_lib_path = Path::new("native/jit/.libs/libjit.so");
	let ref dest_path = out_dir.join("libjit.so");
	let ref submod_path = Path::new("native");
	if !dest_path.exists() {
		if !final_lib_path.exists() {
			if !submod_path.exists() {
				run(Command::new("git").arg("submodule").arg("init"));
			}
			run(Command::new("git").arg("submodule").arg("update"));
			run(Command::new("sh").arg("auto_gen.sh").cwd(submod_path));
			run(Command::new("./configure").cwd(submod_path));
			run(Command::new("make").cwd(submod_path));
		}
		fs::copy(final_lib_path, dest_path).unwrap();
	}
    println!("cargo:root={}", out_dir.display());
    println!("cargo:rustc-flags=-l jit:static");
	println!("cargo:rustc-flags=-L {}", out_dir.display());
}
fn run(command: &mut Command) {
	let process = command.output().unwrap();
	println!("{}", command);
	if !process.status.success() {
		panic!("failed with output: \n{}\n{}", String::from_utf8_lossy(process.output[]), String::from_utf8_lossy(process.error[]));
	}
}