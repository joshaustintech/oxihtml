use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use oxihtml::html5lib::{
    discover_serializer_files, discover_tokenizer_files, discover_tree_construction_files, parse_json_file,
    parse_tree_construction_dat, ScriptDirective,
};
use oxihtml::serialize::{normalize_tree_text, to_test_format};
use oxihtml::html5lib::Json;
use oxihtml::{FragmentContext, Options, Parser};

#[derive(Clone, Debug)]
struct Config {
    tests_root: PathBuf,
    mode_tree: bool,
    mode_tokenizer: bool,
    mode_serializer: bool,
    list_only: bool,
    smoke: bool,
    threads: usize,
    max_failures: usize,
    fail_fast: bool,
    filter: Option<String>,
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn parse_args() -> Result<Config, String> {
    let mut tests_root = None::<PathBuf>;
    let mut mode_tree = false;
    let mut mode_tokenizer = false;
    let mut mode_serializer = false;
    let mut list_only = false;
    let mut smoke = false;
    let mut threads = None::<usize>;
    let mut max_failures = 20usize;
    let mut fail_fast = false;
    let mut filter = None::<String>;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--tests" => {
                let p = args.next().ok_or("--tests needs a path")?;
                tests_root = Some(expand_tilde(&p));
            }
            "--tree" => mode_tree = true,
            "--tokenizer" => mode_tokenizer = true,
            "--serializer" => mode_serializer = true,
            "--all" => {
                mode_tree = true;
                mode_tokenizer = true;
                mode_serializer = true;
            }
            "--list" => list_only = true,
            "--smoke" => smoke = true,
            "--threads" => {
                let n = args.next().ok_or("--threads needs a number")?;
                threads = Some(n.parse::<usize>().map_err(|_| "invalid --threads")?);
            }
            "--max-failures" => {
                let n = args.next().ok_or("--max-failures needs a number")?;
                max_failures = n.parse::<usize>().map_err(|_| "invalid --max-failures")?;
            }
            "--fail-fast" => fail_fast = true,
            "--filter" => {
                filter = Some(args.next().ok_or("--filter needs a string")?);
            }
            "--help" | "-h" => {
                return Err(
                    "Usage: html5lib-runner --tests ~/html5lib-tests [--tree|--tokenizer|--serializer|--all] [--list] [--smoke] [--threads N] [--max-failures N] [--fail-fast] [--filter SUBSTR]"
                        .to_string(),
                );
            }
            _ => return Err(format!("unknown arg: {arg}")),
        }
    }

    let tests_root = tests_root.unwrap_or_else(|| expand_tilde("~/html5lib-tests"));
    let threads = threads.unwrap_or_else(|| {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });

    if !(mode_tree || mode_tokenizer || mode_serializer) {
        mode_tree = true;
    }

    Ok(Config {
        tests_root,
        mode_tree,
        mode_tokenizer,
        mode_serializer,
        list_only,
        smoke,
        threads: threads.max(1),
        max_failures: max_failures.max(1),
        fail_fast,
        filter,
    })
}

#[derive(Clone, Debug)]
struct Failure {
    file: PathBuf,
    case_index: usize,
    script: &'static str,
    input: String,
    expected: String,
    actual: String,
}

#[derive(Clone, Debug, Default)]
struct Summary {
    total: usize,
    passed: usize,
    failed: usize,
    failures: Vec<Failure>,
}

fn json_obj_get<'a>(obj: &'a [(String, Json)], key: &str) -> Option<&'a Json> {
    obj.iter().find_map(|(k, v)| (k == key).then_some(v))
}

fn unimplemented_failure(file: PathBuf, case_index: usize, label: &'static str, input: String) -> Failure {
    Failure {
        file,
        case_index,
        script: label,
        input,
        expected: "(implemented parser output)".to_string(),
        actual: "(unimplemented)".to_string(),
    }
}

