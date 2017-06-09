#[macro_use]
extern crate error_chain;
extern crate dtoa;
extern crate nom;
#[macro_use]
extern crate cpython;

mod datfile;
mod errors;

use datfile::{maybe_any_field, Field};
use cpython::{Python, PyResult, PyObject, ToPyObject, PythonObject, PyErr};
use cpython::exc::BaseException;

py_module_initializer!(nastranrs, initnastranrs, PyInit_nastranrs, |py, m| {
    try!(m.add(py, "__doc__", "This module is implemented in Rust."));
    try!(m.add(py, "parse_field", py_fn!(py, parse_field(field: String))));
    Ok(())
});

py_exception!(nastranrs, ParseFieldError, BaseException);

fn parse_field(py: Python, field: String) -> PyResult<PyObject> {
    match maybe_any_field(field.as_bytes()) {
        Ok(Field::Int(i)) => Ok(i.to_py_object(py).into_object()),
        Ok(Field::Float(v)) => Ok(v.to_py_object(py).into_object()),
        Ok(Field::Double(v)) => Ok(v.to_py_object(py).into_object()),
        Ok(Field::Blank) => Ok("".to_string().to_py_object(py).into_object()),
        Ok(Field::String(_)) => Ok(field.trim().to_py_object(py).into_object()),
        Ok(Field::DoubleString(_)) => Ok(field.trim().to_py_object(py).into_object()),
        Ok(Field::Continuation(_)) => Ok(field.trim().to_py_object(py).into_object()),
        Ok(Field::DoubleContinuation(_)) => Ok(field.trim().to_py_object(py).into_object()),
        Err(_) => {
            let msg = format!("Couldn't parse field '{}'", field);
            Err(PyErr::new::<ParseFieldError, String>(py, msg))
        }
    }
}
