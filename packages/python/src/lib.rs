use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use ::kitoken::Kitoken as Inner;

#[pyclass]
#[derive(Debug)]
pub struct Kitoken {
    inner: Inner,
}
#[pymethods]
impl Kitoken {
    #[new]
    pub fn new(data: &[u8]) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: Inner::from_slice(data)
                .map_err(|e| PyValueError::new_err(format!("{:?}", e)))?,
        })
    }

    pub fn encode(&self, text: &str, encode_specials: Option<bool>) -> PyResult<Vec<u32>> {
        self.inner
            .encode(text, encode_specials.unwrap_or(false))
            .map_err(|e| PyValueError::new_err(format!("{:?}", e)))
    }

    pub fn decode(&self, tokens: Vec<u32>, py: Python<'_>) -> PyResult<PyObject> {
        self.inner
            .decode(tokens)
            .map_err(|e| PyValueError::new_err(format!("{:?}", e)))
            .map(|s| PyBytes::new(py, &s).into())
    }

    #[staticmethod]
    pub fn from_sentencepiece(data: &[u8]) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: Inner::from_sentencepiece_slice(data)
                .map_err(|e| PyValueError::new_err(format!("{:?}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_tiktoken(data: &[u8]) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: Inner::from_tiktoken_slice(data)
                .map_err(|e| PyValueError::new_err(format!("{:?}", e)))?,
        })
    }
}

#[pymodule]
fn kitoken(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Kitoken>()?;
    Ok(())
}
