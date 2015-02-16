#![feature(libc, std_misc, env, path, fs)]
#[cfg(not(windows))]
extern crate "pkg-config" as pkg_config;
extern crate libc;
use std::ffi::CString;
use std::fs::{self, PathExt};
use std::env;
use std::path::Path;

#[cfg(windows)]
static FINAL_LIB:&'static str = "libjit.dll";

#[cfg(not(windows))]
static FINAL_LIB:&'static str = "libjit.a";

static MINGW:&'static str = "c:/mingw";

static INSTALL_AUTOTOOLS_MSG:&'static str = "Failed to generate configuration script. Did you forget to install autotools, bison, flex, and libtool?";
#[cfg(windows)]
static INSTALL_COMPILER_MSG:&'static str = "Failed to configure the library for your platform. Did you forget to install MinGW and MSYS?";
#[cfg(not(windows))]
static INSTALL_COMPILER_MSG:&'static str = "Failed to configure the library for your platform. Did you forget to install gcc?";

fn main() {
	if cfg!(windows) && !Path::new(MINGW).exists() {
		panic!("{}", INSTALL_COMPILER_MSG);
	} else if pkg_config::find_library("jit").is_ok() {
		return;
	}
	let out_dir = env::var("OUT_DIR").unwrap();
	let out_dir = Path::new(&*out_dir);
	let submod_path = Path::new("libjit");
	let final_lib_dir = submod_path.join("jit/.libs");
	if !final_lib_dir.join(FINAL_LIB).exists() {
		run_wocare("git submodule init");
		run("git submodule update");
		if !submod_path.exists() {
			let text = format!("git clone git://git.savannah.gnu.org/libjit.git {}", submod_path.display());
			run(&*text);
		}
		chdir(submod_path);
		run_nice("sh auto_gen.sh", INSTALL_AUTOTOOLS_MSG);
		run_nice("sh configure --enable-static --disable-shared CC=clang CFLAGS=-fPIC", INSTALL_COMPILER_MSG);
		run("make");
	}
	let from = final_lib_dir.join(FINAL_LIB);
	let to = out_dir.join(FINAL_LIB);
	fs::copy(&from, &to).unwrap();
    println!("cargo:rustc-flags=-l jit:static -L {}", out_dir.display());
}
fn chdir(path: &Path) {
	use libc::chdir;
	use std::str::from_utf8_unchecked;
	unsafe {
		let c_path = CString::from_slice(path.to_str().unwrap().as_bytes());
		if libc::chdir(c_path.as_ptr()) == -1 {
			panic!("Failed to change directory into {}", from_utf8_unchecked(c_path.as_bytes()))
		}
	}
}
fn run_nice(cmd: &str, text: &str) {
	unsafe {
		let c_cmd = CString::from_slice(cmd.as_bytes());
		if libc::system(c_cmd.as_ptr()) != 0 {
			panic!("{}", text);
		}
	}
}
fn run(cmd: &str) {
	unsafe {
		let c_cmd = CString::from_slice(cmd.as_bytes());
		if libc::system(c_cmd.as_ptr()) != 0 {
			panic!("{} failed", cmd);
		}
	}
}
fn run_wocare(cmd: &str) {
	unsafe {
		let c_cmd = CString::from_slice(cmd.as_bytes());
		if libc::system(c_cmd.as_ptr()) < 0 {
			panic!("{} failed", cmd);
		}
	}
}