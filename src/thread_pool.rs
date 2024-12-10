use crate::utils::result2pyresult;
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass]
#[derive(Clone)]
pub struct ResizerThreadPool {
    pool: Arc<rayon::ThreadPool>,
}

#[pymethods]
impl ResizerThreadPool {
    #[new]
    #[pyo3(signature = (num_threads = None))]
    fn new(num_threads: Option<usize>) -> PyResult<Self> {
        let mut builder = rayon::ThreadPoolBuilder::new();
        if let Some(num) = num_threads {
            builder = builder.num_threads(num);
        }
        let pool = result2pyresult(builder.build())?;
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    #[getter]
    fn current_num_threads(&self) -> usize {
        self.pool.current_num_threads()
    }
}

impl ResizerThreadPool {
    #[inline]
    pub fn run_within<OP, R>(&self, op: OP) -> R
    where
        OP: FnOnce() -> R + Send,
        R: Send,
    {
        self.pool.install(op)
    }
}
