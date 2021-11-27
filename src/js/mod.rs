use crate::{
    reexported::{mem, Box, Future, NonZeroUsize, Pin, Vec},
    FragmentId,
};
extern crate alloc;
use alloc::{format, sync::Arc};
use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Problem;

    #[wasm_bindgen(structural, method)]
    async fn direct_dependencies(this: &Problem, id: usize) -> JsValue;

    #[wasm_bindgen(catch, structural, method)]
    async fn evaluate(this: &Problem, id: usize) -> Result<(), JsValue>;
}

unsafe impl Send for Problem {}

unsafe impl Sync for Problem {}

// Can't use `async_trait` here because `Problem` is not `Send` and `async_trait` does not do the
// necessary unsafe convertion (see `unsafe_add_send`)
impl crate::Problem for Problem {
    type Error = JsValue;

    fn direct_dependencies<'life0, 'life1, 'async_trait>(
        &'life0 self,
        id: FragmentId,
        dependecies: &'life1 mut Vec<FragmentId>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        unsafe {
            unsafe_add_send(async move {
                dependecies.extend(
                    serde_wasm_bindgen::from_value::<Vec<usize>>(
                        Problem::direct_dependencies(self, id.into()).await,
                    )
                    .unwrap()
                    .into_iter()
                    .map(|x| FragmentId(x)),
                );
            })
        }
    }
    fn evaluate<'life0, 'async_trait>(
        &'life0 self,
        id: FragmentId,
    ) -> Pin<
        Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        unsafe {
            unsafe_add_send(
                async move { Problem::evaluate(self, id.into()).await },
            )
        }
    }
}

type BaseSolver = crate::Solver<Problem>;

#[wasm_bindgen]
pub struct Solver(Arc<BaseSolver>);

#[wasm_bindgen]
impl Solver {
    #[wasm_bindgen(constructor)]
    pub fn new(problem_instance: Problem) -> Self {
        Self(Arc::new(BaseSolver::new(problem_instance)))
    }

    pub fn into_problem_instance(self) -> Promise {
        wasm_bindgen_futures::future_to_promise(async move {
            Ok(BaseSolver::into_problem_instance(
                Arc::try_unwrap(self.0).map_err(|_| {
                    JsValue::from_str(
                        "Solver.into_problem_instance called with a clone",
                    )
                })?,
            )
            .into())
        })
    }

    pub fn status(&self) -> Promise {
        let this = self.0.clone();

        wasm_bindgen_futures::future_to_promise(async move {
            Ok(JsValue::from(format!(
                "{:?}",
                BaseSolver::status(&this).await
            )))
        })
    }

    pub fn enqueue_fragment(&self, id: usize) -> Promise {
        let this = self.0.clone();

        wasm_bindgen_futures::future_to_promise(async move {
            BaseSolver::enqueue_fragment(&this, FragmentId(id)).await;

            Ok(JsValue::undefined())
        })
    }

    pub fn punted_iter(&self) -> Promise {
        let this = self.0.clone();

        wasm_bindgen_futures::future_to_promise(async move {
            Ok(serde_wasm_bindgen::to_value(
                &BaseSolver::punted_iter(&this)
                    .await
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<usize>>(),
            )
            .unwrap())
        })
    }

    pub fn assume_evaluated(&self, id: usize) -> Promise {
        let this = self.0.clone();

        wasm_bindgen_futures::future_to_promise(async move {
            BaseSolver::assume_evaluated(&this, FragmentId(id)).await;

            Ok(JsValue::undefined())
        })
    }

    pub fn run(&self, concurrency: usize) -> Promise {
        let this = self.0.clone();

        wasm_bindgen_futures::future_to_promise(async move {
            Ok(serde_wasm_bindgen::to_value(
                &BaseSolver::run(
                    &this,
                    NonZeroUsize::new(concurrency).ok_or(JsValue::from_str(
                        "The `concurrency` argument for `run` must not be zero",
                    ))?,
                )
                .await?
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<usize>>(),
            )
            .unwrap())
        })
    }

    pub fn step(&self) -> Promise {
        let this = self.0.clone();

        wasm_bindgen_futures::future_to_promise(async move {
            BaseSolver::step(&this).await.map(JsValue::from_bool)
        })
    }
}

unsafe fn unsafe_add_send<'a, F, O>(
    future: F,
) -> Pin<Box<dyn Future<Output = O> + Send + 'a>>
where
    F: Future<Output = O> + 'a,
{
    Pin::new_unchecked(Box::from_raw(mem::transmute(Box::into_raw(Box::new(
        future,
    )
        as Box<dyn Future<Output = O> + 'a>))))
}
