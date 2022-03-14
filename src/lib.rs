use pyo3::prelude::*;
pub mod structs;
pub mod book_store_info;
#[pyclass]
struct Number(i32);

#[pymethods]
impl Number {
    #[new]
    fn new(value: i32) -> Self {
        Self(value)
    }
}

#[pymodule]
fn school_library(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Number>()?;
    m.add_class::<book_store_info::BookStoreInfo>()?;
    Ok(())
}
