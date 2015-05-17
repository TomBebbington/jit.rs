use std::io;
use std::fs;
use std::env;
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
static FINAL_LIB:&'static str = "libjit.dll";

#[cfg(not(windows))]
static FINAL_LIB:&'static str = "libjit.a";

static MINGW:&'static str = "c:/mingw";

static INSTALL_AUTOTOOLS_MSG:&'static str = "Failed to generate configuration script. Did you forget to install autotools, bison, flex, and libtool?";

static USE_CARGO_MSG:&'static str = "Build script should be ran with Cargo, run `cargo build` instead";

#[cfg(windows)]
static INSTALL_COMPILER_MSG:&'static str = "Failed to configure the library for your platform. Did you forget to install MinGW and MSYS?";
#[cfg(not(windows))]
static INSTALL_COMPILER_MSG:&'static str = "Failed to configure the library for your platform. Did you forget to install a C compiler?";

// PathExt::exists isn't stable, so fake it by querying file metadata.
fn exists<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err)
    }
}

fn main() {
	if cfg!(windows) && !exists(&Path::new(MINGW)).unwrap() {
		panic!("{}", INSTALL_COMPILER_MSG);
	}
	let out_dir = env::var("OUT_DIR").ok().expect(USE_CARGO_MSG);
	let num_jobs = env::var("NUM_JOBS").ok().expect(USE_CARGO_MSG);
	let target = env::var("TARGET").ok().expect(USE_CARGO_MSG);
	let out_dir = Path::new(&*out_dir);
    let submod_path = Path::new(&env::var("CARGO_MANIFEST_DIR").ok().expect(USE_CARGO_MSG)).join("libjit");
	let final_lib_dir = submod_path.join("jit/.libs");
	if !exists(&final_lib_dir.join(FINAL_LIB)).unwrap() {
		Command::new("git")
			.args(&["submodule", "init"])
			.status().unwrap();
		run(Command::new("git")
			.args(&["submodule", "update"]),
			None
		);
		if !exists(&submod_path).unwrap() {
			run(Command::new("git")
				.args(&["clone", "git://git.savannah.gnu.org/libjit.git", submod_path.to_str().unwrap()]),
				None
			)
		}
		run(
			Command::new("sh")
				.current_dir(&submod_path)
				.arg("auto_gen.sh"),
			Some(INSTALL_AUTOTOOLS_MSG)
		);
		run(
			Command::new("sh")
				.current_dir(&submod_path)
				.env("CFLAGS", "-fPIC")
				.args(&[
					"configure", "--enable-static", "--disable-shared", &format!("--host={}", target)
				]),
			Some(INSTALL_COMPILER_MSG)
		);
		run(Command::new("make")
			.arg(&format!("-j{}", num_jobs))
			.current_dir(submod_path),
			None
		);
	} else {
		println!("LibJIT has already been built")
	}
	let from = final_lib_dir.join(FINAL_LIB);
	let to = out_dir.join(FINAL_LIB);
	if let Err(error) = fs::copy(&from, &to) {
		panic!("Failed to copy library from {:?} to {:?} due to {}", from, to, error)
	}
	println!("cargo:rustc-link-search=native={}",
                 out_dir.to_str().expect("non-unicode characters in path"));
	println!("cargo:rustc-link-lib=static=jit");
}
fn run(cmd: &mut Command, text: Option<&str>) {
	if !cmd.status().unwrap().success() {
		let text = text.map(|text| format!(" - {}", text)).unwrap_or(String::new());
		panic!("{:?} failed{}", cmd, text)
	}
}
