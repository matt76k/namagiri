pub mod posit;
pub mod quire;
pub mod flquire;

use posit::Posit;
use quire::Quire;
use flquire::FLQuire;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use ndarray::{ArrayD, Array2};
use numpy::{IntoPyArray, PyArrayDyn, PyReadonlyArrayDyn, PyReadonlyArray2, PyArray2};

#[pyfunction]
fn matmul<'p>(py: Python<'p>, a: PyReadonlyArray2<u32>,  b: PyReadonlyArray2<u32>, _n: u8, es: u8) -> PyResult<&'p PyArray2<u32>> {
    if es == 1 {
        let ap: Array2<Quire<8, 1>> = a.as_array().mapv(|i| Posit::<8, 1>(i).into());
        let bp: Array2<Quire<8, 1>> = b.as_array().mapv(|i| Posit::<8, 1>(i).into());

        let cp = ap.dot(&bp).mapv(|i| Posit::<8, 1>::from(i).0);
        Ok(cp.into_pyarray(py))
    }
    else {
        let ap: Array2<Quire<8, 0>> = a.as_array().mapv(|i| Posit::<8, 0>(i).into());
        let bp: Array2<Quire<8, 0>> = b.as_array().mapv(|i| Posit::<8, 0>(i).into());

        let cp = ap.dot(&bp).mapv(|i| Posit::<8, 0>::from(i).0);
        Ok(cp.into_pyarray(py))
    }
}

#[pyfunction]
fn matmul_fl<'p>(py: Python<'p>, a: PyReadonlyArray2<u32>,  b: PyReadonlyArray2<u32>, _n: u8, _es: u8) -> PyResult<&'p PyArray2<u32>> {
    let ap: Array2<FLQuire<8, 1>> = a.as_array().mapv(|i| Posit::<8, 1>(i).into());
    let bp: Array2<FLQuire<8, 1>> = b.as_array().mapv(|i| Posit::<8, 1>(i).into());

    let cp = ap.dot(&bp).mapv(|i| Posit::<8, 1>::from(i).0);
    Ok(cp.into_pyarray(py))
}

#[pyfunction]
fn matmul_p<'p>(py: Python<'p>, a: PyReadonlyArray2<u32>,  b: PyReadonlyArray2<u32>, _n: u8, _es: u8) -> PyResult<&'p PyArray2<u32>> {
    let ap: Array2<Posit<8, 1>> = a.as_array().mapv(|i| Posit::<8, 1>(i));
    let bp: Array2<Posit<8, 1>> = b.as_array().mapv(|i| Posit::<8, 1>(i));

    let cp = ap.dot(&bp).mapv(|i| i.0);
    Ok(cp.into_pyarray(py))
}

#[pyfunction]
fn matmul2<'p>(py: Python<'p>, a: PyReadonlyArray2<f32>,  b: PyReadonlyArray2<f32>, _n: u8, _es: u8) -> PyResult<&'p PyArray2<f32>> {

    let cp = a.as_array().dot(&b.as_array());
    Ok(cp.into_pyarray(py))
}

#[pyfunction]
fn add<'p>(py: Python<'p>, a:PyReadonlyArrayDyn<'p, u32>,  b: PyReadonlyArrayDyn<'p, u32>, _n: u8, _es: u8) -> PyResult<&'p PyArrayDyn<u32>> {
    let ap: ArrayD<Posit<8, 1>> = a.as_array().mapv(|i| Posit::<8, 1>(i));
    let bp: ArrayD<Posit<8, 1>> = b.as_array().mapv(|i| Posit::<8, 1>(i));

    let cp = (ap + bp).mapv(|i| i.0);

    Ok(cp.into_pyarray(py))
}

#[pyfunction]
fn add2<'p>(py: Python<'p>, a:PyReadonlyArrayDyn<'p, f32>,  b: PyReadonlyArrayDyn<'p, f32>, _n: u8, _es: u8) -> PyResult<&'p PyArrayDyn<f32>> {
    let ap: ArrayD<f32> = a.as_array().to_owned();
    let bp: ArrayD<f32> = b.as_array().to_owned();

    let cp = ap + bp;

    Ok(cp.into_pyarray(py))
}

#[pyfunction]
fn to_f32<'p>(py: Python<'p>, x: PyReadonlyArrayDyn<'p, u32>, n: u8, es: u8) -> PyResult<&'p PyArrayDyn<f32>> {

    let p: ArrayD<f32> =
    match n {
        8 => match es  {
            5 => x.as_array().mapv(|i| f32::from(Posit::<8, 5>(i))),
            4 => x.as_array().mapv(|i| f32::from(Posit::<8, 4>(i))),
            3 => x.as_array().mapv(|i| f32::from(Posit::<8, 3>(i))),
            2 => x.as_array().mapv(|i| f32::from(Posit::<8, 2>(i))),
            1 => x.as_array().mapv(|i| f32::from(Posit::<8, 1>(i))),
            _ => x.as_array().mapv(|i| f32::from(Posit::<8, 0>(i))),
        },
        16 => match es  {
            3 => x.as_array().mapv(|i| f32::from(Posit::<16, 3>(i))),
            2 => x.as_array().mapv(|i| f32::from(Posit::<16, 2>(i))),
            _ => x.as_array().mapv(|i| f32::from(Posit::<16, 1>(i))),
        },
        _ => x.as_array().mapv(|i| f32::from(Posit::<8, 1>(i))),
    };

    Ok(p.into_pyarray(py))
}

#[pyfunction]
fn to_posit<'p>(py: Python<'p>, x: PyReadonlyArrayDyn<'p, f32>, n: u8, es: u8) -> PyResult<&'p PyArrayDyn<u32>> {

    let p: ArrayD<u32> =
    match n {
        8 => match es  {
            5 => x.as_array().mapv(|i| Posit::<8, 5>::from(i).0),
            4 => x.as_array().mapv(|i| Posit::<8, 4>::from(i).0),
            3 => x.as_array().mapv(|i| Posit::<8, 3>::from(i).0),
            2 => x.as_array().mapv(|i| Posit::<8, 2>::from(i).0),
            1 => x.as_array().mapv(|i| Posit::<8, 1>::from(i).0),
            _ => x.as_array().mapv(|i| Posit::<8, 0>::from(i).0),
        },
        16 => match es  {
            3 => x.as_array().mapv(|i| Posit::<16, 3>::from(i).0),
            2 => x.as_array().mapv(|i| Posit::<16, 2>::from(i).0),
            _ => x.as_array().mapv(|i| Posit::<16, 1>::from(i).0),
        },
        _ => x.as_array().mapv(|i| Posit::<8, 1>::from(i).0),
    };

    Ok(p.into_pyarray(py))
}

#[pymodule]
fn namagiri(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(to_posit, m)?)?;
    m.add_function(wrap_pyfunction!(to_f32, m)?)?;
    m.add_function(wrap_pyfunction!(matmul, m)?)?;
    m.add_function(wrap_pyfunction!(matmul_fl, m)?)?;
    m.add_function(wrap_pyfunction!(matmul_p, m)?)?;
    m.add_function(wrap_pyfunction!(matmul2, m)?)?;

    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(add2, m)?)?;

    Ok(())
}
