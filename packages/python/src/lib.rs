use core::fmt::Display;
use std::path::PathBuf;
use std::sync::{Arc, Once};

use log::LevelFilter;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyString};
use pyo3_log::Logger;

use ::kitoken::Kitoken as Inner;
use serde_pyobject::{from_pyobject, to_pyobject};

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[pyclass(module = "kitoken", subclass, weakref)]
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
                .map_err(convert_error)?,
        })
    }

    #[pyo3(signature = (text, encode_specials=false))]
    pub fn encode<'a>(
        &self, text: Bound<'a, PyString>, encode_specials: Option<bool>, py: Python<'a>,
    ) -> PyResult<Bound<'a, PyList>> {
        let text = text.extract::<&str>()?;
        py.allow_threads(|| self.inner.encode(text, encode_specials.unwrap_or(false)))
            .map_err(convert_error)
            .map(|tokens| PyList::new(py, tokens))
            .and_then(|texts| texts)
    }

    #[pyo3(signature = (text, encode_specials=false))]
    pub fn encode_all<'a>(
        &self, text: Bound<'a, PyList>, encode_specials: Option<bool>, py: Python<'a>,
    ) -> PyResult<Bound<'a, PyList>> {
        let text = text.extract::<Vec<String>>()?;
        py.allow_threads(|| {
            text.iter()
                .map(|text| self.inner.encode(text, encode_specials.unwrap_or(false)))
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(convert_error)
        .map(|tokens| PyList::new(py, tokens))
        .and_then(|texts| texts)
    }

    #[pyo3(signature = (tokens, decode_specials=false))]
    pub fn decode<'a>(
        &self, tokens: Bound<'a, PyList>, decode_specials: Option<bool>, py: Python<'a>,
    ) -> PyResult<Bound<'a, PyBytes>> {
        let tokens = tokens.extract::<Vec<u32>>()?;
        py.allow_threads(|| self.inner.decode(tokens, decode_specials.unwrap_or(false)))
            .map_err(convert_error)
            .map(|s| PyBytes::new(py, &s))
    }

    #[pyo3(signature = (tokens, decode_specials=false))]
    pub fn decode_all<'a>(
        &self, tokens: Bound<'a, PyList>, decode_specials: Option<bool>, py: Python<'a>,
    ) -> PyResult<Bound<'a, PyList>> {
        let tokens = tokens.extract::<Vec<Vec<u32>>>()?;
        py.allow_threads(|| {
            tokens
                .into_iter()
                .map(|tokens| self.inner.decode(&tokens, decode_specials.unwrap_or(false)))
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(convert_error)
        .map(|texts| PyList::new(py, texts.iter().map(|s| PyBytes::new(py, s))))
        .and_then(|texts| texts)
    }

    pub fn definition<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        to_pyobject(py, &self.inner.to_definition()).map_err(convert_error)
    }

    pub fn set_definition<'a>(
        &mut self, definition: Bound<'a, PyAny>, py: Python<'a>,
    ) -> PyResult<()> {
        let definition = from_pyobject(definition)?;
        self.inner = py
            .allow_threads(|| Inner::from_definition(definition))
            .map(Arc::new)
            .map_err(convert_error)?;
        Ok(())
    }

    pub fn config<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        to_pyobject(py, &self.inner.to_definition().config).map_err(convert_error)
    }

    pub fn set_config<'a>(&mut self, config: Bound<'a, PyAny>, py: Python<'a>) -> PyResult<()> {
        let mut definition = self.inner.to_definition();
        definition.config = from_pyobject(config)?;
        self.inner = py
            .allow_threads(|| Inner::from_definition(definition))
            .map(Arc::new)
            .map_err(convert_error)?;
        Ok(())
    }

    #[staticmethod]
    pub fn from_file(path: Bound<PyString>, py: Python<'_>) -> PyResult<Kitoken> {
        let mut path = PathBuf::from(path.extract::<&str>()?);
        if path.is_relative() {
            path = py
                .eval(c"__builtins__.__import__('os').path.realpath('__file__')", None, None)?
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
                .map_err(convert_error)?,
        })
    }

    pub fn to_file<'a>(&self, path: Bound<'a, PyString>, py: Python<'a>) -> PyResult<()> {
        let mut path = PathBuf::from(path.extract::<&str>()?);
        if path.is_relative() {
            path = py
                .eval(c"__builtins__.__import__('os').path.realpath('__file__')", None, None)?
                .extract::<String>()?
                .parse::<PathBuf>()?
                .parent()
                .ok_or_else(|| PyValueError::new_err("no parent directory"))?
                .join(path);
        }
        py.allow_threads(|| self.inner.to_file(path)).map_err(convert_error)
    }

    pub fn to_bytes<'a>(&self, py: Python<'a>) -> Bound<'a, PyBytes> {
        PyBytes::new(py, &py.allow_threads(|| self.inner.to_vec()))
    }

    #[staticmethod]
    pub fn from_sentencepiece(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_sentencepiece_slice(data))
                .map(Arc::new)
                .map_err(convert_error)?,
        })
    }

    #[staticmethod]
    pub fn from_sentencepiece_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_sentencepiece_file(path))
                .map(Arc::new)
                .map_err(convert_error)?,
        })
    }

    #[staticmethod]
    pub fn from_tiktoken(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tiktoken_slice(data))
                .map(Arc::new)
                .map_err(convert_error)?,
        })
    }

    #[staticmethod]
    pub fn from_tiktoken_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tiktoken_file(path))
                .map(Arc::new)
                .map_err(convert_error)?,
        })
    }

    #[staticmethod]
    pub fn from_tokenizers(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tokenizers_slice(data))
                .map(Arc::new)
                .map_err(convert_error)?,
        })
    }

    #[staticmethod]
    pub fn from_tokenizers_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tokenizers_file(path))
                .map(Arc::new)
                .map_err(convert_error)?,
        })
    }

    #[staticmethod]
    pub fn from_tekken(data: &[u8], py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tekken_slice(data))
                .map(Arc::new)
                .map_err(convert_error)?,
        })
    }

    #[staticmethod]
    pub fn from_tekken_file(path: &str, py: Python<'_>) -> PyResult<Kitoken> {
        Ok(Kitoken {
            inner: py
                .allow_threads(|| Inner::from_tekken_file(path))
                .map(Arc::new)
                .map_err(convert_error)?,
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

#[inline(never)]
fn convert_error(e: impl Display) -> PyErr {
    PyValueError::new_err(format!("{}", e))
}
