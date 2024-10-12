use std::path::PathBuf;
use std::sync::{Arc, Once};

use log::LevelFilter;
use mimalloc::MiMalloc;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyString};
use pyo3_log::Logger;

use ::kitoken::Kitoken as Inner;

#[global_allocator]
static ALLOCATOR: MiMalloc = MiMalloc;

#[pyclass(frozen)]
#[derive(Debug)]
pub struct Kitoken {
    inner: Arc<Inner>,
}
#[pymethods]
impl Kitoken {
    #[new]
    pub fn new(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_slice(data))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_file(path: Bound<PyString>, py: Python<'_>) -> PyResult<Kitoken> {
        let mut path = PathBuf::from(path.extract::<&str>()?);
        if path.is_relative() {
            path = py
                .eval_bound("__builtins__.__import__('os').path.realpath('__file__')", None, None)?
                .extract::<String>()?
                .parse::<PathBuf>()?
                .parent()
                .ok_or_else(|| PyValueError::new_err("no parent directory"))?
                .join(path);
        }
        if !path.exists() {
            return Err(PyValueError::new_err(
                ["file not found: ", &path.display().to_string()].concat(),
            ));
        }
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_file(path))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[pyo3(signature = (text, encode_specials=false))]
    pub fn encode<'a>(
        &self, text: Bound<'a, PyString>, encode_specials: Option<bool>, py: Python<'a>,
    ) -> PyResult<Bound<'a, PyList>> {
        let text = text.extract::<&str>()?;
        py.allow_threads(|| self.inner.encode(text, encode_specials.unwrap_or(false)))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
            .map(|tokens| PyList::new_bound(py, tokens))
    }

    #[pyo3(signature = (tokens, decode_specials=false))]
    pub fn decode<'a>(
        &self, tokens: Bound<'a, PyList>, decode_specials: Option<bool>, py: Python<'a>,
    ) -> PyResult<Bound<'a, PyBytes>> {
        let tokens = tokens.extract::<Vec<u32>>()?;
        py.allow_threads(|| self.inner.decode(tokens, decode_specials.unwrap_or(false)))
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
            .map(|s| PyBytes::new_bound(py, &s))
    }

    #[staticmethod]
    pub fn from_sentencepiece(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_sentencepiece_slice(data))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_sentencepiece_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_sentencepiece_file(path))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_tiktoken(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tiktoken_slice(data))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_tiktoken_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tiktoken_file(path))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_tokenizers(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tokenizers_slice(data))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_tokenizers_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tokenizers_file(path))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }

    #[staticmethod]
    pub fn from_tekken_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tekken_file(path))
                .map(Arc::new)
                .map_err(|e| PyValueError::new_err(format!("{}", e)))?,
        })
    }
}

#[pymodule]
fn kitoken(m: &Bound<'_, PyModule>) -> PyResult<()> {
    static INIT_LOGGER: Once = Once::new();
    INIT_LOGGER.call_once(|| {
        let _ = Logger::default()
            .filter(LevelFilter::Trace)
            .install()
            .map_err(|e| eprintln!("{}", e));
    });
    m.add_class::<Kitoken>()?;
    Ok(())
}
