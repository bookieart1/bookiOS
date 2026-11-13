use std::env;
use std::env::current_dir;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

static INTEGRATION_ENVIRONMENT_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq, Eq)]
enum StubBinaryPath<'a> {
    Framework {
        name: &'a str,
        directory: &'a str,
    },
    Dylib {
        file_name: &'a str,
        link_name: &'a str,
    },
}

fn classify_stub_binary_path(dylib_path: &str) -> Result<StubBinaryPath<'_>, String> {
    if !dylib_path.starts_with('/') {
        return Err("path is not absolute".to_string());
    }

    let (directory, binary_name) = dylib_path
        .rsplit_once('/')
        .ok_or_else(|| "path has no binary component".to_string())?;
    if binary_name.is_empty() {
        return Err("path has an empty binary component".to_string());
    }

    let directory_name = directory
        .rsplit_once('/')
        .map(|(_, name)| name)
        .unwrap_or(directory);
    if let Some(framework_name) = directory_name.strip_suffix(".framework") {
        if framework_name.is_empty() {
            return Err("framework directory has an empty name".to_string());
        }
        if binary_name != framework_name {
            return Err(format!(
                "framework binary {binary_name:?} does not match framework directory {directory_name:?}"
            ));
        }
        return Ok(StubBinaryPath::Framework {
            name: framework_name,
            directory,
        });
    }

    let link_name = binary_name
        .strip_prefix("lib")
        .and_then(|name| name.strip_suffix(".dylib"))
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            "path is neither a Name.framework/Name binary nor a libName.dylib".to_string()
        })?;
    Ok(StubBinaryPath::Dylib {
        file_name: binary_name,
        link_name,
    })
}

#[test]
fn classify_stub_binary_path_table() {
    let cases = [
        (
            "/System/Library/Frameworks/Foundation.framework/Foundation",
            Ok(StubBinaryPath::Framework {
                name: "Foundation",
                directory: "/System/Library/Frameworks/Foundation.framework",
            }),
        ),
        (
            "/System/Library/PrivateFrameworks/GraphicsServices.framework/GraphicsServices",
            Ok(StubBinaryPath::Framework {
                name: "GraphicsServices",
                directory: "/System/Library/PrivateFrameworks/GraphicsServices.framework",
            }),
        ),
        (
            "/usr/lib/libSystem.B.dylib",
            Ok(StubBinaryPath::Dylib {
                file_name: "libSystem.B.dylib",
                link_name: "System.B",
            }),
        ),
        (
            "/System/Library/PrivateFrameworks/Broken.framework/NotBroken",
            Err("does not match framework directory"),
        ),
    ];
    for (path, expected) in cases {
        match (classify_stub_binary_path(path), expected) {
            (Ok(actual), Ok(expected)) => assert_eq!(actual, expected),
            (Err(actual), Err(expected_fragment)) => assert!(
                actual.contains(expected_fragment),
                "unexpected error for {path:?}: {actual}"
            ),
            (actual, expected) => {
                panic!("unexpected classification for {path:?}: {actual:?}, expected {expected:?}")
            }
        }
    }
}

// adapted from `assert_cmd` crate
fn target_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .unwrap()
}

// https://stackoverflow.com/a/35907071/2241008
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Makes a path object, and checks that it exists.
fn make_path_and_check(
    tests_dir: &Path,
    path: &[&str],
    object: &str,
    is_executable: bool,
) -> PathBuf {
    let mut buf = tests_dir.to_path_buf();
    if is_executable {
        for part in &path[..(path.len() - 1)] {
            buf.push(part);
        }
        buf.push(format!(
            "{}{}",
            path.last().unwrap(),
            env::consts::EXE_SUFFIX
        ));
        println!("{}", buf.iter().last().unwrap().display())
    } else {
        for part in path {
            buf.push(part);
        }
    }

    if !buf.exists() {
        panic!(
            "Couldn't find {} at {}. Please see {} for more details.",
            object,
            buf.display(),
            tests_dir.join("README.md").display()
        );
    }
    buf
}

