#![feature(if_let)]
extern crate "pkg-config" as pkg_config;
use std::io;
use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::process::Command;
use std::path::Path;
use std::os;

fn main() {
	io::println("Searching for LibJIT");
	match pkg_config::find_library("jit") {
		Ok(()) => return,
		Err(..) => {}
	}
	let ref out_dir = Path::new(os::getenv("OUT_DIR").expect("OUT_DIR needed for compilation"));
	let ref final_lib_dir = Path::new("native/jit/.libs");
	let ref submod_path = Path::new("native");
	if !final_lib_dir.join("libjit.a").exists() {
		if !submod_path.exists() {
			run(Command::new("git").arg("submodule").arg("init"));
		}
		run(Command::new("git").arg("submodule").arg("update"));
		run(Command::new("sh").arg("auto_gen.sh").cwd(submod_path));
		run(Command::new("sh").arg("configure").arg("--enable-shared").cwd(submod_path));
		run(Command::new("make").cwd(submod_path));
	}
    println!("cargo:root={}", out_dir.display());
    println!("cargo:rustc-flags=-l jit:static");
	println!("cargo:rustc-flags=-L {}", final_lib_dir.display());
}
fn run(command: &mut Command) {
	println!("{}", command);
	match command.output() {
		Ok(ref process) if !process.status.success() => {
			panic!("failed with output: \n{}\n{}", String::from_utf8_lossy(process.output[]), String::from_utf8_lossy(process.error[]));
		},
		Ok(_) => (),
		Err(err) => panic!("failed due to {}", err)
	}
}