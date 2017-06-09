#[macro_use]
extern crate error_chain;
extern crate dtoa;
extern crate nom;
#[macro_use]
extern crate cpython;

mod datfile;
mod errors;

use std::slice;

use datfile::{maybe_field, Field};
use cpython::{Python, PyResult, PyObject, ToPyObject, PythonObject};

py_module_initializer!(nastranrs, initnastranrs, PyInit_nastranrs, |py, m| {
    try!(m.add(py, "__doc__", "This module is implemented in Rust."));
    try!(m.add(py,
               "parse_field",
               py_fn!(py, parse_field(field: String))));
    Ok(())
});

fn parse_field(py: Python, field: String) -> PyResult<PyObject> {
    match maybe_field(field.as_bytes()) {
        Ok(Field::Int(i)) => Ok(i.to_py_object(py).into_object()),
        Ok(Field::Float(v)) => Ok(v.to_py_object(py).into_object()),
        Ok(Field::Double(v)) => Ok(v.to_py_object(py).into_object()),
        Ok(_) => Ok(py.None()),
        Err(_) => Ok(py.None()),
    }
}