fn build_object<I: Iterator<Item = P>, P: AsRef<OsStr>>(
    tests_dir: &Path,
    output_name: &Path,
    sources: I,
    extra_compile_args: &[&str],
) -> Result<(), Box<dyn Error>> {
    let clang_path = make_path_and_check(tests_dir, &["llvm", "bin", "clang"], "Clang", true);

    let bin_path = make_path_and_check(
        tests_dir,
        &["common-3.0.sdk", "usr", "bin"],
        "binary directory",
        false,
    );

    let sdk_path = make_path_and_check(tests_dir, &["common-3.0.sdk"], "SDK sysroot", false);

    let mut linker_arg = OsString::from("-B");
    linker_arg.push(bin_path);
    let mut sdk_arg = OsString::from("--sysroot=");
    sdk_arg.push(sdk_path);

    eprintln!("Building {} for iPhone OS 3...", output_name.display());
    std::io::stderr().flush().unwrap();
    let mut cmd = Command::new(clang_path);
    let output = cmd
        // Uncomment for verbose output (useful for debugging search path
        // issues)
        // .arg("-v")
        // Uncomment for verbose linker output
        // .arg("-Wl,-v")
        // Target iPhone OS 2
        .arg("--target=arm-apple-ios")
        .arg("-miphoneos-version-min=2.0")
        .args(["-arch", "armv6", "-arch", "armv7"])
        // If enabled, the stack protection causes a null pointer crash in some
        // functions. This is probably because ___stack_chk_guard isn't linked.
        .arg("-fno-stack-protector")
        .arg("-DPRODUCT_iPhone")
        // This prevents clang from attempting to link clang_rt on macOS.
        .arg("-nodefaultlibs")
        .arg(linker_arg)
        .arg(sdk_arg)
        .args(extra_compile_args)
        // Input files.
        .args(sources)
        // Write the output to the bundle.
        .arg("-o")
        .arg(output_name)
        .output()
        .expect("failed to execute Clang process");
    eprintln!("Running {:?}", cmd);
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
    eprintln!("Built successfully.");
    Ok(())
}

// Note that source files are looked for in the path
// "{tests_dir}/{test_app_name}_source"
// and binaries are output as
// "{tests_dir}/{test_app_name}.app/{test_app_name}".
fn run_test_app(
    tests_dir: &Path,
    test_app_name: &str,
    extra_compile_args: &[&str],
    extra_run_args: &[&str],
) -> Result<(), Box<dyn Error>> {
    let test_app_path = tests_dir.join(format!("{}.app", test_app_name));

    let source_path = tests_dir.join(format!("{}_source", test_app_name));
    let all_sources: Vec<PathBuf> = std::fs::read_dir(&source_path)
        .unwrap()
        .map(|entry| PathBuf::from(entry.unwrap().file_name()))
        .filter(|filename| {
            filename
                .extension()
                .is_some_and(|ext| ext == "m" || ext == "c" || ext == "cpp")
        })
        .map(|entry| source_path.join(entry))
        .collect();

    // C++ files must be compiled separately because the -ObjC flag (needed
    // for Objective-C linking) forces all files to be treated as Objective-C.
    let mut cpp_objects = Vec::new();
    let (cpp_sources, sources): (Vec<_>, Vec<_>) = all_sources
        .into_iter()
        .partition(|p| p.extension().is_some_and(|ext| ext == "cpp"));
    for cpp_src in &cpp_sources {
        let obj_path = tests_dir
            .join(format!("{}.app", test_app_name))
            .join(cpp_src.file_stem().unwrap())
            .with_extension("o");
        // Compile C++ without -ObjC, -fno-objc-exceptions, etc.
        let mut cpp_args: Vec<&str> = extra_compile_args
            .iter()
            .copied()
            .filter(|a| {
                *a != "-ObjC"
                    && *a != "-fno-objc-exceptions"
                    && *a != "-fno-objc-arc"
                    && *a
                        != "-fno-objc-arc-e
          +xceptions"
            })
            .collect();
        cpp_args.push("-c");
        build_object(
            tests_dir,
            &obj_path,
            [cpp_src.as_path()].into_iter(),
            &cpp_args,
        )?;
        cpp_objects.push(obj_path);
    }

    build_object(
        &tests_dir,
        &tests_dir
            .join(format!("{}.app", test_app_name))
            .join(test_app_name),
        sources.iter().chain(cpp_objects.iter()),
        extra_compile_args,
    )?;
    let binary_name = "touchHLE";
    let binary_path = target_dir().join(format!("{}{}", binary_name, env::consts::EXE_SUFFIX));
    let mut cmd = Command::new(binary_path);
    let output = cmd
        .arg(test_app_path)
        // headless mode avoids a distracting window briefly appearing during
        // testing, and works in CI.
        .arg("--headless")
        .args(extra_run_args)
        // Run the automated CLI tests, rather than the manual UIKit tests.
        .arg("--args")
        .arg("--cli-tests")
        .output()
        .expect("failed to execute touchHLE process");
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());
    // sanity check: check that emulation actually happened
    assert_ne!(
        find_subsequence(output.stderr.as_slice(), b"CPU emulation begins now."),
        None
    );
    write!(
        &mut std::io::stdout(),
        "Finished running {}.\n\n\n",
        test_app_name
    )
    .unwrap();
    Ok(())
}

