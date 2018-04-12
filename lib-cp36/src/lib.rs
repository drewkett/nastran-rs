extern crate nastran;
#[macro_use]
extern crate cpython;

use nastran::datfile::{self, maybe_any_field, Field};
use cpython::{Python, PyResult, PyObject, ToPyObject, PythonObject, PyErr, PyList};
use cpython::exc::BaseException;

py_module_initializer!(nastranrs, initnastranrs, PyInit_nastranrs, |py, m| {
    try!(m.add(py, "__doc__", "This module is implemented in Rust."));
    try!(m.add(
        py,
        "parse_field",
        py_fn!(py, parse_field(field: String)),
    ));
    try!(m.add(
        py,
        "parse_line",
        py_fn!(py, parse_line(line: String)),
    ));
    Ok(())
});

py_exception!(nastranrs, ParseFieldError, BaseException);

pub fn parse_field(py: Python, field: String) -> PyResult<PyObject> {
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

fn field_to_pyobject(py: Python,field: Field) -> PyObject {
    match field {
        Field::Blank => "".to_py_object(py).into_object(),
        Field::Int(v) => v.to_py_object(py).into_object(),
        Field::Float(v) => v.to_py_object(py).into_object(),
        Field::Double(v) => v.to_py_object(py).into_object(),
        Field::Continuation(v) => v.trim().to_py_object(py).into_object(),
        Field::DoubleContinuation(v) => v.trim().to_py_object(py).into_object(),
        Field::String(v) => v.trim().to_py_object(py).into_object(),
        Field::DoubleString(v) => v.trim().to_py_object(py).into_object(),
    }
}


pub fn parse_line(py: Python, field: String) -> PyResult<PyList> {
    match datfile::parse_line(field.as_bytes()) {
        Ok(card) => {
            let mut list = vec![];
            let obj = field_to_pyobject(py,card.first);
            list.push(obj);
            for field in card.fields {
                let obj = field_to_pyobject(py, field);
                list.push(obj);
            }
            if let Some(cont) = card.continuation {
                list.push(cont.to_py_object(py).into_object());
            };
            Ok(PyList::new(py,list.as_slice()))
        }
        Err(e) => {
            println!("{}",e);
            let msg = format!("Couldn't parse line '{}'", field);
            Err(PyErr::new::<ParseFieldError, String>(py, msg))
        }

    }
}