fn run_tree_file(path: &Path, tests_root: &Path, max_failures: usize, fail_fast: bool) -> Summary {
    let mut summary = Summary::default();
    let cases = match parse_tree_construction_dat(path) {
        Ok(c) => c,
        Err(e) => {
            summary.total = 1;
            summary.failed = 1;
            summary.failures.push(Failure {
                file: path.to_path_buf(),
                case_index: 0,
                script: "n/a",
                input: format!("(failed to read/parse .dat: {e})"),
                expected: String::new(),
                actual: String::new(),
            });
            return summary;
        }
    };

    for (i, case) in cases.iter().enumerate() {
        let script_modes: &[(bool, &'static str)] = match case.script_directive {
            ScriptDirective::On => &[(true, "on")],
            ScriptDirective::Off => &[(false, "off")],
            ScriptDirective::Both => &[(true, "on"), (false, "off")],
        };

        for (scripting_enabled, script_label) in script_modes {
            summary.total += 1;

            let mut parser = Parser::new(Options {
                scripting_enabled: *scripting_enabled,
                iframe_srcdoc: false,
                collect_errors: false,
            });

            let actual = if let Some(ctx) = &case.fragment_context {
                let parsed = parser.parse_fragment(
                    FragmentContext {
                        namespace: ctx.namespace.clone(),
                        tag_name: ctx.tag_name.clone(),
                    },
                    &case.data,
                );
                to_test_format(&parsed.value.arena, parsed.value.root)
            } else {
                let parsed = parser.parse_document(&case.data);
                to_test_format(&parsed.value.arena, parsed.value.root)
            };

            let expected_norm = normalize_tree_text(&case.expected);
            let actual_norm = normalize_tree_text(&actual);
            if expected_norm == actual_norm {
                summary.passed += 1;
                continue;
            }

            summary.failed += 1;
            if summary.failures.len() < max_failures {
                let rel = path.strip_prefix(tests_root).unwrap_or(path).to_path_buf();
                summary.failures.push(Failure {
                    file: rel,
                    case_index: i,
                    script: *script_label,
                    input: case.data.clone(),
                    expected: expected_norm,
                    actual: actual_norm,
                });
            }

            if fail_fast {
                return summary;
            }
        }
    }

    summary
}

fn run_tokenizer_suite(config: &Config) -> Summary {
    let mut summary = Summary::default();
    let mut files = match discover_tokenizer_files(&config.tests_root) {
        Ok(f) => f,
        Err(e) => {
            summary.total = 1;
            summary.failed = 1;
            summary.failures.push(unimplemented_failure(
                PathBuf::from("tokenizer"),
                0,
                "n/a",
                format!("failed to discover tokenizer fixtures: {e}"),
            ));
            return summary;
        }
    };
    if let Some(substr) = &config.filter {
        files.retain(|p| p.to_string_lossy().contains(substr));
    }

    for path in files {
        let rel = path.strip_prefix(&config.tests_root).unwrap_or(&path).to_path_buf();
        let json = match parse_json_file(&path) {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                summary.total += 1;
                summary.failed += 1;
                if summary.failures.len() < config.max_failures {
                    summary.failures.push(unimplemented_failure(
                        rel,
                        0,
                        "n/a",
                        format!("JSON parse error: {} @{}", e.message, e.offset),
                    ));
                }
                continue;
            }
            Err(e) => {
                summary.total += 1;
                summary.failed += 1;
                if summary.failures.len() < config.max_failures {
                    summary
                        .failures
                        .push(unimplemented_failure(rel, 0, "n/a", format!("read error: {e}")));
                }
                continue;
            }
        };

        let tests = match &json {
            Json::Object(obj) => match json_obj_get(obj, "tests") {
                Some(Json::Array(arr)) => arr,
                _ => {
                    summary.total += 1;
                    summary.failed += 1;
                    if summary.failures.len() < config.max_failures {
                        summary.failures.push(unimplemented_failure(
                            rel,
                            0,
                            "n/a",
                            "missing top-level tests array".to_string(),
                        ));
                    }
                    continue;
                }
            },
            _ => {
                summary.total += 1;
                summary.failed += 1;
                if summary.failures.len() < config.max_failures {
                    summary.failures.push(unimplemented_failure(
                        rel,
                        0,
                        "n/a",
                        "top-level JSON is not an object".to_string(),
                    ));
                }
                continue;
            }
        };

        for (i, test) in tests.iter().enumerate() {
            let (input, variants) = match test {
                Json::Object(obj) => {
                    let input = match json_obj_get(obj, "input") {
                        Some(Json::String(s)) => s.clone(),
                        _ => String::new(),
                    };
                    let variants = match json_obj_get(obj, "initialStates") {
                        Some(Json::Array(a)) if !a.is_empty() => a.len(),
                        _ => 1,
                    };
                    (input, variants)
                }
                _ => (String::new(), 1),
            };
            for _ in 0..variants {
                summary.total += 1;
                summary.failed += 1;
                if summary.failures.len() < config.max_failures {
                    summary.failures.push(unimplemented_failure(rel.clone(), i, "n/a", input.clone()));
                }
                if config.fail_fast {
                    return summary;
                }
            }
        }
    }

    summary
}

