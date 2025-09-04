pub mod trace;

mod channel;
mod engine;
mod exchange;
mod execution;
mod indexer;
mod path;
mod plot;
mod protocol;
mod route;
mod subscription;

use pyo3::prelude::*;

#[pyfunction]
fn py_test() {
    println!("Hello from py test!");
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn quantx(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_test, m)?)?;

    Ok(())
}
