use nastran::datfile::{self, maybe_any_field, Field};
use pyo3::prelude::*;
use pyo3::{create_exception, types::PyList, wrap_pyfunction};

#[pymodule]
fn naspy(_py: Python, m: &PyModule) -> PyResult<()> {
    //This module is implemented in Rust.
    m.add_wrapped(wrap_pyfunction!(parse_field))?;
    m.add_wrapped(wrap_pyfunction!(parse_line))?;
    Ok(())
}

create_exception!(naspy, ParseFieldError, pyo3::exceptions::Exception);

#[pyfunction]
pub fn parse_field(py: Python, field: String) -> PyResult<PyObject> {
    maybe_any_field(field.as_bytes())
        .map(|f| field_to_pyobject(py, f))
        .map_err(|_| {
            let msg = format!("Couldn't parse field '{}'", field);
            PyErr::new::<ParseFieldError, _>(msg)
        })
}

fn field_to_pyobject(py: Python, field: Field) -> PyObject {
    match field {
        Field::Blank => "".to_object(py),
        Field::Int(v) => v.to_object(py),
        Field::Float(v) => v.to_object(py),
        Field::Double(v) => v.to_object(py),
        Field::Continuation(v) => v.to_object(py),
        Field::DoubleContinuation(v) => v.to_object(py),
        Field::String(v) => v.to_object(py),
        Field::DoubleString(v) => v.to_object(py),
    }
}

#[pyfunction]
pub fn parse_line(py: Python, field: String) -> PyResult<&PyList> {
    match datfile::parse_line(field.as_bytes()) {
        Ok(card) => {
            let mut list = vec![];
            if let Some(o) = card.first {
                list.push(field_to_pyobject(py, o));
            } else {
                list.push(field_to_pyobject(py, Field::Blank));
            }
            for field in card.fields {
                let obj = field_to_pyobject(py, field);
                list.push(obj);
            }
            list.push(card.continuation.to_object(py));
            Ok(PyList::new(py, list.as_slice()))
        }
        Err(e) => {
            println!("{}", e);
            let msg = format!("Couldn't parse line '{}'", field);
            Err(PyErr::new::<ParseFieldError, _>(msg))
        }
    }
}