fn run_serializer_suite(config: &Config) -> Summary {
    let mut summary = Summary::default();
    let mut files = match discover_serializer_files(&config.tests_root) {
        Ok(f) => f,
        Err(e) => {
            summary.total = 1;
            summary.failed = 1;
            summary.failures.push(unimplemented_failure(
                PathBuf::from("serializer"),
                0,
                "n/a",
                format!("failed to discover serializer fixtures: {e}"),
            ));
            return summary;
        }
    };
    if let Some(substr) = &config.filter {
        files.retain(|p| p.to_string_lossy().contains(substr));
    }

    for path in files {
        let rel = path.strip_prefix(&config.tests_root).unwrap_or(&path).to_path_buf();
        let json = match parse_json_file(&path) {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                summary.total += 1;
                summary.failed += 1;
                if summary.failures.len() < config.max_failures {
                    summary.failures.push(unimplemented_failure(
                        rel,
                        0,
                        "n/a",
                        format!("JSON parse error: {} @{}", e.message, e.offset),
                    ));
                }
                continue;
            }
            Err(e) => {
                summary.total += 1;
                summary.failed += 1;
                if summary.failures.len() < config.max_failures {
                    summary
                        .failures
                        .push(unimplemented_failure(rel, 0, "n/a", format!("read error: {e}")));
                }
                continue;
            }
        };

        let tests = match &json {
            Json::Object(obj) => match json_obj_get(obj, "tests") {
                Some(Json::Array(arr)) => arr,
                _ => {
                    summary.total += 1;
                    summary.failed += 1;
                    if summary.failures.len() < config.max_failures {
                        summary.failures.push(unimplemented_failure(
                            rel,
                            0,
                            "n/a",
                            "missing top-level tests array".to_string(),
                        ));
                    }
                    continue;
                }
            },
            _ => {
                summary.total += 1;
                summary.failed += 1;
                if summary.failures.len() < config.max_failures {
                    summary.failures.push(unimplemented_failure(
                        rel,
                        0,
                        "n/a",
                        "top-level JSON is not an object".to_string(),
                    ));
                }
                continue;
            }
        };

        for (i, test) in tests.iter().enumerate() {
            let desc = match test {
                Json::Object(obj) => match json_obj_get(obj, "description") {
                    Some(Json::String(s)) => s.clone(),
                    _ => String::new(),
                },
                _ => String::new(),
            };
            summary.total += 1;
            summary.failed += 1;
            if summary.failures.len() < config.max_failures {
                summary.failures.push(unimplemented_failure(rel.clone(), i, "n/a", desc));
            }
            if config.fail_fast {
                return summary;
            }
        }
    }

    summary
}

