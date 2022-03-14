use pyo3::prelude::*;
pub mod structs;
pub mod book_store_info;

#[pymodule]
fn school_library(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<book_store_info::BookStoreInfo>()?;
    Ok(())
}
