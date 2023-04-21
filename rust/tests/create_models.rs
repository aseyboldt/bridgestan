use std::{
    f64::consts::PI,
    ffi::CString,
    path::{Path, PathBuf},
    thread::spawn, sync::{Barrier, Arc},
};

use approx::assert_ulps_eq;
use bridgestan::{open_library, BridgeStanError, Model, StanLibrary};

fn model_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test_models")
}

/// Load stan library and corresponding data if available
fn get_model<S: AsRef<str>>(name: S) -> (StanLibrary, Option<CString>) {
    let name = name.as_ref();
    let mut base = model_dir();
    base.push(name);
    let lib = base.join(format!("{}_model.so", name));
    if !lib.exists() {
        panic!("Could not find compiled model {}", name);
    }
    let lib = open_library(lib).unwrap_or_else(|_| panic!("Could not open library {}", name,));

    let data_path = base.join(name).with_extension("data.json");

    if data_path.exists() {
        let contents = std::fs::read(data_path).unwrap();
        (lib, Some(CString::new(contents).unwrap()))
    } else {
        (lib, None)
    }
}

#[test]
fn create_all_serial() {
    let base = model_dir();
    for path in base.read_dir().unwrap() {
        let path = path.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();

        if (name == "logistic") | (name == "regression") | (name == "syntax_error") {
            continue;
        }

        let (lib, data) = get_model(name);
        // Create the model with a reference
        let Ok(model) = Model::new(&lib, data.as_ref(), 42) else {
            // Only those two models should fail to create.
            assert!((name == "ode") | (name == "throw_data"));
            continue;
        };
        assert!(model.name().unwrap().contains(name));
        println!("{}", name);
    }
}

#[test]
fn create_all_late_drop_bwd() {
    let base = model_dir();
    let names: Vec<String> = base
        .read_dir()
        .unwrap()
        .map(|path| {
            let path = path.unwrap().path();
            path.file_name().unwrap().to_str().unwrap().to_string()
        })
        .collect();

    let handles: Vec<_> = names
        .into_iter()
        .filter(|name| {
            (name != "logistic") & (name != "regression") & (name != "syntax_error")
        })
        .map(|name| {
            println!("Load {}", name);
            let (lib, data) = get_model(&name);
            /*
            let Ok(model) = Model::new(&lib, data.as_ref(), 42) else {
                // Only those two models should fail to create.
                assert!((name == "ode") | (name == "throw_data"));
                return (lib, name)
            };
            assert!(model.name().unwrap().contains(&name));
            drop(model);
            */
            (lib, name)
        })
        .collect();
    handles
        .into_iter()
        .rev()
        .for_each(|(lib, name)| {
            println!("Drop bwd {}", name);
            drop(lib)
        })
}

#[test]
fn create_all_late_drop_fwd() {
    let base = model_dir();
    let names: Vec<String> = base
        .read_dir()
        .unwrap()
        .map(|path| {
            let path = path.unwrap().path();
            path.file_name().unwrap().to_str().unwrap().to_string()
        })
        .collect();

    let handles: Vec<_> = names
        .into_iter()
        .filter(|name| {
            (name != "logistic") & (name != "regression") & (name != "syntax_error")
        })
        .map(|name| {
            println!("Load {}", name);
            let (lib, data) = get_model(&name);
            /*
            let Ok(model) = Model::new(&lib, data.as_ref(), 42) else {
                // Only those two models should fail to create.
                assert!((name == "ode") | (name == "throw_data"));
                return (lib, name)
            };
            assert!(model.name().unwrap().contains(&name));
            drop(model);
            */
            (lib, name)
        })
        .collect();
    handles
        .into_iter()
        .for_each(|(lib, name)| {
            println!("Drop fwd {}", name);
            drop(lib)
        })
}

#[test]
fn create_all_thread_serial() {
    let base = model_dir();
    let names: Vec<String> = base
        .read_dir()
        .unwrap()
        .map(|path| {
            let path = path.unwrap().path();
            path.file_name().unwrap().to_str().unwrap().to_string()
        })
        .collect();

    names
        .into_iter()
        .for_each(|name| {
            spawn(move || {
                if (&name == "logistic") | (&name == "regression") | (&name == "syntax_error") {
                    return;
                }

                let (lib, data) = get_model(&name);
                /*
                // Create the model with a reference
                let Ok(model) = Model::new(&lib, data.as_ref(), 42) else {
                    // Only those two models should fail to create.
                    assert!((name == "ode") | (name == "throw_data"));
                    return;
                };
                assert!(model.name().unwrap().contains(&name));
                */
            }).join().unwrap()
        })
    //handles
        //.into_iter()
        //.for_each(|handle| handle.join().unwrap())
}