fn run_trenches_selector_mutation_app(
    tests_dir: &Path,
    extra_compile_args: &[&str],
) -> Result<(), Box<dyn Error>> {
    const APP_NAME: &str = "TrenchesSelectorMutation";
    const CASES: [(&str, &str); 3] = [
        (
            "control",
            "TRENCHESTEST case=control callbacks=ABCD membership=ABCD returned=1 overflow=0 result=PASS",
        ),
        (
            "self-remove",
            "TRENCHESTEST case=self-remove callbacks=ABCD membership=AC returned=1 overflow=0 result=PASS",
        ),
        (
            "remove-later",
            "TRENCHESTEST case=remove-later callbacks=ABCD membership=ABC returned=1 overflow=0 result=PASS",
        ),
    ];

    let source = tests_dir.join(format!("{}_source/main.m", APP_NAME));
    let header_root = tests_dir.join(format!("{}_headers", APP_NAME));
    let header_search_arg = format!("-I{}", header_root.display());
    let mut fixture_compile_args = Vec::with_capacity(extra_compile_args.len() + 1);
    fixture_compile_args.push(header_search_arg.as_str());
    fixture_compile_args.extend_from_slice(extra_compile_args);
    let app_path = tests_dir.join(format!("{}.app", APP_NAME));
    let output_path = app_path.join(APP_NAME);
    build_object(
        tests_dir,
        &output_path,
        [source.as_path()].into_iter(),
        &fixture_compile_args,
    )?;

    let binary_path = target_dir().join(format!("touchHLE{}", env::consts::EXE_SUFFIX));
    let mut results = Vec::new();
    for (case, expected) in CASES {
        let output = Command::new(&binary_path)
            .arg(&app_path)
            .arg("--headless")
            .arg("--args")
            .arg(case)
            .output()
            .expect("failed to execute touchHLE process");
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        results.push((case, expected, output));
    }

    let mut failures = Vec::new();
    for (case, expected, output) in results {
        let emulation_started =
            find_subsequence(&output.stderr, b"CPU emulation begins now.").is_some();
        let expected_marker = find_subsequence(&output.stdout, expected.as_bytes()).is_some();
        if !output.status.success() || !emulation_started || !expected_marker {
            failures.push(format!(
                "case={case} status={} emulation_started={emulation_started} expected_marker={expected_marker}",
                output.status
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "Trenches selector-mutation contract failures:\n{}",
        failures.join("\n")
    );
    Ok(())
}

fn run_ui_control_alignment_app(
    tests_dir: &Path,
    extra_compile_args: &[&str],
) -> Result<(), Box<dyn Error>> {
    const APP_NAME: &str = "UIControlAlignment";
    const EXPECTED: &str =
        "OFGREEN_ALIGNMENT ancestry=UIButton default=0 values=1,2,3 release=1 result=PASS";

    let source = tests_dir.join(format!("{}_source/main.m", APP_NAME));
    let header_root = tests_dir.join(format!("{}_headers", APP_NAME));
    let header_search_arg = format!("-I{}", header_root.display());
    let mut fixture_compile_args = Vec::with_capacity(extra_compile_args.len() + 1);
    fixture_compile_args.push(header_search_arg.as_str());
    fixture_compile_args.extend_from_slice(extra_compile_args);
    let app_path = tests_dir.join(format!("{}.app", APP_NAME));
    let output_path = app_path.join(APP_NAME);
    build_object(
        tests_dir,
        &output_path,
        [source.as_path()].into_iter(),
        &fixture_compile_args,
    )?;

    let binary_path = target_dir().join(format!("touchHLE{}", env::consts::EXE_SUFFIX));
    let output = Command::new(&binary_path)
        .arg(&app_path)
        .arg("--headless")
        .output()
        .expect("failed to execute touchHLE process");
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();

    let emulation_started =
        find_subsequence(&output.stderr, b"CPU emulation begins now.").is_some();
    let expected_marker = find_subsequence(&output.stdout, EXPECTED.as_bytes()).is_some();
    assert!(
        output.status.success() && emulation_started && expected_marker,
        "UIControl alignment contract failed: status={} emulation_started={} expected_marker={}",
        output.status,
        emulation_started,
        expected_marker
    );
    Ok(())
}

fn run_ui_button_content_edge_insets_app(
    tests_dir: &Path,
    extra_compile_args: &[&str],
) -> Result<(), Box<dyn Error>> {
    const APP_NAME: &str = "UIButtonContentEdgeInsets";
    const EXPECTED: &str =
        "CONTENT_EDGE_INSETS ancestry=UIButton owner=UIButton unrelated=UIControl default=0,0,0,0 value=1.25,-2.5,3.75,4.5 release=1 result=PASS";

    let source = tests_dir.join(format!("{}_source/main.m", APP_NAME));
    let header_root = tests_dir.join(format!("{}_headers", APP_NAME));
    let header_search_arg = format!("-I{}", header_root.display());
    let mut fixture_compile_args = Vec::with_capacity(extra_compile_args.len() + 1);
    fixture_compile_args.push(header_search_arg.as_str());
    fixture_compile_args.extend_from_slice(extra_compile_args);
    let app_path = tests_dir.join(format!("{}.app", APP_NAME));
    let output_path = app_path.join(APP_NAME);
    build_object(
        tests_dir,
        &output_path,
        [source.as_path()].into_iter(),
        &fixture_compile_args,
    )?;

    let binary_path = target_dir().join(format!("touchHLE{}", env::consts::EXE_SUFFIX));
    let output = Command::new(&binary_path)
        .arg(&app_path)
        .arg("--headless")
        .output()
        .expect("failed to execute touchHLE process");
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();

    let emulation_started =
        find_subsequence(&output.stderr, b"CPU emulation begins now.").is_some();
    let expected_marker = find_subsequence(&output.stdout, EXPECTED.as_bytes()).is_some();
    assert!(
        output.status.success() && emulation_started && expected_marker,
        "UIButton contentEdgeInsets contract failed: status={} emulation_started={} expected_marker={}",
        output.status,
        emulation_started,
        expected_marker
    );
    Ok(())
}

/// Recursively copy a directory's content to a destination skipping symlinks
fn copy_dir_all(source: PathBuf, destination: PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(&destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;

        if entry_type.is_file() {
            std::fs::copy(entry.path(), destination.join(entry.file_name()))?;
        } else if entry_type.is_dir() {
            copy_dir_all(entry.path(), destination.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn prepare_integration_environment() -> Result<(PathBuf, Vec<String>), Box<dyn Error>> {
    let tests_dir = current_dir()?.join("tests");
    let stubs_dir = tests_dir.join("stubs");
    let stubs_src_dir = stubs_dir.join("src");
    let stubs_lib_dir = stubs_dir.join("lib");
    let stubs_frameworks_dir = stubs_dir.join("Frameworks");

    // Wipe the stubs dir to ensure it is clean.
    let _ = std::fs::remove_dir_all(&stubs_dir);
    // Create the stubs dir
    std::fs::create_dir(&stubs_dir).unwrap();

    let bundled_libs_search_arg = "-L".to_owned() + current_dir()?.join("dylibs").to_str().unwrap();
    let stubs_lib_search_arg = "-L".to_owned() + stubs_lib_dir.to_str().unwrap();
    let stubs_frameworks_search_arg = "-F".to_owned() + stubs_frameworks_dir.to_str().unwrap();
    let mut extra_linker_args = Vec::<String>::new();
    let mut extra_compile_args = vec![
        "-mlinker-version=253".to_string(),
        "-Wno-expansion-to-defined".to_string(),
        "-Wno-literal-range".to_string(),
        bundled_libs_search_arg.clone(),
        stubs_lib_search_arg.clone(),
        stubs_frameworks_search_arg,
        "-ObjC".to_string(),
        "-fblocks".to_string(),
        "-fno-objc-exceptions".to_string(),
        // ARC is not available until IOS 5, so it can't be used.
        "-fno-objc-arc".to_string(),
        "-fno-objc-arc-exceptions".to_string(),
    ];

    // Generate symbol stubs.

    let symbols_path = stubs_dir.join("SYMBOLS.txt");
    let dump_file_option = format!("--dump-file={}", symbols_path.to_str().unwrap());
    let dump_run_args = ["--dump=symbols", dump_file_option.as_str(), "--headless"];
    let binary_name = "touchHLE";
    let binary_path = target_dir().join(format!("{}{}", binary_name, env::consts::EXE_SUFFIX));
    let mut cmd = Command::new(binary_path);
    let output = cmd
        .args(dump_run_args)
        .output()
        .expect("failed to execute touchHLE process");
    assert!(output.status.success());

    // Split SYMBOLS.txt into individual source files.

    std::fs::create_dir(&stubs_src_dir).unwrap();
    let mut files_to_compile = Vec::<(String, PathBuf)>::new();
    {
        let mut in_body = false;
        let mut current_file = None::<BufWriter<File>>;
        for line in BufReader::new(File::open(symbols_path).unwrap()).lines() {
            let line = line.unwrap();
            if let Some(dylib_path) = line.strip_prefix("// ") {
                // First comment after a series of non-comment lines, or first
                // comment in the file: this is the canonical name of the dylib.
                if in_body || current_file.is_none() {
                    let dylib_name = dylib_path.rsplit_once("/").unwrap().1;
                    let stub_src_path = stubs_src_dir.join(format!("{}.m", dylib_name));
                    current_file = Some(BufWriter::new(File::create(&stub_src_path).unwrap()));
                    files_to_compile.push((dylib_path.to_string(), stub_src_path));
                    in_body = false;
                }
                // Ignore the non-canonical dylib names for now.
            } else if let Some(ref mut current_file) = current_file {
                in_body = true;
                writeln!(current_file, "{}", line).unwrap();
            }
        }
        // current_file dropping out of scope here flushes it.
    }

    // Build the stub libraries and ensure TestApp will link to them.

    for (dylib_path, stub_src_path) in files_to_compile {
        if dylib_path.starts_with("/.touchHLE") {
            // skip the fake app picker library
            continue;
        }
        let compile_args = [
            "-mlinker-version=253",
            "-fno-builtin",
            "-nostdlib",
            &format!("-Wl,-install_name,{}", dylib_path),
            "-Wno-objc-root-class", // silence clang warning about inheritance
            "-Wl,-dylib",
            stubs_lib_search_arg.as_str(),
            "-lobjc.A",
        ];
        // - The only non-framework libs should be libSystem and libobjc, which
        //   we expect to appear in the list before all the frameworks, and need
        //   to be compiled first.
        // - The frameworks have bare filenames, and usually need libobjc.
        let stub_binary = classify_stub_binary_path(&dylib_path).map_err(|reason| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unsupported registered dylib path {dylib_path:?}: {reason}"),
            )
        })?;
        let (compile_args, out_path, framework_directory) = match stub_binary {
            StubBinaryPath::Framework { name, directory } => {
                extra_linker_args.push(format!("-framework"));
                extra_linker_args.push(name.to_string());
                (
                    &compile_args[..],
                    stubs_frameworks_dir
                        .join(format!("{name}.framework"))
                        .join(name),
                    Some(directory),
                )
            }
            StubBinaryPath::Dylib {
                file_name,
                link_name,
            } => {
                extra_linker_args.push(format!("-l{link_name}"));
                (
                    // skip "-Ltests/stubs/lib/" and "-lobjc.A"
                    &compile_args[..compile_args.len() - 2],
                    stubs_lib_dir.join(file_name),
                    None,
                )
            }
        };
        // Ensure that stubs/Frameworks/FooBarKit/ or stubs/lib/ exists
        std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

        // Ideally one could provide two framework search paths.
        // In reality only the first matching path is used, so the stub
        // framework needs to provide headers copied from the sdk.
        if let Some(framework_directory) = framework_directory {
            let stub_headers = out_path.parent().unwrap().join("Headers");
            let source_headers = tests_dir
                .join("common-3.0.sdk")
                .join(framework_directory.trim_start_matches('/'))
                .join("Headers");

            if source_headers.exists() {
                copy_dir_all(source_headers, stub_headers)?;
            }
        }
        build_object(&tests_dir, &out_path, [stub_src_path].iter(), &compile_args).unwrap();
    }

    // Link against libstdc++ to support C++ test sources (virtual inheritance
    // tests etc). The bundled libstdc++ dylib path is already in the search
    // path via bundled_libs_search_arg.
    extra_compile_args.push("-lstdc++.6.0.9".to_string());
    extra_compile_args.extend(extra_linker_args);

    Ok((tests_dir, extra_compile_args))
}

fn borrowed_args(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

#[test]
fn test_app() -> Result<(), Box<dyn Error>> {
    let _lock = INTEGRATION_ENVIRONMENT_LOCK.lock().unwrap();
    let (tests_dir, owned_compile_args) = prepare_integration_environment()?;
    let extra_compile_args = borrowed_args(&owned_compile_args);

    // Finally, build and run TestApp plus the bounded selector-mutation
    // fixture.
    run_test_app(&tests_dir, "TestApp", &extra_compile_args, &[])?;
    run_trenches_selector_mutation_app(&tests_dir, &extra_compile_args)
}

#[test]
fn trenches_selector_mutation_contract() -> Result<(), Box<dyn Error>> {
    let _lock = INTEGRATION_ENVIRONMENT_LOCK.lock().unwrap();
    let (tests_dir, owned_compile_args) = prepare_integration_environment()?;
    let extra_compile_args = borrowed_args(&owned_compile_args);
    run_trenches_selector_mutation_app(&tests_dir, &extra_compile_args)
}

#[test]
fn ofgreen_button_alignment_contract() -> Result<(), Box<dyn Error>> {
    let _lock = INTEGRATION_ENVIRONMENT_LOCK.lock().unwrap();
    let (tests_dir, owned_compile_args) = prepare_integration_environment()?;
    let extra_compile_args = borrowed_args(&owned_compile_args);
    run_ui_control_alignment_app(&tests_dir, &extra_compile_args)
}

#[test]
fn ui_button_content_edge_insets_contract() -> Result<(), Box<dyn Error>> {
    let _lock = INTEGRATION_ENVIRONMENT_LOCK.lock().unwrap();
    let (tests_dir, owned_compile_args) = prepare_integration_environment()?;
    let extra_compile_args = borrowed_args(&owned_compile_args);
    run_ui_button_content_edge_insets_app(&tests_dir, &extra_compile_args)
}
