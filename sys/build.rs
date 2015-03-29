#![feature(path_ext)]
#[cfg(not(windows))]
extern crate pkg_config;
use std::fs::{self, PathExt};
use std::env;
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
static FINAL_LIB:&'static str = "libjit.dll";

#[cfg(not(windows))]
static FINAL_LIB:&'static str = "libjit.a";

static MINGW:&'static str = "c:/mingw";

static INSTALL_AUTOTOOLS_MSG:&'static str = "Failed to generate configuration script. Did you forget to install autotools, bison, flex, and libtool?";

static USE_CARGO_MSG:&'static str = "Build script should be ran with Cargo";

#[cfg(windows)]
static INSTALL_COMPILER_MSG:&'static str = "Failed to configure the library for your platform. Did you forget to install MinGW and MSYS?";
#[cfg(not(windows))]
static INSTALL_COMPILER_MSG:&'static str = "Failed to configure the library for your platform. Did you forget to install gcc or clang?";

fn parse(cmd: &str) -> Command {
	let mut words = cmd.split(' ');
	let mut command = Command::new(words.next().unwrap());
	for arg in words {
		command.arg(arg);
	}
	command
}

fn main() {
	if cfg!(windows) && !Path::new(MINGW).exists() {
		panic!("{}", INSTALL_COMPILER_MSG);
	} else if pkg_config::find_library("jit").is_ok() {
		println!("Copy of LibJIT found on system - no need to build");
		return;
	}
	let out_dir = env::var("OUT_DIR").ok().expect(USE_CARGO_MSG);
	let out_dir = Path::new(&*out_dir);
	let submod_path = Path::new("libjit");
	let final_lib_dir = submod_path.join("jit/.libs");
	if !final_lib_dir.join(FINAL_LIB).exists() {
		run_wocare(&mut parse("git submodule init"));
		run(&mut parse("git submodule update"));
		if !submod_path.exists() {
			run(Command::new("git").args(&["clone", "git://git.savannah.gnu.org/libjit.git", submod_path.to_str().unwrap()]))
		}
		run_nice(parse("sh auto_gen.sh").current_dir(submod_path), INSTALL_AUTOTOOLS_MSG);
		run_nice(parse("sh configure --enable-static --disable-shared CC=clang CFLAGS=-fPIC").current_dir(submod_path), INSTALL_COMPILER_MSG);
		run(Command::new("make").arg("j8").current_dir(submod_path));
	}
	let from = final_lib_dir.join(FINAL_LIB);
	let to = out_dir.join(FINAL_LIB);
	if let Err(error) = fs::copy(&from, &to) {
		panic!("Failed to copy library from {:?} to {:?} due to {}", from, to, error)
	}
	println!("cargo:rustc-link-lib=jit");
	println!("cargo:rustc-link-search=native={:?}", out_dir);
}
fn run_nice(cmd: &mut Command, text: &str) {
	if !cmd.status().unwrap().success() {
		panic!("{:?} failed - {}", cmd, text)
	}
}
fn run(cmd: &mut Command) {
	if !cmd.status().unwrap().success() {
		panic!("{:?} failed", cmd)
	}
}
fn run_wocare(cmd: &mut Command) {
	cmd.status().unwrap();
}
