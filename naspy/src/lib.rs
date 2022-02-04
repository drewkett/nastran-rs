use pyo3::create_exception;
use pyo3::prelude::*;

#[pymodule]
fn naspy(_py: Python, _m: &PyModule) -> PyResult<()> {
    //This module is implemented in Rust.
    // m.add_wrapped(wrap_pyfunction!(parse_field))?;
    // m.add_wrapped(wrap_pyfunction!(parse_line))?;
    Ok(())
}

create_exception!(naspy, ParseFieldError, pyo3::exceptions::PyException);

// #[pyfunction]
// pub fn parse_field(py: Python, field: String) -> PyResult<PyObject> {
//     maybe_any_field(field.as_bytes())
//         .map(|f| field_to_pyobject(py, f))
//         .map_err(|_| {
//             let msg = format!("Couldn't parse field '{}'", field);
//             PyErr::new::<ParseFieldError, _>(msg)
//         })
// }