fn main() -> std::process::ExitCode {
    let config = match parse_args() {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("{msg}");
            return std::process::ExitCode::from(2);
        }
    };

    if (config.mode_tokenizer || config.mode_serializer) && !config.list_only && !config.smoke {
        eprintln!("note: tokenizer/serializer execution is not implemented yet; use --smoke to validate fixture parsing");
    }

    if config.smoke {
        let mut ok = true;

        if config.mode_tree {
            let files = match discover_tree_construction_files(&config.tests_root) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("failed to discover tree-construction tests: {e}");
                    return std::process::ExitCode::from(2);
                }
            };
            for path in files {
                if let Err(e) = parse_tree_construction_dat(&path) {
                    ok = false;
                    eprintln!("tree .dat parse error: {}: {e}", path.display());
                }
            }
        }

        if config.mode_tokenizer {
            let files = match discover_tokenizer_files(&config.tests_root) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("failed to discover tokenizer tests: {e}");
                    return std::process::ExitCode::from(2);
                }
            };
            for path in files {
                match parse_json_file(&path) {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => {
                        ok = false;
                        eprintln!("tokenizer .test JSON parse error: {}: {} @{}", path.display(), e.message, e.offset);
                    }
                    Err(e) => {
                        ok = false;
                        eprintln!("tokenizer .test read error: {}: {e}", path.display());
                    }
                }
            }
        }

        if config.mode_serializer {
            let files = match discover_serializer_files(&config.tests_root) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("failed to discover serializer tests: {e}");
                    return std::process::ExitCode::from(2);
                }
            };
            for path in files {
                match parse_json_file(&path) {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => {
                        ok = false;
                        eprintln!("serializer .test JSON parse error: {}: {} @{}", path.display(), e.message, e.offset);
                    }
                    Err(e) => {
                        ok = false;
                        eprintln!("serializer .test read error: {}: {e}", path.display());
                    }
                }
            }
        }

        return if ok {
            std::process::ExitCode::SUCCESS
        } else {
            std::process::ExitCode::from(1)
        };
    }

    if !config.mode_tree {
        if !config.list_only {
            eprintln!("no runnable mode selected (only --tree execution is implemented currently)");
            return std::process::ExitCode::from(2);
        }
    }

    let mut files = match discover_tree_construction_files(&config.tests_root) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to discover tree-construction tests: {e}");
            return std::process::ExitCode::from(2);
        }
    };

    if let Some(substr) = &config.filter {
        files.retain(|p| p.to_string_lossy().contains(substr));
    }

    if config.list_only {
        println!("tree-construction files: {}", files.len());
        if config.mode_tokenizer {
            let tok = discover_tokenizer_files(&config.tests_root).unwrap_or_default();
            println!("tokenizer files: {}", tok.len());
        }
        if config.mode_serializer {
            let ser = discover_serializer_files(&config.tests_root).unwrap_or_default();
            println!("serializer files: {}", ser.len());
        }
        return std::process::ExitCode::SUCCESS;
    }

    let mut all = Summary::default();
    if !files.is_empty() {
        let threads = config.threads.min(files.len());
        let (tx, rx) = mpsc::channel::<Summary>();

        let chunk_size = (files.len() + threads - 1) / threads;
        for chunk in files.chunks(chunk_size) {
            let tx = tx.clone();
            let tests_root = config.tests_root.clone();
            let max_failures = config.max_failures;
            let fail_fast = config.fail_fast;
            let paths = chunk.to_vec();
            thread::spawn(move || {
                let mut summary = Summary::default();
                for path in paths {
                    let s = run_tree_file(
                        &path,
                        &tests_root,
                        max_failures.saturating_sub(summary.failures.len()),
                        fail_fast,
                    );
                    summary.total += s.total;
                    summary.passed += s.passed;
                    summary.failed += s.failed;
                    summary.failures.extend(s.failures);
                    if fail_fast && summary.failed > 0 {
                        break;
                    }
                    if summary.failures.len() >= max_failures {
                        break;
                    }
                }
                let _ = tx.send(summary);
            });
        }
        drop(tx);

        for s in rx {
            all.total += s.total;
            all.passed += s.passed;
            all.failed += s.failed;
            if all.failures.len() < config.max_failures {
                all.failures.extend(s.failures);
                all.failures.truncate(config.max_failures);
            }
        }
    }

    let mut exit_fail = all.failed > 0;

    println!("tree-construction: {}/{} passed ({} failed)", all.passed, all.total, all.failed);

    if config.mode_tokenizer {
        let tok = run_tokenizer_suite(&config);
        exit_fail |= tok.failed > 0;
        println!("tokenizer: {}/{} passed ({} failed)", tok.passed, tok.total, tok.failed);
        all.failures.extend(tok.failures);
    }

    if config.mode_serializer {
        let ser = run_serializer_suite(&config);
        exit_fail |= ser.failed > 0;
        println!("serializer: {}/{} passed ({} failed)", ser.passed, ser.total, ser.failed);
        all.failures.extend(ser.failures);
    }

    if !all.failures.is_empty() {
        all.failures.truncate(config.max_failures);
        println!("failures (showing up to {}):", config.max_failures);
        for f in &all.failures {
            println!("- {} case={} mode={}", f.file.display(), f.case_index, f.script);
        }
    }

    if exit_fail {
        std::process::ExitCode::from(1)
    } else {
        std::process::ExitCode::SUCCESS
    }
}