#[test]
fn create_all_thread_barrier_serial() {
    let base = model_dir();
    let names: Vec<String> = base
        .read_dir()
        .unwrap()
        .map(|path| {
            let path = path.unwrap().path();
            path.file_name().unwrap().to_str().unwrap().to_string()
        })
        .filter(|name| {
            (name != "logistic") & (name != "regression") & (name != "syntax_error")
        })
        .collect();

    let barrier = Arc::new(Barrier::new(names.len()));

    let handles: Vec<_> = names
        .into_iter()
        .map(|name| {
            let barrier = Arc::clone(&barrier);
            spawn(move || {
                let (lib, data) = get_model(&name);

                barrier.wait();

                drop(lib)
                /*
                // Create the model with a reference
                let Ok(model) = Model::new(&lib, data.as_ref(), 42) else {
                    // Only those two models should fail to create.
                    assert!((name == "ode") | (name == "throw_data"));
                    return;
                };
                assert!(model.name().unwrap().contains(&name));
                */
            })
        })
        .collect();

    handles
        .into_iter()
        .for_each(|handle| handle.join().unwrap())
}

#[test]
fn create_all_parallel() {
    let base = model_dir();
    let names: Vec<String> = base
        .read_dir()
        .unwrap()
        .map(|path| {
            let path = path.unwrap().path();
            path.file_name().unwrap().to_str().unwrap().to_string()
        })
        .collect();

    let handles: Vec<_> = names
        .into_iter()
        .map(|name| {
            spawn(move || {
                if (&name == "logistic") | (&name == "regression") | (&name == "syntax_error") {
                    return;
                }

                let (lib, data) = get_model(&name);
                /*
                // Create the model with a reference
                let Ok(model) = Model::new(&lib, data.as_ref(), 42) else {
                    // Only those two models should fail to create.
                    assert!((name == "ode") | (name == "throw_data"));
                    return;
                };
                assert!(model.name().unwrap().contains(&name));
                */
            })
        })
        .collect();
    handles
        .into_iter()
        .for_each(|handle| handle.join().unwrap())
}

#[test]
fn load_after_unload_diff() {
    let (lib1, data1) = get_model("throw_data");
    drop(lib1);

    let (lib2, data2) = get_model("stdnormal");
    drop(lib2);
}

#[test]
fn load_after_unload_same() {
    let (lib1, data1) = get_model("throw_data");
    let Err(_) = Model::new(&lib1, data1, 42) else {
        panic!("Did not return error")
    };
    drop(lib1);

    let (lib2, data2) = get_model("throw_data");
    let Err(_) = Model::new(&lib2, data2, 42) else {
        panic!("Did not return error")
    };
    drop(lib2);
}

#[test]
fn load_twice_diff() {
    let (lib1, data1) = get_model("throw_data");
    let (lib2, data2) = get_model("stdnormal");

    drop(lib1);
    drop(lib2);
}

#[test]
fn load_twice_reorder_diff() {
    let (lib1, data1) = get_model("throw_data");
    let (lib2, data2) = get_model("stdnormal");

    drop(lib2);
    drop(lib1);
}

#[test]
fn load_twice_same() {
    let (lib1, data1) = get_model("throw_data");
    let (lib2, data2) = get_model("throw_data");

    let Err(_) = Model::new(&lib1, data1, 42) else {
        panic!("Did not return error")
    };
    let Err(_) = Model::new(&lib2, data2, 42) else {
        panic!("Did not return error")
    };
    drop(lib1);
    drop(lib2);
}

#[test]
fn load_parallel() {
    let handles: Vec<_> = (0..50)
        .map(|_| {
            spawn(|| {
                let (lib1, data1) = get_model("throw_data");
                let Err(_) = Model::new(&lib1, data1, 42) else {
                    panic!("Did not return error")
                };
            })
        })
        .collect();
    handles
        .into_iter()
        .for_each(|handle| handle.join().unwrap())
}

#[test]
fn throw_data() {
    let (lib, data) = get_model("throw_data");
    let Err(err) = Model::new(&lib, data, 42) else {
        panic!("throw_data model should not successfully be created.");
    };

    let BridgeStanError::ConstructFailed(msg) = err else {
        panic!("Creating throw_data model return an unexpected error");
    };
    assert!(msg.contains("find this text: datafails"));
}

#[test]
#[should_panic(expected = "number of parameters")]
fn bad_arglength() {
    let (lib, data) = get_model("stdnormal");
    let model = Model::new(&lib, data, 42).unwrap();
    let theta = vec![];
    let mut grad = vec![];
    let _ = model.log_density_gradient(&theta[..], true, true, &mut grad[..]);
}

#[test]
fn logp_gradient() {
    let (lib, data) = get_model("stdnormal");
    let model = Model::new(&lib, data, 42).unwrap();
    let theta = vec![1f64];
    let mut grad = vec![0f64];
    let logp = model
        .log_density_gradient(&theta[..], false, true, &mut grad[..])
        .unwrap();
    assert_ulps_eq!(logp, (2. * PI).sqrt().recip().ln() - 0.5);
    assert_ulps_eq!(grad[0], -1f64);
}
